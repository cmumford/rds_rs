#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

/// Maximum number of transparent data channels we track.
/// See the RBDS Standard section 4.18.
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

/// Single RDS block (A, B, C, or D)
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RdsBlock {
    /// The 16-bit block value
    pub value: u16,
    /// Number of bit errors detected in this block
    pub errors: BlockErrorCount,
}

/// Group type version.
/// See the RDS Standard section 3.1.3.
/// #[derive(BitfieldSpecifier)]
#[derive(BitfieldSpecifier, Clone, PartialEq, Eq)]
#[bits = 1]
pub enum GroupVersion {
    A = 0,
    B = 1,
}

/// Group type code and version.
/// See the RDS Standard section 3.1.3 - Table 3.
#[bitfield(bits = 5)]
#[derive(Default, Clone, PartialEq, Eq)]
pub struct GroupType {
    code: B4,
    version: GroupVersion,
}

/// Alternative frequency band
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Band {
    Uhf = 0, // UHF band.
    #[default]
    LfMf = 1, // LF/MF band.
}

/// How an alternative frequency relates to the tuned frequency
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AltFreqAttribute {
    #[default]
    SameProgram = 0,
    RegionalVariant = 1,
}

/// Alternative frequency encoding method.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AltFreqEncoding {
    #[default]
    Unknown = 0,
    MethodA = 1,
    MethodB = 2,
}

/// A single alternative frequency entry
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Frequency {
    pub band: Band,
    pub attribute: AltFreqAttribute,
    /// Frequency value:
    /// - UHF: in 100 kHz steps (885 = 88.5 MHz)
    /// - LF/MF: in kHz (531 = 531 kHz)
    pub freq: u16,
}

/// Decoded table of alternative frequencies
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AltFreqTable {
    /// Tuned frequency (used in Method B)
    pub tuned_freq: Frequency,
    /// Number of valid entries in `entries`
    pub count: u8,
    /// Alternative frequencies
    pub entries: [Frequency; 25],
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

/// Group of multiple decoded AF tables
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AltFreqTableGroup {
    pub current_table_idx: i8,
    pub count: u8,
    pub tables: [AltFreqDecodeTable; 20],
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
pub struct Radiotext {
    /// Final decoded text (64 bytes)
    pub display: [u8; 64],
}

impl Default for Radiotext {
    fn default() -> Self {
        let mut display = [0u8; 64];
        display.fill(b' ');

        Self { display }
    }
}

/// Which RT variant is currently being decoded
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum RtVariant {
    #[default]
    A,
    B,
}

/// Clock Time and Date (CT)
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Clock {
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
pub enum SlcVariant {
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

/// Bitflags indicating which RDS fields are valid / have been received
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ValidFlags(u32);

impl ValidFlags {
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

// Program identification codes and Extended country codes.
// See the RBDS Standard Annex D.
#[bitfield(bits = 16)]
#[derive(Default, Clone, PartialEq, Eq)]
pub struct ProgramInformation {
    pub country_code: B4,
    pub program_type: B4,
    pub program_reference_number: u8,
}

// A combination of Traffic Program (TP) and Traffic Announcement (TA) codes
// See the RBDS Standard section 3.2.1.3.
#[derive(Default, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum TrafficCodes {
    /// This program does not carry traffic announcements nor does it refer,
    /// via EON, to a program that does.
    #[default]
    TrafficNoEonNo = 0,
    /// This program carries EON information about another program which gives
    /// traffic information.
    TrafficNoEonYes = 1,
    /// This program carries traffic announcements but none are being broadcast
    /// at present and may also carry EON information about other traffic
    /// announcements.
    TrafficMaybeEonMaybe = 2,
    /// A traffic announcement is being broadcast on this program at present.
    TrafficYes = 3,
}

#[derive(Default, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum ProgramType {
    #[default]
    None = 0,
    News = 1,
    Information = 2,
    Sports = 3,
    Talk = 4,
    Rock = 5,
    ClassicRock = 6,
    AdultHits = 7,
    SoftRock = 8,
    Top40 = 9,
    Country = 10,
    Oldies = 11,
    Soft = 12,
    Nostalgia = 13,
    Jazz = 14,
    Classical = 15,
    RhythmAndBlues = 16,
    SoftRhythmAndBlues = 17,
    ForeignLanguage = 18,
    ReligiousMusic = 19,
    ReligiousTalk = 20,
    Personality = 21,
    Public = 22,
    College = 23,
    Unnasigned1 = 24,
    Unnasigned2 = 25,
    Unnasigned3 = 26,
    Unnasigned4 = 27,
    Unnasigned5 = 28,
    Weather = 29,
    EmergencyTest = 30,
    Emergency = 31,
}

/// Music/speech (M/S) switch code.
/// See the RBDS Standard section 3.2.1.4.
#[derive(Default, Clone, PartialEq, Eq)]
pub enum Content {
    Speech = 0,
    #[default]
    Music = 1,
}

/// Main container for all decoded RDS data
#[derive(Default, Clone, PartialEq, Eq)]
pub struct RdsData {
    /// Program Identification Code
    pub pi_code: ProgramInformation,

    /// Program Item Number Code
    pub pic: RdsPic,

    /// Program Type (PTY)
    pub pty: ProgramType,

    /// Traffic Program / Announcement codes.
    pub traffic: TrafficCodes,

    /// Music/Speech flag.
    pub content: Content,

    /// Program Service name (8 bytes, not null-terminated)
    pub ps: PsData,

    /// Radiotext
    pub rt: RtData,

    /// Clock time
    pub clock: Clock,

    /// Slow labelling codes
    pub slc: SlcData,

    /// Program Type Name (extended PTY)
    pub ptyn: PtynData,

    /// Alternative frequencies
    pub af: AltFreqTableGroup,

    /// Enhanced Other Networks
    pub eon: EonData,

    /// Active Open Data Applications
    pub oda: OdaData,

    /// Transparent Data Channels
    pub tdc: TdcData,

    /// Emergency Warning System
    pub ews: EwsData,

    /// Bitmask of which fields are valid
    pub valid: ValidFlags,

    pub stats: DevStats,
}

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
    pub a: Radiotext,
    pub b: Radiotext,
    pub current_variant: RtVariant,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SlcData {
    pub linkage_actuator: bool,
    pub variant: SlcVariant,
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
    pub af: AltFreqDecodeTable,
    pub pi_code: u16,
    pub pic: RdsPic,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EonMap {
    pub tn_tuned_freq: Frequency,
    pub on_freq: Frequency,
}

#[derive(Default, Clone, PartialEq, Eq)]
pub struct OdaData {
    pub count: u8,
    pub entries: [OdaEntry; 10],
}

#[derive(Default, Clone, PartialEq, Eq)]
pub struct OdaEntry {
    pub id: u16,
    pub group_type: GroupType,
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
    fn on_oda(&mut self, app_id: u16, rds_data: &RdsData, group_type: &GroupType);
    fn on_clear(&mut self);
}
