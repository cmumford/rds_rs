use crate::af_decode_table::AltFreqDecodeTable;
use crate::types::{Frequency, RdsPic};

#[derive(Default, Clone, PartialEq)]
pub struct EonData {
    pub on: EonOtherNetwork,
    pub maps: [EonMap; 5],
}

#[derive(Default, Clone, PartialEq)]
pub struct EonOtherNetwork {
    pub ps: [u8; 8],
    pub pty: u8,
    pub tp: bool,
    pub ta: bool,
    pub af: AltFreqDecodeTable,
    pub pi_code: u16,
    pub pic: RdsPic,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct EonMap {
    pub tn_tuned_freq: Frequency,
    pub on_freq: Frequency,
}
