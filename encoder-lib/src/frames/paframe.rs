use super::make_bitset;
use crate::prelude::*;

pub struct PaFrame {
    body: Vec<u8>,
}

impl PaFrame {
    pub fn build(params: &Params, prev: &Image, curr: &Image) -> Self {
        let mut body = Vec::new();

        for bx in 0..params.xblocks() {
            for by in 0..params.yblocks() {
                let idx = params.block_idx(bx, by);
                let prev = prev.block(idx);
                let curr = curr.block(idx);

                body.extend(prev.pixels().zip(curr.pixels()).map(|(a, b)| a == b));
            }
        }

        Self {
            body: make_bitset(body),
        }
    }

    pub fn serialize(self) -> Vec<u8> {
        self.body
    }
}
