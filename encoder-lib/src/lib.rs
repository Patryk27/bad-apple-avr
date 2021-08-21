mod block;
mod frame;
mod frames;
mod image;
mod packet;
mod params;
mod source;
mod stats;

use self::{block::*, frame::*, frames::*, image::*, packet::*};
pub use ::image::RgbImage;
use anyhow::{ensure, Context, Result};
use std::array;

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

        let ctxt = FrameCtxt {
            params: self.params,
            prev: self.prev.as_ref(),
            curr: &curr,
        };

        let candidates = [
            IFrame::build_packet(ctxt),
            DFrame::build_packet(ctxt),
            PFrame::build_packet(ctxt),
        ];

        let packet = array::IntoIter::new(candidates)
            .into_iter()
            .flatten()
            .min_by_key(|a| a.len())
            .unwrap(); // shouldn't happen as iframes are always present

        if self.write(packet).is_ok() {
            self.stats.frames += 1;
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

        if self.buffer.len() + data.len() > (self.params.size_limit as usize) {
            Err(())
        } else {
            *self.stats.packets.entry(ty).or_default() += 1;
            self.stats.bytes += data.len();
            self.buffer.extend(data);

            Ok(())
        }
    }
}
