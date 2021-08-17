use image::{io::Reader as ImageReader, Pixel, RgbImage};
use std::{convert::TryInto, fmt::Write};

#[derive(Debug)]
struct Codec {
    params: Params,
    buffer: Vec<u8>,
    prev: Option<Frame>,
}

#[derive(Debug)]
struct Params {
    block_width: u32,
    block_height: u32,
    video_width: u32,
    video_height: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Frame {
    blocks: Vec<Block>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Block {
    pixels: Vec<bool>,
}

impl Codec {
    fn new(params: Params) -> Self {
        Self {
            params,
            buffer: Default::default(),
            prev: Default::default(),
        }
    }

    fn add(&mut self, curr: Frame) {
        let packet = if let Some(prev) = self.prev.take() {
            if curr == prev {
                eprintln!("-> dframe");
                Self::build_dframe()
            } else {
                let iframe = Self::build_iframe(&curr);
                let pframe = Self::build_pframe(&self.params, &prev, &curr);

                if iframe.len() < pframe.len() {
                    eprintln!("-> iframe");
                    iframe
                } else {
                    eprintln!("-> pframe");
                    pframe
                }
            }
        } else {
            eprintln!("-> iframe");
            Self::build_iframe(&curr)
        };

        self.buffer.extend(packet);
        self.prev = Some(curr);
    }

    fn build_iframe(curr: &Frame) -> Vec<u8> {
        let pixels = Self::encode_bitset(curr.pixels());

        Self::build_packet(0, pixels)
    }

    fn build_dframe() -> Vec<u8> {
        Self::build_packet(1, vec![])
    }

    fn build_pframe(params: &Params, prev: &Frame, curr: &Frame) -> Vec<u8> {
        let mut deltas = Vec::new();

        for bx in 0..params.xblocks() {
            for by in 0..params.yblocks() {
                let prev = &prev.blocks[params.block_idx(bx, by) as usize];
                let curr = &curr.blocks[params.block_idx(bx, by) as usize];

                let delta_pixels = prev
                    .pixels
                    .iter()
                    .zip(curr.pixels.iter())
                    .map(|(a, b)| a != b)
                    .collect();

                deltas.extend(Self::encode_bitset(delta_pixels));
            }
        }

        Self::build_packet(2, deltas)
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

    fn build_packet(ty: u8, body: Vec<u8>) -> Vec<u8> {
        let len: u16 = body.len().try_into().expect("Packet too large");

        if len > 512 {
            panic!("Packet too large: {} {:?}", len, body);
        }

        let mut buffer = Vec::new();

        buffer.push(ty);
        buffer.extend(len.to_le_bytes());
        buffer.extend(Self::compress_packet_body(body));
        buffer
    }

    fn compress_packet_body(body: Vec<u8>) -> Vec<u8> {
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

    fn len(&self) -> usize {
        self.buffer.len()
    }

    fn finish(mut self) -> String {
        self.buffer.extend(Self::build_packet(3, vec![]));

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

        code
    }
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
        bx * self.block_width + by
    }
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
    let mut codec = Codec::new(Params {
        block_width: 4,
        block_height: 2,
        video_width: 84,
        video_height: 48,
    });

    let frames = glob::glob("../video/*.bmp").expect("Couldn't find frames");

    for frame in frames {
        let frame = frame.unwrap();

        eprintln!("Processing: {:?}", frame);

        let frame = ImageReader::open(frame)
            .expect("Couldn't open frame")
            .decode()
            .expect("Couldn't decode frame");

        let frame = Frame::new(&codec.params, &frame.into_rgb8());

        codec.add(frame);

        if codec.len() > 29 * 1024 {
            eprintln!("Limit reached");
            break;
        }
    }

    println!("{}", codec.finish());
}
