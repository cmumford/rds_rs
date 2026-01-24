use crate::af_table::AltFreqTable;
use crate::types::Band;

/// Alternative frequency encoding method.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AltFreqEncoding {
    #[default]
    Unknown = 0,
    MethodA = 1,
    MethodB = 2,
}

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
