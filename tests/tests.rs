use modular_bitfield_msb::prelude::*;
use rds::{GroupType, GroupVersion, ProgramType, RtVariant};

#[cfg(test)]

mod tests {
    use super::*;

    // #[derive(Specifier, Debug, PartialEq, Eq)]
    // #[bits = 1]
    // pub enum MyGroupVersion {
    //     A = 0,
    //     B = 1,
    // }

    // #[bitfield(bits = 5)]
    // #[derive(Specifier, Default, Copy, Clone, PartialEq, Eq)]
    // pub struct MyGroupType {
    //     pub code: B4,                // Group type code.
    //     pub version: MyGroupVersion, // Group version (A/B).
    // }

    // #[derive(Specifier, Debug, Default, Clone, PartialEq, Eq)]
    // #[bits = 5]
    // pub enum MyProgramType {
    //     #[default]
    //     None = 0,
    //     News = 1,
    //     Information = 2,
    //     Sports = 3,
    //     Talk = 4,
    //     Rock = 5,
    //     ClassicRock = 6,
    //     AdultHits = 7,
    //     SoftRock = 8,
    //     Top40 = 9,
    //     Country = 10,
    //     Oldies = 11,
    //     Soft = 12,
    //     Nostalgia = 13,
    //     Jazz = 14,
    //     Classical = 15,
    //     RhythmAndBlues = 16,
    //     SoftRhythmAndBlues = 17,
    //     ForeignLanguage = 18,
    //     ReligiousMusic = 19,
    //     ReligiousTalk = 20,
    //     Personality = 21,
    //     Public = 22,
    //     College = 23,
    //     Unnasigned1 = 24,
    //     Unnasigned2 = 25,
    //     Unnasigned3 = 26,
    //     Unnasigned4 = 27,
    //     Unnasigned5 = 28,
    //     Weather = 29,
    //     EmergencyTest = 30,
    //     Emergency = 31,
    // }

    // #[derive(Specifier, Debug, Default, Clone, Copy, PartialEq, Eq)]
    // #[bits = 1]
    // pub enum MyRtVariant {
    //     #[default]
    //     A,
    //     B,
    // }

    #[bitfield(filled = false)]
    #[derive(BitfieldSpecifier, PartialEq, Eq, Copy, Clone)]
    struct BlockBCommon {
        group_type: GroupType,     // Group type (code + version).
        traffic_program: bool,     // TP bit.
        program_type: ProgramType, // PTY: Program type.
    }

    #[bitfield]
    #[cfg_attr(not(feature = "unknown"), repr(u16))]
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
}
