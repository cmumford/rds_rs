// Need to allow unused braces because of the way that the
// modular-bitfields-msb Debug attribute macro expands.
#![allow(unused_braces, clippy::unusual_byte_groupings)]
#[cfg(test)]
use crate::radiotext::RtVariant;
use crate::types::{GroupType, GroupVersion, ProgramType};
use modular_bitfield_msb::prelude::*;

#[bitfield(bits = 16)]
#[repr(u16)]
struct GroupType2BlockB {
    group_type: GroupType,     // Group type (code + version).
    traffic_program: bool,     // TP bit.
    program_type: ProgramType, // PTY: Program type.
    //        common: BlockBCommon,
    text_flag: RtVariant,
    text_segment_addr: B4,
}

#[test]
fn test_block_2b_decode() {
    // A test Block 2A decode
    //               |code|v|t| pty |F|addr|
    let data: u16 = 0b0010_0_1_00110_1_1010;

    let block = GroupType2BlockB::from(data);

    assert_eq!(block.group_type().code(), 2);
    assert_eq!(block.group_type().version(), GroupVersion::A);
    assert!(block.traffic_program());
    assert_eq!(block.program_type(), ProgramType::ClassicRock);
    assert_eq!(block.text_flag(), RtVariant::B);
    assert_eq!(block.text_segment_addr(), 10);
}
