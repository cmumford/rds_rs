#![allow(dead_code)]

use crate::callbacks::{RdsData, RdsDecoderCallbacks};
use crate::radiotext::RtVariant;
use crate::types::{
    Group, GroupType, GroupVersion, ProgramInformation, ProgramType, RdsPic, SlcData, TrafficCodes,
};
use modular_bitfield_msb::prelude::*;

/// All type B blocks share the same 11-bit common prefix.
/// See RDS Standard section 3.1.4.2.
#[bitfield(bits = 11)]
#[derive(BitfieldSpecifier, Default, Clone, PartialEq, Eq)]
struct BlockBCommon {
    group_type: GroupType,     // Group type (code + version).
    traffic_program: bool,     // TP bit.
    program_type: ProgramType, // PTY: Program type.
}

// See RDS Standard section 3.1.5.1.
#[bitfield(bits = 16)]
#[derive(Default, Clone, PartialEq, Eq)]
struct GroupType0BlockB {
    common: BlockBCommon,         // Common block B fields.
    traffic_announcement: bool,   // TA bit: section 3.2.1.3.
    ms: bool,                     // M/S bit: section 3.2.1.4.
    decoder_identification: bool, // DI bit: section 3.2.1.5.
    c: B2,                        // Prog. service name and DI segment addr.
}

// See RDS Standard section 3.1.5.2.
#[bitfield(bits = 16)]
#[derive(Default, Clone, PartialEq, Eq)]
struct GroupType1BlockB {
    common: BlockBCommon,   // Common block B fields.
    radio_paging_codes: B5, // See Annex M.
}

// See RDS Standard section 3.1.5.3.
#[bitfield(bits = 16)]
#[derive(Default, Clone, PartialEq, Eq)]
struct GroupType2BlockB {
    common: BlockBCommon, // Common block B fields.
    text_flag: RtVariant, // See Annex M.
    text_segment_addr: B4,
}

pub struct Decoder<'a> {
    callbacks: &'a mut dyn RdsDecoderCallbacks,
    advanced_ps_decoding: bool,
    rds_data: RdsData,
}

