#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

/// An RDS group, when transmitted, is a 104 bit item consisting of 4 blocks
/// (A, B, C, D). Each block consists of 26 bits: a 16 data information word
/// followed by a 10 bit checkword. The receiver strips the 10 bit checkword,
/// and evaluates it to determine if the the block should be passed along for
/// decoding.
///
/// See the RDS Standard section 2.1.
#[derive(Clone, PartialEq, Eq)]
pub struct Group {
    pub a: Option<u16>, // Block A data word.
    pub b: Option<u16>, // Block B data word.
    pub c: Option<u16>, // Block C data word.
    pub d: Option<u16>, // Block D data word.
}

/// Maximum number of transparent data channels we track.
/// See the RBDS Standard section 4.18.
pub const NUM_TDC: usize = 32;

/// Number of transparent data bytes kept per channel
pub const TDC_LEN: usize = 32;

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
#[derive(BitfieldSpecifier, Default, Clone, PartialEq, Eq)]
pub struct GroupType {
    pub code: B4,              // Group type code.
    pub version: GroupVersion, // Group version (A/B).
}

/// Decoder identification and Dynamic PTY indicator / DI codes
/// See the RDS Standard section 3.2.1.5.
#[bitfield(bits = 4)]
#[derive(Default, Clone, PartialEq, Eq)]
pub struct DiCodes {
    dynamic_pty: bool,
    compressed: bool,
    artificial_head: bool,
    stereo: bool,
}

/// Alternative frequency band
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Band {
    #[default]
    Uhf = 0, // UHF band.
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

/// Program Item Number Code (PIN)
/// See the RBDS Standard section 3.2.1.7.
#[bitfield(bits = 16)]
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct RdsPic {
    pub day: B5,
    pub hour: B5,
    pub minute: B6,
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

/// Bitflags indicating which RDS fields are valid / have been received
#[bitfield(bits = 17)]
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct ValidFlags {
    pub af: bool,
    pub clock: bool,
    pub ews: bool,
    pub fbt: bool,
    pub mc: bool,
    pub pic: bool,
    pub pi_code: bool,
    pub ps: bool,
    pub pty: bool,
    pub ptyn: bool,
    pub rt: bool,
    pub slc: bool,
    pub tdc: bool,
    pub ta_code: bool,
    pub tp_code: bool,
    pub ms: bool,
    pub eon: bool,
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

#[derive(BitfieldSpecifier, Default, Clone, PartialEq, Eq)]
#[bits = 5]
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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PsData {
    pub display: [u8; 8],
    pub pvt: PsPrivate,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PsPrivate {
    pub hi_prob: [u8; 8],
    pub lo_prob: [u8; 8],
    pub hi_prob_cnt: [u8; 8],
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RtData {
    pub a: Radiotext,
    pub b: Radiotext,
    pub current_variant: RtVariant,
}

/// Slow labelling code variant
#[derive(BitfieldSpecifier, Default, Clone, PartialEq, Eq)]
#[bits = 3]
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

#[bitfield(bits = 16)]
#[derive(Default, Clone, PartialEq, Eq)]
pub struct SlcData {
    pub linkage_actuator: bool, // See RDSM spec. (3.2.1.8.3).
    pub variant: SlcVariant,
    pub data: B12,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PtynData {
    pub display: [u8; 8],
    pub last_ab: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TdcData {
    pub data: [[u8; TDC_LEN]; NUM_TDC],
    pub current_channel: u8,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EwsData {
    pub b: u16,
    pub c: u16,
    pub d: u16,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DevStats {
    pub packet_counts: [i32; 20],
    pub group_counts: [[u16; 2]; 16], // [A, B] per group type
    pub data_received_count: u16,
    pub block_b_error_count: u16,
}
