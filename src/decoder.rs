#![allow(dead_code)]

use crate::callbacks::{RdsData, RdsDecoderCallbacks};
use crate::radiotext::RtVariant;
use crate::types::{
    Group, GroupType, GroupVersion, NUM_TDC, OdaEntry, ProgramInformation, ProgramType, RdsPic,
    SlcData, TrafficCodes,
};
use heapless::LinearMap;
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

/// Is the ODA application ID valid?
fn is_valid_oda_app_id(app_id: u16) -> bool {
    return app_id != 0x0;
}

fn is_group_type_used(map: &LinearMap<u16, OdaEntry, 10>, gt: GroupType) -> bool {
    for (_key, val) in map.iter() {
        if val.group_type == gt {
            return true;
        }
    }
    return false;
}

impl Group {
    fn get_type(&self) -> GroupType {
        GroupType0BlockB::from_bytes(self.b.unwrap().to_be_bytes())
            .common()
            .group_type()
    }
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

    fn decode_group_type_2a(&mut self, group: &Group) {
        // See specification setion 3.1.5.3.
        let block_b = GroupType2BlockB::from_bytes(group.b.unwrap().to_be_bytes());
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
    }

    fn decode_group_type_2b(&mut self, group: &Group) {
        // See specification setion 3.1.5.3.
        let block_b = GroupType2BlockB::from_bytes(group.b.unwrap().to_be_bytes());
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
    }

    fn decode_oda(&mut self, _group: &Group) {}

    fn decode_group_type_3a(&mut self, group: &Group) {
        // See RDS Standard section 3.1.5.4.
        #[bitfield(bits = 16)]
        #[derive(Default, Clone, PartialEq, Eq)]
        struct GroupType3ABlockB {
            common: BlockBCommon,         // Common block B fields.
            application_group: GroupType, // See Annex M.
        }

        if group.d.is_none() {
            return;
        }
        let block_b = GroupType3ABlockB::from_bytes(group.b.unwrap().to_be_bytes());
        let app_id = group.d.unwrap();
        if !is_valid_oda_app_id(app_id) {
            return;
        }

        let entry = self.rds_data.oda.get_mut(&app_id);
        if entry.is_some() {
            let e = entry.unwrap();
            e.group_type = block_b.common().group_type();
        } else {
            if !self.rds_data.oda.is_full() {
                let _ = self.rds_data.oda.insert(
                    app_id,
                    OdaEntry {
                        group_type: block_b.common().group_type(),
                        packet_count: 0,
                    },
                );
            }
        }
    }

    fn decode_group_type_3b(&mut self, group: &Group) {
        // See RDS Standard section 3.1.5.5.
        self.decode_oda(group);
    }

    fn decode_group_type_4a(&mut self, group: &Group) {
        // See RDS Standard section 3.1.5.6.
        #[bitfield(bits = 16)]
        struct BlockB {
            common: BlockBCommon, // Common block B fields.
            spare: B3,            // Unused.
            date_msb: B2,         // Top two MSB bits of julian date.
        }
        #[bitfield(bits = 16)]
        struct BlockC {
            date: B15,
            hour_msb: B1,
        }
        #[bitfield(bits = 16)]
        struct BlockD {
            hour: B4,
            minute: B6,
            local_offset_dir: B1, // Offset direction from UTC: 0=+, 1=-;
            local_offset_val: B5, // Offset in half-hour increments
        }

        if group.c.is_none() || group.d.is_none() {
            return;
        }
        let block_b = BlockB::from_bytes(group.b.unwrap().to_be_bytes());
        let block_c = BlockC::from_bytes(group.b.unwrap().to_be_bytes());
        let block_d = BlockD::from_bytes(group.b.unwrap().to_be_bytes());

        self.rds_data.clock.mjd = ((block_b.date_msb() as u32) << 15) + block_c.date() as u32;
        self.rds_data.clock.hour = ((block_c.hour_msb() as u8) << 4) + block_d.hour();
        self.rds_data.clock.minute = block_d.minute();
        if block_d.local_offset_dir() == 0 {
            self.rds_data.clock.utc_offset_half_hours = block_d.local_offset_val() as i8;
        } else {
            self.rds_data.clock.utc_offset_half_hours = -(block_d.local_offset_val() as i8);
        }
    }

    fn decode_group_type_4b(&mut self, group: &Group) {
        // See RDS Standard section 3.1.5.7.
        self.decode_oda(group);
    }

    fn decode_tdc_block(&mut self, block: u16) {
        // See RDS Standard section 4.18.

        let channel = self.rds_data.tdc.current_channel as usize;
        // `channel` comes from a 5-bit value, so shouldn't be greater than 31.
        assert!(channel < NUM_TDC);

        self.rds_data.valid.set_tdc(true);
        self.rds_data.tdc.data[channel].write((block >> 8) as u8);
        self.rds_data.tdc.data[channel].write((block & 0xff) as u8);
    }

