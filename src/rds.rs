use crate::af_table_group::AltFreqTableGroup;
use crate::eon::EonData;
use crate::radiotext::RtData;
use crate::types::{
    Clock, Content, DevStats, EwsData, OdaEntry, ProgramInformation, ProgramType, PsData, PtynData,
    RdsPic, SlcData, TdcData, TrafficCodes, ValidFlags,
};
use heapless::LinearMap;

/// Main container for all decoded RDS data
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
    pub valid: ValidFlags,

    pub stats: DevStats,
}
