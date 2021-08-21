use crate::*;

pub trait Frame
where
    Self: Sized,
    Packet: From<Self>,
{
    fn build(ctxt: FrameCtxt<'_>) -> Option<Self>;

    fn build_packet(ctxt: FrameCtxt<'_>) -> Option<Packet> {
        Self::build(ctxt).map(Packet::from)
    }
}

#[derive(Copy, Clone)]
pub struct FrameCtxt<'a> {
    pub params: &'a Params,
    pub prev: Option<&'a Image>,
    pub curr: &'a Image,
}
