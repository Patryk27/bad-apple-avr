use super::make_bitset;
use crate::prelude::*;

pub struct PbFrame {
    body: Vec<u8>,
}

impl PbFrame {
    pub fn build(params: &Params, prev: &Image, curr: &Image) -> Self {
        let mut bdeltas = Vec::new();
        let mut pdeltas = Vec::new();

        for bx in 0..params.xblocks() {
            for by in 0..params.yblocks() {
                let idx = params.block_idx(bx, by);
                let prev = prev.block(idx);
                let curr = curr.block(idx);

                if prev == curr {
                    bdeltas.push(false);
                } else {
                    bdeltas.push(true);
                    pdeltas.extend(prev.pixels().zip(curr.pixels()).map(|(a, b)| a != b));
                }
            }
        }

        let mut body = Vec::new();
        body.extend(make_bitset(bdeltas));
        body.extend(make_bitset(pdeltas));

        Self { body }
    }

    pub fn serialize(self) -> Vec<u8> {
        self.body
    }
}
