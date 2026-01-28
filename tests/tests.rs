use modular_bitfield_msb::prelude::*;
use rds::{GroupType, GroupVersion, ProgramType, RtVariant, rds_to_utf8_lossy};

#[cfg(test)]

mod tests {
    use super::*;

    #[bitfield]
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
        assert_eq!(block.traffic_program(), true);
        assert_eq!(block.program_type(), ProgramType::ClassicRock);
        assert_eq!(block.text_flag(), RtVariant::B);
        assert_eq!(block.text_segment_addr(), 10);
    }

    #[test]
    fn test_rt_convert_ascii() {
        assert_eq!(
            rds_to_utf8_lossy(
                "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789:{}[]();!\"*+-'./%&"
                    .as_bytes()
            ),
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789:{}[]();!\"*+-'./%&"
                .to_string()
        );
    }
}
