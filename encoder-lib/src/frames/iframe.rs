use super::{compress, serialize_bitset};
use crate::*;

pub struct IFrame {
    body: Vec<u8>,
}

impl IFrame {
    pub fn serialize(self) -> Vec<u8> {
        self.body
    }
}

impl Frame for IFrame {
    fn build(ctxt: FrameCtxt<'_>) -> Option<Self> {
        Some(Self {
            body: compress(serialize_bitset(ctxt.curr.pixels())),
        })
    }
}
