use crate::alt_freq_decoder::AfDecoder;
use crate::alt_freq_table::AfTable;
use crate::oda::OdaEntry;
use crate::ps::PsData;
use crate::ptyn::PtynData;
use crate::radiotext::RtData;
use crate::types::{
    Clock, Content, DiCodes, EwsData, Pin, ProgramInformation, ProgramType, SlcData, TdcData,
    TrafficCodes, ValidFields,
};
use heapless::LinearMap;

/// Container for all decoded RDS data.
///
/// This struct is populated by the `Decoder`, which is passed many blocks
/// of raw RDS data to decode. Depending what information is broadcast
/// and passed to the decoder, some of the fields in this struct will contain
/// valid data. The `valid` field is a bitmask that should be used first before
/// dereferencing any members of this struct.
#[derive(Default, Clone, PartialEq)]
pub struct RdsData {
    /// Program Identification Code
    pub program_information: ProgramInformation,

    /// Program Item Number Code
    pub pin: Pin,

    /// Program Type (PTY)
    pub program_type: ProgramType,

    /// Traffic Program / Announcement codes.
    pub traffic: TrafficCodes,

    /// Music/Speech flag.
    pub content: Content,

    /// Program Service name (8 bytes, not null-terminated)
    pub ps: PsData,

    pub ps_on: PsData, // PS data for other network. RBDS spec. sect. 3.1.5.19.

    /// Radiotext
    pub rt: RtData,

    /// Clock time
    pub clock: Clock,

    /// Slow labelling codes
    pub slc: SlcData,

    /// Program Type Name (extended PTY)
    pub ptyn: PtynData,

    /// Active Open Data Applications
    pub oda: LinearMap<u16, OdaEntry, 10>,

    /// Transparent Data Channels
    pub tdc: TdcData,

    /// Emergency Warning System
    pub ews: EwsData,

    pub did_pty: DiCodes,

    pub alt_freqs: AfTable, // Alternative frequency table from group 0A.
    pub alt_freq_decoder: AfDecoder,

    pub on_freqs: AfTable, // Other network AFs. See RBDS spec. sect. 3.2.1.6.6.
    pub on_freq_decoder: AfDecoder,

    pub map_freqs: AfTable, // Mapped AFs. See RBDS spec. sect. 3.1.5.19 and 3.2.1.6.6.

    /// Bitmask of which fields are valid
    pub valid: ValidFields,
}
