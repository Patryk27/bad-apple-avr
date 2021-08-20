use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct Stats {
    pub bytes: usize,
    pub frames: usize,
    pub packets: BTreeMap<u8, usize>,
}
