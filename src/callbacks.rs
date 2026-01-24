use crate::eon::EonData;
use crate::types::{
    Clock, Content, DevStats, EwsData, GroupType, OdaData, ProgramInformation, ProgramType, PsData,
    PtynData, RdsPic, RtData, SlcData, TdcData, TrafficCodes, ValidFlags,
};

use crate::frequency_table_group::AltFreqTableGroup;

/// Main container for all decoded RDS data
#[derive(Default, Clone, PartialEq, Eq)]
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
    pub program_type_name: PtynData,

    /// Alternative frequencies
    pub alternative_freqs: AltFreqTableGroup,

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
pub trait RdsDecoderCallbacks {
    fn on_oda(&mut self, app_id: u16, rds_data: &RdsData, group_type: &GroupType);
    fn on_clear(&mut self);
}
