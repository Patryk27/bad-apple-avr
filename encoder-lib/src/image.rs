use crate::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    blocks: Vec<Block>,
}

impl Image {
    pub fn new(params: &Params, img: &RgbImage) -> Self {
        let mut blocks = Vec::new();

        for bx in 0..params.xblocks() {
            for by in 0..params.yblocks() {
                blocks.push(Block::new(params, img, bx, by));
            }
        }

        Self { blocks }
    }

    pub fn block(&self, idx: usize) -> &Block {
        &self.blocks[idx]
    }

    pub fn pixels(&self) -> impl Iterator<Item = bool> + '_ {
        self.blocks.iter().flat_map(|block| block.pixels())
    }
}
