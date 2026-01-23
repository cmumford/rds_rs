#![allow(dead_code)]

use crate::callbacks::RdsDecoderCallbacks;
use crate::types::{
    Group, GroupType, GroupVersion, ProgramInformation, ProgramType, RdsData, TrafficCodes,
};
use modular_bitfield_msb::prelude::*;

/// All type B blocks share the same 11-bit common prefix.
/// See RDS Standard section 3.1.4.2.
#[bitfield(bits = 11)]
#[derive(BitfieldSpecifier, Default, Clone, PartialEq, Eq)]
pub struct BlockBCommon {
    group_type: GroupType,     // Group type (code + version).
    traffic_program: bool,     // TP bit.
    program_type: ProgramType, // PTY: Program type.
}

// See RDS Standard section 3.1.5.1.
#[bitfield(bits = 16)]
#[derive(Default, Clone, PartialEq, Eq)]
pub struct GroupType0BlockB {
    common: BlockBCommon,         // Common block B fields.
    traffic_announcement: bool,   // TA bit: section 3.2.1.3.
    ms: bool,                     // M/S bit: section 3.2.1.4.
    decoder_identification: bool, // DI bit: section 3.2.1.5.
    c: B2,                        // Prog. service name and DI segment addr.
}

pub struct Decoder<'a> {
    callbacks: &'a mut dyn RdsDecoderCallbacks,
    rds_data: RdsData,
}

impl<'a> Decoder<'a> {
    pub fn new(callbacks: &'a mut dyn RdsDecoderCallbacks) -> Self {
        Decoder {
            callbacks,
            rds_data: RdsData::default(),
        }
    }

    fn decode_block_b_common(&mut self, block: &BlockBCommon) {
        // TODO: This is only setting the TP bit and not the TA bit.
        // Might have to decouple these if they come from different groups.
        if block.traffic_program() {
            self.rds_data.traffic = TrafficCodes::TrafficYes;
        } else {
            self.rds_data.traffic = TrafficCodes::TrafficNoEonNo;
        }
        self.rds_data.valid.set_tp_code(true);

        self.rds_data.program_type = block.program_type();
        self.rds_data.valid.set_pty(true);
    }

    pub fn decode_alt_freq(&mut self, _group: &Group) {}

    pub fn decode_ta(&mut self, _group: &Group) {}

    pub fn decode_ms(&mut self, _group: &Group) {}

    pub fn decode_group_type_0(&mut self, gt: GroupType, group: &Group) {
        if gt.version() == GroupVersion::A {
            self.decode_alt_freq(group);
        }
        if group.d.is_none() {
            return;
        }
        self.decode_ta(group);
        self.decode_ms(group);
    }

    pub fn decode(&mut self, group: &Group) {
        if group.a.is_some() {
            self.rds_data.program_information =
                ProgramInformation::from_bytes(group.a.unwrap().to_be_bytes());
        }
        if group.b.is_none() {
            return;
        }

        let generic_b = GroupType0BlockB::from_bytes(group.b.unwrap().to_be_bytes());
        self.decode_block_b_common(&generic_b.common());

        self.callbacks
            .on_oda(0, &self.rds_data, &GroupType::default());

        match generic_b.common().group_type().code() {
            0 => {
                self.decode_group_type_0(generic_b.common().group_type(), &group);
            }
            _ => {
                // Other group types not implemented yet
            }
        }
    }
}
