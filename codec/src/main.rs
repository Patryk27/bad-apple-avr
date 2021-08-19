use image::{io::Reader as ImageReader, Pixel, RgbImage};
use indicatif::ParallelProgressIterator;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{collections::BTreeMap, fmt::Write};

#[derive(Debug)]
struct Codec {
    params: Params,
    limit: usize,
    stats: Stats,
    buffer: Vec<u8>,
    prev: Option<Frame>,
}

impl Codec {
    fn new(params: Params, limit: usize) -> Self {
        let mut this = Self {
            params,
            limit,
            stats: Default::default(),
            buffer: Default::default(),
            prev: Default::default(),
        };

        this.write(Packet::new(
            0,
            vec![
                this.params.block_width as u8,
                this.params.block_height as u8,
                this.params.video_width as u8,
                this.params.video_height as u8,
            ],
        ))
        .expect("Couldn't write initialization packet");

        this
    }

    fn add(&mut self, curr: Frame) -> bool {
        let packet_candidates = if let Some(prev) = self.prev.take() {
            if curr == prev {
                vec![Self::build_dframe()]
            } else {
                vec![
                    Self::build_iframe(&curr),
                    Self::build_paframe(&self.params, &prev, &curr),
                    Self::build_pbframe(&self.params, &prev, &curr),
                    Self::build_pcframe(&prev, &curr),
                ]
            }
        } else {
            vec![Self::build_iframe(&curr)]
        };

        let packet = packet_candidates
            .into_iter()
            .min_by(|a, b| {
                let a = a.body.len();
                let b = b.body.len();

                a.cmp(&b)
            })
            .unwrap();

        self.stats.frames += 1;
        *self.stats.packets.entry(packet.ty).or_default() += 1;

        if self.write(packet).is_ok() {
            self.prev = Some(curr);
            true
        } else {
            false
        }
    }

    fn build_iframe(curr: &Frame) -> Packet {
        Packet::new(1, Self::encode_bitset(curr.pixels()))
    }

    fn build_dframe() -> Packet {
        Packet::new(2, vec![])
    }

    fn build_paframe(params: &Params, prev: &Frame, curr: &Frame) -> Packet {
        let mut bdeltas = Vec::new();
        let mut pdeltas = Vec::new();

        for bx in 0..params.xblocks() {
            for by in 0..params.yblocks() {
                let prev = &prev.blocks[params.block_idx(bx, by) as usize];
                let curr = &curr.blocks[params.block_idx(bx, by) as usize];

                if prev == curr {
                    bdeltas.push(false);
                } else {
                    bdeltas.push(true);
                    pdeltas.extend(prev.pixels.iter().zip(&curr.pixels).map(|(a, b)| a != b));
                }
            }
        }

        Packet::new(3, {
            let mut body = Vec::new();
            body.extend(Self::encode_bitset(bdeltas));
            body.extend(Self::encode_bitset(pdeltas));
            body
        })
    }

    fn build_pbframe(params: &Params, prev: &Frame, curr: &Frame) -> Packet {
        let mut body = Vec::new();

        for bx in 0..params.xblocks() {
            for by in 0..params.yblocks() {
                let prev = &prev.blocks[params.block_idx(bx, by) as usize];
                let curr = &curr.blocks[params.block_idx(bx, by) as usize];

                if prev != curr {
                    body.push(params.block_idx(bx, by) as u8);

                    body.extend(Self::encode_bitset(
                        prev.pixels
                            .iter()
                            .zip(&curr.pixels)
                            .map(|(a, b)| a != b)
                            .collect(),
                    ));
                }
            }
        }

        Packet::new(4, body)
    }

    fn build_pcframe(prev: &Frame, curr: &Frame) -> Packet {
        let mut body = Vec::new();

        for (idx, (a, b)) in prev.pixels().into_iter().zip(curr.pixels()).enumerate() {
            if a != b {
                body.push(idx as u8);
            }
        }

        Packet::new(5, body)
    }

    fn encode_bitset(data: Vec<bool>) -> Vec<u8> {
        data.chunks(8)
            .map(|pixels| {
                pixels
                    .iter()
                    .enumerate()
                    .fold(
                        0u8,
                        |delta, (bit_idx, bit)| {
                            if *bit {
                                delta | (1 << bit_idx)
                            } else {
                                delta
                            }
                        },
                    )
            })
            .collect()
    }

    fn write(&mut self, packet: Packet) -> Result<(), ()> {
        let packet = packet.serialize();

        if self.buffer.len() + packet.len() > self.limit {
            Err(())
        } else {
            self.stats.bytes += packet.len();
            self.buffer.extend(packet);

            Ok(())
        }
    }

    fn finish(self) -> (Stats, String) {
        let mut code = String::new();

        writeln!(code, "const uint8_t video[] PROGMEM = {{").unwrap();

        for bytes in self.buffer.chunks(25) {
            write!(code, "  ").unwrap();

            for byte in bytes {
                write!(code, "{:#04x}, ", byte).unwrap();
            }

            writeln!(code).unwrap();
        }

        writeln!(code, "}};").unwrap();

        (self.stats, code)
    }
}

#[derive(Clone, Debug)]
struct Params {
    block_width: u32,
    block_height: u32,
    video_width: u32,
    video_height: u32,
}

