use crate::prelude::*;

pub struct IFrame {
    body: Vec<u8>,
}

impl IFrame {
    pub fn build(curr: &Image) -> Self {
        Self {
            body: super::make_bitset(curr.pixels()),
        }
    }

    pub fn serialize(self) -> Vec<u8> {
        self.body
    }
}
