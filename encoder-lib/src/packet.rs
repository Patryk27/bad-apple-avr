use crate::*;

pub struct Packet {
    ty: u8,
    body: Vec<u8>,
}

pub struct InitPacket {
    body: Vec<u8>,
}

impl Packet {
    fn new(ty: u8, body: Vec<u8>) -> Self {
        Self { ty, body }
    }

    pub fn ty(&self) -> u8 {
        self.ty
    }

    pub fn len(&self) -> usize {
        1 + self.body.len()
    }

    pub fn serialize(self) -> Vec<u8> {
        let mut buffer = Vec::new();

        buffer.push(self.ty);
        buffer.extend(self.body);
        buffer
    }
}

impl From<InitPacket> for Packet {
    fn from(packet: InitPacket) -> Self {
        Self::new(0, packet.body)
    }
}

impl From<IFrame> for Packet {
    fn from(frame: IFrame) -> Self {
        Self::new(1, frame.serialize())
    }
}

impl From<DFrame> for Packet {
    fn from(frame: DFrame) -> Self {
        Self::new(2, frame.serialize())
    }
}

impl From<PFrame> for Packet {
    fn from(frame: PFrame) -> Self {
        Self::new(3, frame.serialize())
    }
}

impl InitPacket {
    pub fn new(params: &Params) -> Self {
        Self {
            body: vec![
                params.block_width,
                params.block_height,
                params.video_width,
                params.video_height,
            ],
        }
    }
}
