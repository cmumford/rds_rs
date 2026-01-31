#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use heapless::HistoryBuf;

/// An RDS group, when transmitted, is a 104 bit item consisting of 4 blocks
/// (A, B, C, D). Each block consists of 26 bits: a 16 data information word
/// followed by a 10 bit checkword. The receiver strips the 10 bit checkword,
/// and evaluates it to determine if the the block should be passed along for
/// decoding.
///
/// See the RDS Standard section 2.1.
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
#[derive(BitfieldSpecifier, Debug, PartialEq, Eq)]
#[bits = 1]
pub enum GroupVersion {
    A = 0,
    B = 1,
}

/// Group type code and version.
/// See the RDS Standard section 3.1.3 - Table 3.
#[bitfield(bits = 5)]
#[derive(BitfieldSpecifier, Default, Copy, Clone, PartialEq, Eq)]
pub struct GroupType {
    #[skip(setters)]
    pub code: B4, // Group type code.
    #[skip(setters)]
    pub version: GroupVersion, // Group version (A/B).
}

/// Decoder identification and Dynamic PTY indicator / DI codes
/// See the RDS Standard section 3.2.1.5.
#[bitfield(bits = 4)]
#[derive(Default, Clone, PartialEq, Eq)]
pub struct DiCodes {
    #[skip(setters)]
    dynamic_pty: bool,
    #[skip(setters)]
    compressed: bool,
    #[skip(setters)]
    artificial_head: bool,
    #[skip(setters)]
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
/// The scheduled broadcast start time and day of month as published by
/// the broadcaster.
/// See the RBDS Standard section 3.2.1.7.
#[bitfield(bits = 16)]
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct RdsPic {
    #[skip(setters)]
    pub day: B5,
    #[skip(setters)]
    pub hour: B5,
    #[skip(setters)]
    pub minute: B6,
}

/// Clock Time and Date (CT)
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Clock {
    pub mjd: u32,                  // Modified Julian Day.
    pub hour: u8,                  // UTC hour.
    pub minute: u8,                // UTC minute.
    pub utc_offset_half_hours: i8, // Local time offset from UTC in half-hours
}

const AVG_DAYS_PER_MONTH: f32 = 30.6001;
const AVT_DAYS_PER_YEAR: f32 = 365.25;
const MJD_JAN_1_2000: f32 = 15078.2; // Likely 2000 January 1, 04:48 UT

impl Clock {
    fn yp(&self) -> i32 {
        // Y' = int [ (MJD - 15078,2) / 365,25 ]
        (((self.mjd as f32) - 15078.2) / AVT_DAYS_PER_YEAR) as i32
    }
    fn k(&self) -> i32 {
        // If M' = 14 or M' = 15, then K = 1; else K = 0
        match self.mp() {
            14 | 15 => 1,
            _ => 0,
        }
    }
    pub fn year(&self) -> i32 {
        1900 + self.yp() + self.k()
    }
    fn mp(&self) -> i32 {
        // int { [ MJD - 14956,1 - int (Y' × 365,25) ] / 30,6001 }
        let a: f32 = (self.mjd as f32) - 14956.1;
        let b: f32 = (self.yp() as f32) * AVT_DAYS_PER_YEAR;
        ((a - b.floor()) / AVG_DAYS_PER_MONTH) as i32
    }
    pub fn month(&self) -> i32 {
        // M = M' - 1 - K × 12
        self.mp() - 1 - self.k() * 12
    }
    pub fn day(&self) -> i32 {
        // D = MJD - 14956 - int ( Y' × 365,25 ) - int ( M' × 30,6001 )
        let a: i32 = ((self.yp() as f32) * AVT_DAYS_PER_YEAR) as i32;
        let b: i32 = ((self.mp() as f32) * AVG_DAYS_PER_MONTH) as i32;
        (self.mjd as i32) - 14956 - a - b
    }
}

/// Bitflags indicating which RDS fields are valid / have been received
#[bitfield(bits = 17)]
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct ValidFields {
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
    #[skip(setters)]
    pub country_code: B4,
    #[skip(setters)]
    pub program_type: B4,
    #[skip(setters)]
    pub program_reference_number: u8,
}

// A combination of Traffic Program (TP) and Traffic Announcement (TA) codes
// See the RBDS Standard section 3.2.1.3.
#[bitfield(bits = 2)]
#[derive(Default, Clone, PartialEq, Eq)]
pub struct TrafficCodes {
    pub tp: bool, // Traffic Program code (TP).
    pub ta: bool, // Traffic Announcement code (TA).
}

#[derive(BitfieldSpecifier, Debug, Default, Clone, PartialEq, Eq)]
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
#[derive(BitfieldSpecifier, Debug, Default, Clone, PartialEq, Eq)]
#[bits = 1]
pub enum Content {
    Speech = 0,
    #[default]
    Music = 1,
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
    #[skip(setters)]
    pub linkage_actuator: bool, // See RDSM spec. (3.2.1.8.3).
    #[skip(setters)]
    pub variant: SlcVariant,
    #[skip(setters)]
    pub data: B12,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TdcData {
    pub data: [HistoryBuf<u8, TDC_LEN>; NUM_TDC],
    pub current_channel: u8,
}

#[bitfield(bits = 37)]
#[derive(Default, Clone, PartialEq, Eq)]
pub struct EwsData {
    // The data is the bottom five bits of block B, and all of C and D.
    //
    // The spec says:
    // > Format and application of these EWS message bits may be
    // > assigned unilaterally by each country.
    pub block_b_lsb: B5,
    pub block_c: u16,
    pub block_d: u16,
}
