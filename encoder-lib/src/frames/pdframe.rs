use crate::*;

pub struct PdFrame {
    body: Vec<u8>,
}

impl PdFrame {
    pub fn build(prev: &Image, curr: &Image) -> Self {
        let mut body = Vec::new();

        for (idx, (a, b)) in prev.pixels().into_iter().zip(curr.pixels()).enumerate() {
            if a != b {
                body.push(idx as u8);
            }
        }

        Self { body }
    }

    pub fn serialize(self) -> Vec<u8> {
        self.body
    }
}