impl Params {
    fn xblocks(&self) -> u32 {
        self.video_width / self.block_width
    }

    fn yblocks(&self) -> u32 {
        self.video_height / self.block_height
    }

    fn block_x0(&self, bx: u32) -> u32 {
        bx * self.block_width
    }

    fn block_x1(&self, bx: u32) -> u32 {
        self.block_x0(bx) + self.block_width
    }

    fn block_y0(&self, by: u32) -> u32 {
        by * self.block_height
    }

    fn block_y1(&self, by: u32) -> u32 {
        self.block_y0(by) + self.block_height
    }

    fn block_idx(&self, bx: u32, by: u32) -> u32 {
        bx * self.yblocks() + by
    }
}

#[derive(Debug, Default)]
struct Stats {
    bytes: usize,
    frames: usize,
    packets: BTreeMap<u8, usize>,
}

#[derive(Clone, Debug)]
struct Packet {
    ty: u8,
    body: Vec<u8>,
}

impl Packet {
    pub fn new(ty: u8, body: Vec<u8>) -> Self {
        Self { ty, body }
    }

    pub fn serialize(self) -> Vec<u8> {
        let mut buffer = Vec::new();

        buffer.push(self.ty);
        buffer.extend(Self::compress_body(self.body));
        buffer
    }

    fn compress_body(body: Vec<u8>) -> Vec<u8> {
        enum State {
            Awaiting,
            Compressing { prev: u8, len: u8 },
        }

        let mut out = Vec::new();
        let mut state = State::Awaiting;

        for curr in body {
            state = match state {
                State::Awaiting => State::Compressing { prev: curr, len: 1 },

                State::Compressing { mut prev, mut len } => {
                    if curr == prev {
                        if len == 255 {
                            out.push(curr);
                            out.push(curr);
                            out.push(255);

                            len = 1;
                        } else {
                            len += 1;
                        }
                    } else {
                        if len == 1 {
                            out.push(prev);
                        } else {
                            out.push(prev);
                            out.push(prev);
                            out.push(len);
                        }

                        prev = curr;
                        len = 1;
                    }

                    State::Compressing { prev, len }
                }
            };
        }

        if let State::Compressing { prev, len } = state {
            out.push(prev);
            out.push(prev);
            out.push(len);
        }

        out
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Frame {
    blocks: Vec<Block>,
}

impl Frame {
    fn new(params: &Params, img: &RgbImage) -> Self {
        let mut blocks = Vec::new();

        for bx in 0..params.xblocks() {
            for by in 0..params.yblocks() {
                blocks.push(Block::new(params, img, bx, by));
            }
        }

        Self { blocks }
    }

    fn pixels(&self) -> Vec<bool> {
        self.blocks
            .iter()
            .flat_map(|block| &block.pixels)
            .copied()
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Block {
    pixels: Vec<bool>,
}

impl Block {
    fn new(params: &Params, img: &RgbImage, bx: u32, by: u32) -> Self {
        let mut pixels = Vec::new();

        for x in params.block_x0(bx)..params.block_x1(bx) {
            for y in params.block_y0(by)..params.block_y1(by) {
                let p = {
                    let p = img.get_pixel(x, y);
                    let r = p.channels()[0] as f32;
                    let g = p.channels()[1] as f32;
                    let b = p.channels()[2] as f32;

                    (r + g + b) / 3.0 / 255.0
                };

                pixels.push(p >= 0.5);
            }
        }

        Self { pixels }
    }
}

fn main() {
    let video_width = 60;
    let video_height = 30;

    // Step 1: Prepare parameters
    let params = {
        let mut params = Vec::new();

        for block_width in 1..84 {
            if video_width % block_width != 0 {
                continue;
            }

            for block_height in 1..48 {
                if video_height % block_height != 0 {
                    continue;
                }

                params.push(Params {
                    block_width,
                    block_height,
                    video_width,
                    video_height,
                });
            }
        }

        params
    };

    // Step 2: Perform encodings
    let results: Vec<_> = {
        let params_len = params.len();

        params
            .into_par_iter()
            .progress_count(params_len as u64)
            .map(|params| {
                let (stats, code) = encode(params.clone());
                (params, stats, code)
            })
            .collect()
    };

    // Step 3: Choose the best encoding
    let (params, stats, code) = {
        results
            .into_iter()
            .max_by(|(_, a, _), (_, b, _)| {
                let a = a.frames;
                let b = b.frames;

                a.cmp(&b)
            })
            .unwrap()
    };

    eprintln!("{:#?}", params);
    eprintln!("{:#?}", stats);
    eprintln!("({} frames total)", stats.frames);
    println!("{}", code);
}

fn encode(params: Params) -> (Stats, String) {
    let mut codec = Codec::new(params, 30500);

    let frames = glob::glob("../video/*.bmp").expect("Couldn't find frames");

    for frame in frames {
        let frame = frame.unwrap();

        let frame = ImageReader::open(frame)
            .expect("Couldn't open frame")
            .decode()
            .expect("Couldn't decode frame");

        let frame = Frame::new(&codec.params, &frame.into_rgb8());

        if !codec.add(frame) {
            break;
        }
    }

    codec.finish()
}
