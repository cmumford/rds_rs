//! Rust translation of the RDS C structures and constants

/// Maximum number of transparent data channels we track
pub const NUM_TDC: usize = 32;

/// Number of transparent data bytes kept per channel
pub const TDC_LEN: usize = 32;

/// Block error rate classification
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BlockErrorCount {
    #[default]
    None = 0, // No block errors
    OneToTwo = 1,    // 1–2 block errors
    ThreeToFive = 2, // 3–5 block errors
    SixPlus = 3,     // 6+ block errors
}

/// Maximum acceptable block error rates per block
pub mod max_bler {
    use super::BlockErrorCount;

    pub const A: BlockErrorCount = BlockErrorCount::ThreeToFive;
    pub const B: BlockErrorCount = BlockErrorCount::OneToTwo;
    pub const C: BlockErrorCount = BlockErrorCount::ThreeToFive;
    pub const D: BlockErrorCount = BlockErrorCount::ThreeToFive;
}

/// Single RDS block (A, B, C, or D)
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RdsBlock {
    /// The 16-bit block value
    pub value: u16,
    /// Number of bit errors detected in this block
    pub errors: BlockErrorCount,
}

/// All four blocks of an RDS group
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RdsBlocks {
    pub a: Option<RdsBlock>,
    pub b: Option<RdsBlock>,
    pub c: Option<RdsBlock>,
    pub d: Option<RdsBlock>,
}

/// RDS group type (0–15) and version (A or B)
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RdsGroupType {
    /// Group type code (0..15)
    pub code: u8,
    /// 'A' or 'B'
    pub version: char,
}

/// Alternative frequency band
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RdsBand {
    Uhf = 0,
    #[default]
    LfMf = 1,
}

/// How an alternative frequency relates to the tuned frequency
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RdsAfAttribute {
    #[default]
    SameProgram = 0,
    RegionalVariant = 1,
}

/// Encoding method used for alternative frequencies
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RdsAfEncoding {
    #[default]
    Unknown = 0,
    MethodA = 1,
    MethodB = 2,
}

/// A single alternative frequency entry
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RdsFreq {
    pub band: RdsBand,
    pub attribute: RdsAfAttribute,
    /// Frequency value:
    /// - UHF: in 100 kHz steps (885 = 88.5 MHz)
    /// - LF/MF: in kHz (531 = 531 kHz)
    pub freq: u16,
}

/// Decoded table of alternative frequencies
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RdsAfTable {
    /// Tuned frequency (used in Method B)
    pub tuned_freq: RdsFreq,
    /// Number of valid entries in `entries`
    pub count: u8,
    /// Alternative frequencies
    pub entries: [RdsFreq; 25],
}

/// Internal state while decoding an AF table
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RdsAfDecodeTablePrivate {
    pub band: RdsBand,
    pub prev_encoding: RdsAfEncoding,
    pub expected_count: u8,
}

/// One AF decoding context
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RdsAfDecodeTable {
    pub table: RdsAfTable,
    pub encoding: RdsAfEncoding,
    pub pvt: RdsAfDecodeTablePrivate,
}

/// Group of multiple decoded AF tables
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RdsAfTableGroup {
    pub current_table_idx: i8,
    pub count: u8,
    pub tables: [RdsAfDecodeTable; 20],
}

/// Program Item Number Code (PIN)
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RdsPic {
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
}

/// Radiotext (RT) decoding state for one variant (A or B)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RdsRt {
    /// Final decoded text (64 bytes)
    pub display: [u8; 64],
}

