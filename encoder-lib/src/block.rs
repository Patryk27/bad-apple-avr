use crate::prelude::*;
use ::image::Pixel;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pixels: Vec<bool>,
}

impl Block {
    pub fn new(params: &Params, img: &RgbImage, bx: u8, by: u8) -> Self {
        let mut pixels = Vec::new();

        for x in params.block_x0(bx)..params.block_x1(bx) {
            for y in params.block_y0(by)..params.block_y1(by) {
                let p = {
                    let p = img.get_pixel(x as u32, y as u32);
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

    pub fn pixels(&self) -> impl Iterator<Item = bool> + '_ {
        self.pixels.iter().copied()
    }
}
