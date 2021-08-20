mod dframe;
mod iframe;
mod paframe;
mod pbframe;
mod pcframe;
mod pdframe;

pub use self::{dframe::*, iframe::*, paframe::*, pbframe::*, pcframe::*, pdframe::*};

fn make_bitset(data: impl IntoIterator<Item = bool>) -> Vec<u8> {
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
