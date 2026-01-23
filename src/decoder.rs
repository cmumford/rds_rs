#![allow(dead_code)]

use crate::callbacks::RdsDecoderCallbacks;
use crate::types::{Group, GroupType, ProgramInformation, RdsData};
use modular_bitfield_msb::prelude::*;

#[bitfield(bits = 16)]
#[derive(Default, Clone, PartialEq, Eq)]
pub struct GenericGroupB {
    group_type: GroupType, // Group type (code + version).
    traffic_program: bool, // TP bit.
    program_type: B5,      // Group version (A/B).
    spare: B5,             // Spare.
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

    pub fn decode(&mut self, blocks: &Group) {
        if blocks.a.is_some() {
            self.rds_data.program_information =
                ProgramInformation::from_bytes(blocks.a.unwrap().to_be_bytes());
        }
        if blocks.b.is_none() {
            return;
        }
        let group_type = GroupType::default();
        self.callbacks.on_oda(0, &self.rds_data, &group_type);
    }
}
