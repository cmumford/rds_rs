use crate::af_table::AltFreqTable;
use crate::types::{AltFreqEncoding, Band};

/// Internal state while decoding an AF table
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RdsAfDecodeTablePrivate {
    pub band: Band,
    pub prev_encoding: AltFreqEncoding,
    pub expected_count: u8,
}

/// One AF decoding context
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AltFreqDecodeTable {
    pub table: AltFreqTable,
    pub encoding: AltFreqEncoding,
    pub pvt: RdsAfDecodeTablePrivate,
}

impl AltFreqDecodeTable {
    pub fn decode_freq_table_start_block(&mut self, num_freqs_in_table: u8, second_byte: u8) {}

    pub fn decode_freq_table_nth_block(&mut self, first_byte: u8, second_byte: u8) {}
}
