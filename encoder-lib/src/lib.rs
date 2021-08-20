mod block;
mod frames;
mod image;
mod packet;
mod params;
mod source;
mod stats;

mod prelude {
    pub(crate) use super::*;
}

use self::{block::*, frames::*, image::*, packet::*};
pub use ::image::RgbImage;
use anyhow::{ensure, Context, Result};

pub use self::{params::*, source::*, stats::*};

#[derive(Debug)]
pub struct Encoder<'a> {
    params: &'a Params,
    stats: Stats,
    buffer: Vec<u8>,
    prev: Option<Image>,
}

impl<'a> Encoder<'a> {
    pub fn new(params: &'a Params) -> Result<Self> {
        let mut this = Self {
            params,
            stats: Default::default(),
            buffer: Default::default(),
            prev: Default::default(),
        };

        let init = Packet::from(InitPacket::new(params));

        ensure!(
            this.write(init).is_ok(),
            "Couldn't write initialization packet"
        );

        Ok(this)
    }

    pub fn add(&mut self, curr: &RgbImage) -> bool {
        let curr = Image::new(&self.params, curr);

        let candidates = if let Some(prev) = self.prev.take() {
            if curr == prev {
                vec![Packet::from(DFrame::build())]
            } else {
                vec![
                    Packet::from(IFrame::build(&curr)),
                    Packet::from(PaFrame::build(&self.params, &prev, &curr)),
                    // Packet::from(PbFrame::build(&self.params, &prev, &curr)),
                    // Packet::from(PcFrame::build(&self.params, &prev, &curr)),
                    // Packet::from(PdFrame::build(&prev, &curr)),
                ]
            }
        } else {
            vec![Packet::from(IFrame::build(&curr))]
        };

        let packet = candidates
            .into_iter()
            .min_by(|a, b| {
                let a = a.len();
                let b = b.len();

                a.cmp(&b)
            })
            .unwrap();

        self.stats.frames += 1;

        if self.write(packet).is_ok() {
            self.prev = Some(curr);
            true
        } else {
            false
        }
    }

    pub fn finish(self) -> (Stats, String) {
        use std::fmt::Write;

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

    fn write(&mut self, packet: Packet) -> Result<(), ()> {
        let ty = packet.ty();
        let data = packet.serialize();

        if self.buffer.len() + data.len() > (self.params.limit as usize) {
            Err(())
        } else {
            *self.stats.packets.entry(ty).or_default() += 1;
            self.stats.bytes += data.len();
            self.buffer.extend(data);

            Ok(())
        }
    }
}