    fn decode_group_type_5a(&mut self, group: &Group) {
        // See RDS Standard section 3.1.5.8.
        #[bitfield(bits = 16)]
        struct BlockB {
            common: BlockBCommon, // Common block B fields.
            // Address code identifies "channel number" (out of 32) to which the data are addressed.
            address: B5,
        }
        let block_b = BlockB::from_bytes(group.b.unwrap().to_be_bytes());

        if is_group_type_used(&self.rds_data.oda, block_b.common().group_type()) {
            self.decode_oda(group);
            return;
        }
        self.rds_data.tdc.current_channel = block_b.address();
        if group.c.is_some() {
            self.decode_tdc_block(group.c.unwrap());
        }
        if group.d.is_some() {
            self.decode_tdc_block(group.d.unwrap());
        }
    }

    fn decode_group_type_5b(&mut self, group: &Group) {
        // See RDS Standard section 3.1.5.8.
        const GROUP_TYPE: GroupType = GroupType::from_bytes([5 << 1 + GroupVersion::B as u8]);
        if is_group_type_used(&self.rds_data.oda, GROUP_TYPE) {
            self.decode_oda(group);
            return;
        }
    }

    // Type 6 groups: In-house applications or ODA/
    // See RDS Standard section 3.1.5.9.
    fn decode_group_type_6(&mut self, group: &Group) {
        #[bitfield(bits = 16)]
        struct BlockB {
            common: BlockBCommon, // Common block B fields.
            unused: B5,
        }
        let block_b = BlockB::from_bytes(group.b.unwrap().to_be_bytes());
        if is_group_type_used(&self.rds_data.oda, block_b.common().group_type()) {
            self.decode_oda(group);
            return;
        }

        // According to RBDS spec.: "Consumer receivers should ignore the in-house
        // information coded in these groups".
    }

    // Type 7A groups: Radio Paging or ODA
    fn decode_group_type_7a(&mut self, group: &Group) {
        // See RDS Standard section 3.1.5.10.
        const GROUP_TYPE: GroupType = GroupType::from_bytes([7 << 1 + GroupVersion::A as u8]);
        if is_group_type_used(&self.rds_data.oda, GROUP_TYPE) {
            self.decode_oda(group);
            return;
        }

        // No stations seem to broadcast this data. Will implement if/when needed.
    }

    // Type 7B groups: Open data application
    fn decode_group_type_7b(&mut self, group: &Group) {
        // See RDS Standard section 3.1.5.11.
        self.decode_oda(group);
    }

    // Type 8 groups: Traffic Message Channel or ODA
    fn decode_group_type_8(&mut self, group: &Group) {
        // See RDS Standard section 3.1.5.12.
        let gt = group.get_type();
        if is_group_type_used(&self.rds_data.oda, gt) {
            self.decode_oda(group);
            return;
        }
        if gt.version() == GroupVersion::A {
            // Decode TMC data. This requires obtaining a copy of EN ISO 14819-1:2013.
        }
    }

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

        match (
            block_b.common().group_type().code(),
            block_b.common().group_type().version(),
        ) {
            (0, GroupVersion::A) | (0, GroupVersion::B) => {
                self.decode_group_type_0(&group);
            }
            (1, GroupVersion::A) | (1, GroupVersion::B) => {
                self.decode_group_type_1(&group);
            }
            (2, GroupVersion::A) => {
                self.decode_group_type_2a(&group);
            }
            (2, GroupVersion::B) => {
                self.decode_group_type_2b(&group);
            }
            (3, GroupVersion::A) => {
                self.decode_group_type_3a(&group);
            }
            (3, GroupVersion::B) => {
                self.decode_group_type_3b(&group);
            }
            (4, GroupVersion::A) => {
                self.decode_group_type_4a(&group);
            }
            (4, GroupVersion::B) => {
                self.decode_group_type_4b(&group);
            }
            (5, GroupVersion::A) => {
                self.decode_group_type_5a(&group);
            }
            (5, GroupVersion::B) => {
                self.decode_group_type_5b(&group);
            }
            (6, GroupVersion::A) | (6, GroupVersion::B) => {
                self.decode_group_type_6(&group);
            }
            (7, GroupVersion::A) => {
                self.decode_group_type_7a(&group);
            }
            (7, GroupVersion::B) => {
                self.decode_group_type_7b(&group);
            }
            (8, GroupVersion::A) | (8, GroupVersion::B) => {
                self.decode_group_type_8(&group);
            }
            (9, GroupVersion::A) | (9, GroupVersion::B) => {
                self.decode_group_type_9(&group);
            }
            (10, GroupVersion::A) | (10, GroupVersion::B) => {
                self.decode_group_type_10(&group);
            }
            (11, GroupVersion::A) | (11, GroupVersion::B) => {
                self.decode_group_type_11(&group);
            }
            (12, GroupVersion::A) | (12, GroupVersion::B) => {
                self.decode_group_type_12(&group);
            }
            (13, GroupVersion::A) | (13, GroupVersion::B) => {
                self.decode_group_type_13(&group);
            }
            (14, GroupVersion::A) | (14, GroupVersion::B) => {
                self.decode_group_type_14(&group);
            }
            (15, GroupVersion::A) | (15, GroupVersion::B) => {
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