impl Default for RdsRt {
    fn default() -> Self {
        let mut display = [0u8; 64];
        display.fill(b' ');

        Self { display }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RdsRtPrivate {
    pub high_prob: [u8; 64],
    pub low_prob: [u8; 64],
    pub high_prob_count: [u8; 64],
}

/// Which RT variant is currently being decoded
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum RdsRtVariant {
    #[default]
    A,
    B,
}

/// Clock Time and Date (CT)
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RdsClock {
    pub mjd_high: bool,
    pub mjd_low: u16,
    pub hour: u8,
    pub minute: u8,
    /// Local time offset from UTC in half-hours
    pub utc_offset_half_hours: i8,
}

/// Slow labelling code variant
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RdsSlcVariant {
    #[default]
    Paging = 0,
    TmcId = 1,
    PagingId = 2,
    Language = 3,
    NotAssigned4 = 4,
    NotAssigned5 = 5,
    Broadcaster = 6,
    Ews = 7,
}

/// Transparent Data Channel (TDC)
pub const TDC_DATA: [[u8; TDC_LEN]; NUM_TDC] = [[0; TDC_LEN]; NUM_TDC];

/// Bitflags indicating which RDS fields are valid / have been received
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RdsValidFlags(u32);

impl RdsValidFlags {
    pub const AF: Self = Self(0x00001);
    pub const CLOCK: Self = Self(0x00002);
    pub const EWS: Self = Self(0x00004);
    pub const FBT: Self = Self(0x00008);
    pub const MC: Self = Self(0x00010);
    pub const PIC: Self = Self(0x00020);
    pub const PI_CODE: Self = Self(0x00040);
    pub const PS: Self = Self(0x00080);
    pub const PTY: Self = Self(0x00100);
    pub const PTYN: Self = Self(0x00200);
    pub const RT: Self = Self(0x00400);
    pub const SLC: Self = Self(0x00800);
    pub const TDC: Self = Self(0x01000);
    pub const TA_CODE: Self = Self(0x02000);
    pub const TP_CODE: Self = Self(0x04000);
    pub const MS: Self = Self(0x08000);
    pub const EON: Self = Self(0x10000);
}

/// Main container for all decoded RDS data
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RdsData {
    /// Program Identification Code
    pub pi_code: u16,

    /// Program Item Number Code
    pub pic: RdsPic,

    /// Program Type (PTY)
    pub pty: u8,

    /// Traffic Program flag
    pub tp: bool,

    /// Traffic Announcement flag
    pub ta: bool,

    /// Music/Speech flag (true = music)
    pub music: bool,

    /// Program Service name (8 bytes, not null-terminated)
    pub ps: PsData,

    /// Radiotext
    pub rt: RtData,

    /// Clock time
    pub clock: RdsClock,

    /// Slow labelling codes
    pub slc: SlcData,

    /// Program Type Name (extended PTY)
    pub ptyn: PtynData,

    /// Alternative frequencies
    pub af: RdsAfTableGroup,

    /// Enhanced Other Networks
    pub eon: EonData,

    /// Active Open Data Applications
    pub oda: OdaData,

    /// Transparent Data Channels
    pub tdc: TdcData,

    /// Emergency Warning System
    pub ews: EwsData,

    /// Bitmask of which fields are valid
    pub valid: RdsValidFlags,

    pub stats: DevStats,
}

// ──────────────────────────────────────────────────────────────────────────────
// Helper sub-structures
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PsData {
    pub display: [u8; 8],
    pub pvt: PsPrivate,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PsPrivate {
    pub high_prob: [u8; 8],
    pub low_prob: [u8; 8],
    pub high_prob_count: [u8; 8],
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RtData {
    pub a: RdsRt,
    pub b: RdsRt,
    pub current_variant: RdsRtVariant,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SlcData {
    pub linkage_actuator: bool,
    pub variant: RdsSlcVariant,
    pub data: SlcPayload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlcPayload {
    Paging { paging: u8, country_code: u8 },
    TmcId(u16),
    PagingId(u16),
    LanguageCodes(u16),
    Broadcasters(u16),
    EwsChannelId(u16),
}

// TODO: Temporary. Delete the default value once the decoder is implemented.
impl Default for SlcPayload {
    fn default() -> Self {
        SlcPayload::TmcId(0)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PtynData {
    pub display: [u8; 8],
    pub last_ab: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EonData {
    pub on: EonOtherNetwork,
    pub maps: [EonMap; 5],
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EonOtherNetwork {
    pub ps: [u8; 8],
    pub pty: u8,
    pub tp: bool,
    pub ta: bool,
    pub af: RdsAfDecodeTable,
    pub pi_code: u16,
    pub pic: RdsPic,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EonMap {
    pub tn_tuned_freq: RdsFreq,
    pub on_freq: RdsFreq,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct OdaData {
    pub count: u8,
    pub entries: [OdaEntry; 10],
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct OdaEntry {
    pub id: u16,
    pub group_type: RdsGroupType,
    pub packet_count: u16,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TdcData {
    pub data: [[u8; TDC_LEN]; NUM_TDC],
    pub current_channel: u8,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EwsData {
    pub b: RdsBlock,
    pub c: RdsBlock,
    pub d: RdsBlock,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DevStats {
    pub packet_counts: [i32; 20],
    pub group_counts: [[u16; 2]; 16], // [A, B] per group type
    pub data_received_count: u16,
    pub block_b_error_count: u16,
}

pub trait RdsDecoderCallbacks {
    fn on_oda(&mut self, app_id: u16, rds_data: &RdsData, group_type: &RdsGroupType);
    fn on_clear(&mut self);
}
