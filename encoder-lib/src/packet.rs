use crate::prelude::*;

pub struct Packet {
    ty: u8,
    body: Vec<u8>,
}

pub struct InitPacket {
    body: Vec<u8>,
}

impl Packet {
    fn new(ty: u8, body: Vec<u8>) -> Self {
        Self {
            ty,
            body: compress(body),
        }
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

impl From<PaFrame> for Packet {
    fn from(frame: PaFrame) -> Self {
        Self::new(3, frame.serialize())
    }
}

impl From<PbFrame> for Packet {
    fn from(frame: PbFrame) -> Self {
        Self::new(4, frame.serialize())
    }
}

impl From<PcFrame> for Packet {
    fn from(frame: PcFrame) -> Self {
        Self::new(5, frame.serialize())
    }
}

impl From<PdFrame> for Packet {
    fn from(frame: PdFrame) -> Self {
        Self::new(6, frame.serialize())
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

fn compress(body: Vec<u8>) -> Vec<u8> {
    enum State {
        Awaiting,
        Compressing { prev: u8, len: u8 },
    }

    let mut out = Vec::new();
    let mut state = State::Awaiting;

    for curr in body {
        state = match state {
            State::Awaiting => State::Compressing { prev: curr, len: 1 },

            State::Compressing { mut prev, mut len } => {
                if curr == prev {
                    if len == 255 {
                        out.push(curr);
                        out.push(curr);
                        out.push(255);

                        len = 1;
                    } else {
                        len += 1;
                    }
                } else {
                    if len == 1 {
                        out.push(prev);
                    } else {
                        out.push(prev);
                        out.push(prev);
                        out.push(len);
                    }

                    prev = curr;
                    len = 1;
                }

                State::Compressing { prev, len }
            }
        };
    }

    if let State::Compressing { prev, len } = state {
        out.push(prev);
        out.push(prev);
        out.push(len);
    }

    out
}
