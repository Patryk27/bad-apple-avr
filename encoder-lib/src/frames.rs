mod dframe;
mod iframe;
mod pframe;

pub use self::{dframe::*, iframe::*, pframe::*};

fn compress(body: Vec<u8>) -> Vec<u8> {
    enum State {
        AwaitingFirstByte,
        CompressingByte { prev: u8, run: u8 },
    }

    let mut out = Vec::new();
    let mut state = State::AwaitingFirstByte;

    for curr in body {
        state = match state {
            State::AwaitingFirstByte => State::CompressingByte { prev: curr, run: 0 },

            State::CompressingByte { mut prev, mut run } => {
                if curr == prev {
                    if run == 255 {
                        out.push(curr);
                        out.push(curr);
                        out.push(run);

                        run = 0;
                    } else {
                        run += 1;
                    }
                } else {
                    if run == 0 {
                        out.push(prev);
                    } else {
                        out.push(prev);
                        out.push(prev);
                        out.push(run);
                    }

                    prev = curr;
                    run = 0;
                }

                State::CompressingByte { prev, run }
            }
        };
    }

    if let State::CompressingByte { prev, run } = state {
        out.push(prev);
        out.push(prev);
        out.push(run);
    }

    out
}

fn serialize_bitset(data: impl IntoIterator<Item = bool>) -> Vec<u8> {
    let data: Vec<_> = data.into_iter().collect();

    data.chunks(8)
        .map(|pixels| {
            pixels.iter().enumerate().fold(
                0u8,
                |delta, (bit_idx, bit)| {
                    if *bit {
                        delta | (1 << bit_idx)
                    } else {
                        delta
                    }
                },
            )
        })
        .collect()
}