impl<'a> Decoder<'a> {
    pub fn new(callbacks: &'a mut dyn RdsDecoderCallbacks) -> Self {
        Decoder {
            callbacks,
            advanced_ps_decoding: true,
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

    fn decode_alt_freq(&mut self, group: &Group) {
        if group.c.is_none() {
            return;
        }
        self.rds_data.valid.set_af(true);
        self.rds_data
            .alternative_freqs
            .decode_freq_group_block(group.c.unwrap());
    }

    fn decode_ta(&mut self, _group: &Group) {}

    fn decode_ms(&mut self, _group: &Group) {}

    fn update_ps_simple(&mut self, char_idx: u8, current_ps_byte: u8) {
        assert!(char_idx < 8);
        self.rds_data.ps.display[char_idx as usize] = current_ps_byte;
        self.rds_data.valid.set_ps(true);
    }

    /// Update the Program Service text in our buffers from the shadow registers.
    ///
    /// This implementation of the Program Service update attempts to display only
    /// complete messages for stations who rotate text through the PS field in
    /// violation of the RBDS standard as well as providing enhanced error detection.
    ///
    /// This function is from the Silicon Labs sample application.
    fn update_ps_advanced(&mut self, char_idx: usize, byte: u8) {
        const PS_VALIDATE_LIMIT: u8 = 2;

        let mut in_transition = false; // Indicates if the PS text is in transition.
        let mut complete = true; // Indicates the PS text is ready to be displayed.

        if self.rds_data.ps.pvt.hi_prob[char_idx] == byte {
            // The new byte matches the high probability byte.
            if self.rds_data.ps.pvt.hi_prob_cnt[char_idx] < PS_VALIDATE_LIMIT {
                self.rds_data.ps.pvt.hi_prob_cnt[char_idx] += 1;
            } else {
                // we have received this byte enough to max out our counter and push it
                // into the low probability array as well.
                self.rds_data.ps.pvt.hi_prob_cnt[char_idx] = PS_VALIDATE_LIMIT;
                self.rds_data.ps.pvt.lo_prob[char_idx] = byte;
            }
        } else if self.rds_data.ps.pvt.lo_prob[char_idx] == byte {
            // The new byte is a match with the low probability byte. Swap them, reset
            // the counter and flag the text as in transition. Note that the counter for
            // this character goes higher than the validation limit because it will get
            // knocked down later.
            if self.rds_data.ps.pvt.hi_prob_cnt[char_idx] >= PS_VALIDATE_LIMIT {
                in_transition = true;
                self.rds_data.ps.pvt.hi_prob_cnt[char_idx] = PS_VALIDATE_LIMIT + 1;
            } else {
                self.rds_data.ps.pvt.hi_prob_cnt[char_idx] = PS_VALIDATE_LIMIT;
            }
            self.rds_data.ps.pvt.lo_prob[char_idx] = self.rds_data.ps.pvt.hi_prob[char_idx];
            self.rds_data.ps.pvt.hi_prob[char_idx] = byte;
        } else if self.rds_data.ps.pvt.hi_prob_cnt[char_idx] == 0 {
            // The new byte is replacing an empty byte in the high probability array.
            self.rds_data.ps.pvt.hi_prob[char_idx] = byte;
            self.rds_data.ps.pvt.hi_prob_cnt[char_idx] = 1;
        } else {
            // The new byte doesn't match anything, put it in the low probability array.
            self.rds_data.ps.pvt.lo_prob[char_idx] = byte;
        }

        if in_transition {
            // When the text is changing, decrement the count for all characters to
            // prevent displaying part of a message that is in transition.
            for count in self.rds_data.ps.pvt.hi_prob_cnt.iter_mut() {
                if *count > 1 {
                    *count -= 1;
                }
            }
        }

        // The PS text is incomplete if any character in the high probability array
        // has been seen fewer times than the validation limit.
        for count in self.rds_data.ps.pvt.hi_prob_cnt.iter_mut() {
            if *count < PS_VALIDATE_LIMIT {
                complete = false;
                break;
            }
        }

        // If the PS text in the high probability array is complete copy it to the
        // display array.
        if complete {
            self.rds_data.valid.set_ps(true);
            self.rds_data
                .ps
                .display
                .copy_from_slice(&self.rds_data.ps.pvt.hi_prob);
        }
    }

    fn decode_group_type_0(&mut self, group: &Group) {
        let block_b = GroupType0BlockB::from_bytes(group.b.unwrap().to_be_bytes());
        if block_b.common().group_type().version() == GroupVersion::A {
            self.decode_alt_freq(group);
        }
        if group.d.is_none() {
            return;
        }
        self.decode_ta(group);
        self.decode_ms(group);

        let pair_idx = 2 * block_b.c();
        let d_val = group.d.unwrap();
        let hi_byte = (d_val >> 8) as u8;
        let lo_byte = (d_val & 0xFF) as u8;
        if self.advanced_ps_decoding {
            self.update_ps_advanced((pair_idx + 0) as usize, hi_byte);
            self.update_ps_advanced((pair_idx + 1) as usize, lo_byte);
        } else {
            self.update_ps_simple(pair_idx + 0, hi_byte);
            self.update_ps_simple(pair_idx + 1, lo_byte);
        }
    }

    fn decode_group_type_1(&mut self, group: &Group) {
        let block_b = GroupType1BlockB::from_bytes(group.b.unwrap().to_be_bytes());
        if block_b.common().group_type().version() == GroupVersion::A && group.c.is_some() {
            self.rds_data.slc = SlcData::from_bytes(group.c.unwrap().to_be_bytes());
            self.rds_data.valid.set_slc(true);
        }

        if group.d.is_some() {
            self.rds_data.program_item_number = RdsPic::from_bytes(group.d.unwrap().to_be_bytes());
            self.rds_data.valid.set_pic(true);
        } else {
            // Per spec (3.2.1.7): If a type 1 group is transmitted without a
            // valid PIN, the day of the month shall be set to zero. In this
            // case a receiver which evaluates PIN shall ignore the other
            // information in block 4.
            self.rds_data.program_item_number = RdsPic::default();
        }
    }

    fn decode_group_type_2(&mut self, group: &Group) {
        let block_b = GroupType2BlockB::from_bytes(group.b.unwrap().to_be_bytes());
        if block_b.common().group_type().version() == GroupVersion::A {
            if group.c.is_none() || group.d.is_none() {
                return;
            }
            let mut rtchars: [u8; 4] = [
                (group.c.unwrap() >> 8) as u8,
                (group.c.unwrap() & 0xff) as u8,
                (group.d.unwrap() >> 8) as u8,
                (group.d.unwrap() & 0xff) as u8,
            ];
            let rt = &mut self.rds_data.rt.a;
            let addr = 4 * block_b.text_segment_addr();
            rt.update_rt_simple(group, 4, addr as usize, &rtchars);
            if self.rds_data.rt.current_variant != block_b.text_flag() {
                rt.bump_rt_validation_count();
            }
            rt.update_rt_advance(group, 4, addr as usize, &mut rtchars);
            return;
        }

        // Version B.
        if group.d.is_none() {
            return;
        }
        let mut rtchars: [u8; 4] = [
            (group.d.unwrap() >> 8) as u8,
            (group.d.unwrap() & 0xff) as u8,
            0,
            0,
        ];
        let rt = &mut self.rds_data.rt.b;
        let addr = 4 * block_b.text_segment_addr();
        rt.update_rt_simple(group, 2, addr as usize, &rtchars);
        if self.rds_data.rt.current_variant != block_b.text_flag() {
            rt.bump_rt_validation_count();
        }
        rt.update_rt_advance(group, 2, addr as usize, &mut rtchars);
        return;
    }

    fn decode_group_type_3(&mut self, _group: &Group) {}

    fn decode_group_type_4(&mut self, _group: &Group) {}

    fn decode_group_type_5(&mut self, _group: &Group) {}

    fn decode_group_type_6(&mut self, _group: &Group) {}

    fn decode_group_type_7(&mut self, _group: &Group) {}

    fn decode_group_type_8(&mut self, _group: &Group) {}

    fn decode_group_type_9(&mut self, _group: &Group) {}

    fn decode_group_type_10(&mut self, _group: &Group) {}

    fn decode_group_type_11(&mut self, _group: &Group) {}

    fn decode_group_type_12(&mut self, _group: &Group) {}

    fn decode_group_type_13(&mut self, _group: &Group) {}

    fn decode_group_type_14(&mut self, _group: &Group) {}

    fn decode_group_type_15(&mut self, _group: &Group) {}

    pub fn decode(&mut self, group: &Group) {
        if group.a.is_some() {
            self.rds_data.program_information =
                ProgramInformation::from_bytes(group.a.unwrap().to_be_bytes());
        }
        if group.b.is_none() {
            return;
        }

        // All groups have block B common fields.
        let block_b = GroupType0BlockB::from_bytes(group.b.unwrap().to_be_bytes());
        self.decode_block_b_common(&block_b.common());

        match block_b.common().group_type().code() {
            0 => {
                self.decode_group_type_0(&group);
            }
            1 => {
                self.decode_group_type_1(&group);
            }
            2 => {
                self.decode_group_type_2(&group);
            }
            3 => {
                self.decode_group_type_3(&group);
            }
            4 => {
                self.decode_group_type_4(&group);
            }
            5 => {
                self.decode_group_type_5(&group);
            }
            6 => {
                self.decode_group_type_6(&group);
            }
            7 => {
                self.decode_group_type_7(&group);
            }
            8 => {
                self.decode_group_type_8(&group);
            }
            9 => {
                self.decode_group_type_9(&group);
            }
            10 => {
                self.decode_group_type_10(&group);
            }
            11 => {
                self.decode_group_type_11(&group);
            }
            12 => {
                self.decode_group_type_12(&group);
            }
            13 => {
                self.decode_group_type_13(&group);
            }
            14 => {
                self.decode_group_type_14(&group);
            }
            15 => {
                self.decode_group_type_15(&group);
            }
            _ => {
                // Other group types not implemented yet
            }
        }

        self.callbacks
            .on_oda(0, &self.rds_data, &GroupType::default());
    }
}
