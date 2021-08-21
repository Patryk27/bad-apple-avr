use crate::*;

#[derive(Clone, Debug)]
pub struct Params {
    pub(crate) block_width: u8,
    pub(crate) block_height: u8,
    pub(crate) video_width: u8,
    pub(crate) video_height: u8,
    pub(crate) size_limit: u16,
}

impl Params {
    pub fn new(
        block_width: u8,
        block_height: u8,
        video_width: u8,
        video_height: u8,
        size_limit: u16,
    ) -> Result<Self> {
        ensure!(video_width % block_width == 0);
        ensure!(video_height % block_height == 0);

        Ok(Self {
            block_width,
            block_height,
            video_width,
            video_height,
            size_limit,
        })
    }

    pub fn size_limit(&self) -> u16 {
        self.size_limit
    }

    pub(crate) fn xblocks(&self) -> u8 {
        self.video_width / self.block_width
    }

    pub(crate) fn yblocks(&self) -> u8 {
        self.video_height / self.block_height
    }

    pub(crate) fn block_x0(&self, bx: u8) -> u8 {
        bx * self.block_width
    }

    pub(crate) fn block_x1(&self, bx: u8) -> u8 {
        self.block_x0(bx) + self.block_width
    }

    pub(crate) fn block_y0(&self, by: u8) -> u8 {
        by * self.block_height
    }

    pub(crate) fn block_y1(&self, by: u8) -> u8 {
        self.block_y0(by) + self.block_height
    }

    pub(crate) fn block_idx(&self, bx: u8, by: u8) -> usize {
        (bx as usize) * (self.yblocks() as usize) + (by as usize)
    }
}
