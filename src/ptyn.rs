#![allow(dead_code)]

use crate::radiotext::BLANK_CHAR;
use crate::rds::RdsData;
use crate::types::{Group, GroupType, ProgramType, ValidFields};
use modular_bitfield_msb::prelude::*;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PtynData {
    pub display: [u8; 8],
    // TODO: Unify A/B flag types.
    pub last_ab: bool,
}

impl Default for PtynData {
    fn default() -> Self {
        let mut spaces = [0u8; 8];
        spaces.fill(' ' as u8);

        Self {
            display: spaces,
            last_ab: false,
        }
    }
}

pub fn decode_ptyn(group: &Group, rds_data: &mut RdsData) -> ValidFields {
    // See RBDS Standard section 3.1.5.14.
    #[bitfield(bits = 16)]
    struct BlockB {
        group_type: GroupType,     // Group type (code + version).
        traffic_program: bool,     // TP bit.
        program_type: ProgramType, // PTY: Program type.
        ab_flag: bool,
        reserved: B3,
        segment_addr: B1,
    }
    let block_b = BlockB::from_bytes(group.b.unwrap().to_be_bytes());
    if rds_data.ptyn.last_ab != block_b.ab_flag() {
        rds_data.ptyn.display.fill(BLANK_CHAR as u8);
        rds_data.ptyn.last_ab = block_b.ab_flag();
    }

    let base: usize = 4 * (block_b.segment_addr() as usize);
    if group.c.is_some() {
        rds_data.ptyn.display[base + 0] = (group.c.unwrap() >> 8) as u8;
        rds_data.ptyn.display[base + 1] = (group.c.unwrap() & 0xff) as u8;
    }
    if group.d.is_some() {
        rds_data.ptyn.display[base + 2] = (group.d.unwrap() >> 8) as u8;
        rds_data.ptyn.display[base + 3] = (group.d.unwrap() & 0xff) as u8;
    }
    ValidFields::new().with_ptyn(true)
}
