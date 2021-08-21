use crate::*;

pub struct DFrame;

impl DFrame {
    pub fn serialize(&self) -> Vec<u8> {
        Default::default()
    }
}

impl Frame for DFrame {
    fn build(ctxt: FrameCtxt<'_>) -> Option<Self> {
        if ctxt.curr == ctxt.prev? {
            Some(Self)
        } else {
            None
        }
    }
}
