use crate::af_table_group::AltFreqTableGroup;
use crate::eon::EonData;
use crate::oda::OdaEntry;
use crate::ps::PsData;
use crate::ptyn::PtynData;
use crate::radiotext::RtData;
use crate::types::{
    Clock, Content, EwsData, ProgramInformation, ProgramType, RdsPic, SlcData, TdcData,
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
    pub program_item_number: RdsPic,

    /// Program Type (PTY)
    pub program_type: ProgramType,

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
    pub alternative_freqs: AltFreqTableGroup,

    /// Enhanced Other Networks
    pub eon: EonData,

    /// Active Open Data Applications
    pub oda: LinearMap<u16, OdaEntry, 10>,

    /// Transparent Data Channels
    pub tdc: TdcData,

    /// Emergency Warning System
    pub ews: EwsData,

    /// Bitmask of which fields are valid
    pub valid: ValidFields,
}
