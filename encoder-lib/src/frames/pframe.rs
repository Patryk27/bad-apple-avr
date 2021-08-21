use super::{compress, serialize_bitset};
use crate::*;

pub struct PFrame {
    body: Vec<u8>,
}

impl PFrame {
    pub fn serialize(self) -> Vec<u8> {
        self.body
    }
}

impl Frame for PFrame {
    fn build(ctxt: FrameCtxt<'_>) -> Option<Self> {
        let (params, prev, curr) = (ctxt.params, ctxt.prev?, ctxt.curr);
        let mut body = Vec::new();

        for bx in 0..params.xblocks() {
            for by in 0..params.yblocks() {
                let idx = params.block_idx(bx, by);
                let prev = prev.block(idx);
                let curr = curr.block(idx);

                if prev == curr {
                    body.push(false);
                } else {
                    body.push(true);
                    body.extend(prev.pixels().zip(curr.pixels()).map(|(a, b)| a != b));
                }
            }
        }

        let mut body = compress(serialize_bitset(body));

        let len = (body.len() as u16).to_le_bytes();
        body.insert(0, len[0]);
        body.insert(1, len[1]);

        Some(Self { body })
    }
}
