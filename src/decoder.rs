#![allow(dead_code)]

use crate::radiotext::RtVariant;
use crate::rds::RdsData;
use crate::types::{
    Group, GroupType, GroupVersion, NUM_TDC, OdaEntry, ProgramInformation, ProgramType, RdsPic,
    SlcData, TrafficCodes,
};
use heapless::LinearMap;
use modular_bitfield_msb::prelude::*;

/// All type B blocks share the same 11-bit common prefix.
/// See RBDS Standard section 3.1.4.2.
#[bitfield(bits = 11)]
#[derive(BitfieldSpecifier)]
struct BlockBCommon {
    group_type: GroupType,     // Group type (code + version).
    traffic_program: bool,     // TP bit.
    program_type: ProgramType, // PTY: Program type.
}

// See RBDS Standard section 3.1.5.1.
#[bitfield(bits = 16)]
struct GroupType0BlockB {
    common: BlockBCommon,         // Common block B fields.
    traffic_announcement: bool,   // TA bit: section 3.2.1.3.
    ms: bool,                     // M/S bit: section 3.2.1.4.
    decoder_identification: bool, // DI bit: section 3.2.1.5.
    c: B2,                        // Prog. service name and DI segment addr.
}

// See RBDS Standard section 3.1.5.3.
#[bitfield(bits = 16)]
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

pub struct Decoder {
    advanced_ps_decoding: bool,
}

impl<'a> Decoder {
    pub fn new() -> Self {
        Decoder {
            advanced_ps_decoding: true,
        }
    }

    fn decode_block_b_common(&mut self, block: &BlockBCommon, rds_data: &mut RdsData) {
        // TODO: This is only setting the TP bit and not the TA bit.
        // Might have to decouple these if they come from different groups.
        if block.traffic_program() {
            rds_data.traffic = TrafficCodes::TrafficYes;
        } else {
            rds_data.traffic = TrafficCodes::TrafficNoEonNo;
        }
        rds_data.valid.set_tp_code(true);

        rds_data.program_type = block.program_type();
        rds_data.valid.set_pty(true);
    }

    fn decode_alt_freq(&mut self, group: &Group, rds_data: &mut RdsData) {
        if group.c.is_none() {
            return;
        }
        rds_data.valid.set_af(true);
        rds_data
            .alternative_freqs
            .decode_freq_group_block(group.c.unwrap());
    }

    fn decode_ta(&mut self, _group: &Group) {}

    fn decode_ms(&mut self, _group: &Group) {}

    fn update_ps_simple(&mut self, char_idx: u8, current_ps_byte: u8, rds_data: &mut RdsData) {
        assert!(char_idx < 8);
        rds_data.ps.display[char_idx as usize] = current_ps_byte;
        rds_data.valid.set_ps(true);
    }

    /// Update the Program Service text in our buffers from the shadow registers.
    ///
    /// This implementation of the Program Service update attempts to display only
    /// complete messages for stations who rotate text through the PS field in
    /// violation of the RBDS standard as well as providing enhanced error detection.
    ///
    /// This function is from the Silicon Labs sample application.
    fn update_ps_advanced(&mut self, char_idx: usize, byte: u8, rds_data: &mut RdsData) {
        const PS_VALIDATE_LIMIT: u8 = 2;

        let mut in_transition = false; // Indicates if the PS text is in transition.
        let mut complete = true; // Indicates the PS text is ready to be displayed.

        if rds_data.ps.pvt.hi_prob[char_idx] == byte {
            // The new byte matches the high probability byte.
            if rds_data.ps.pvt.hi_prob_cnt[char_idx] < PS_VALIDATE_LIMIT {
                rds_data.ps.pvt.hi_prob_cnt[char_idx] += 1;
            } else {
                // we have received this byte enough to max out our counter and push it
                // into the low probability array as well.
                rds_data.ps.pvt.hi_prob_cnt[char_idx] = PS_VALIDATE_LIMIT;
                rds_data.ps.pvt.lo_prob[char_idx] = byte;
            }
        } else if rds_data.ps.pvt.lo_prob[char_idx] == byte {
            // The new byte is a match with the low probability byte. Swap them, reset
            // the counter and flag the text as in transition. Note that the counter for
            // this character goes higher than the validation limit because it will get
            // knocked down later.
            if rds_data.ps.pvt.hi_prob_cnt[char_idx] >= PS_VALIDATE_LIMIT {
                in_transition = true;
                rds_data.ps.pvt.hi_prob_cnt[char_idx] = PS_VALIDATE_LIMIT + 1;
            } else {
                rds_data.ps.pvt.hi_prob_cnt[char_idx] = PS_VALIDATE_LIMIT;
            }
            rds_data.ps.pvt.lo_prob[char_idx] = rds_data.ps.pvt.hi_prob[char_idx];
            rds_data.ps.pvt.hi_prob[char_idx] = byte;
        } else if rds_data.ps.pvt.hi_prob_cnt[char_idx] == 0 {
            // The new byte is replacing an empty byte in the high probability array.
            rds_data.ps.pvt.hi_prob[char_idx] = byte;
            rds_data.ps.pvt.hi_prob_cnt[char_idx] = 1;
        } else {
            // The new byte doesn't match anything, put it in the low probability array.
            rds_data.ps.pvt.lo_prob[char_idx] = byte;
        }

        if in_transition {
            // When the text is changing, decrement the count for all characters to
            // prevent displaying part of a message that is in transition.
            for count in rds_data.ps.pvt.hi_prob_cnt.iter_mut() {
                if *count > 1 {
                    *count -= 1;
                }
            }
        }

        // The PS text is incomplete if any character in the high probability array
        // has been seen fewer times than the validation limit.
        for count in rds_data.ps.pvt.hi_prob_cnt.iter_mut() {
            if *count < PS_VALIDATE_LIMIT {
                complete = false;
                break;
            }
        }

        // If the PS text in the high probability array is complete copy it to the
        // display array.
        if complete {
            rds_data.valid.set_ps(true);
            rds_data
                .ps
                .display
                .copy_from_slice(&rds_data.ps.pvt.hi_prob);
        }
    }

    fn decode_group_type_0(&mut self, group: &Group, rds_data: &mut RdsData) {
        let block_b = GroupType0BlockB::from_bytes(group.b.unwrap().to_be_bytes());
        if block_b.common().group_type().version() == GroupVersion::A {
            self.decode_alt_freq(group, rds_data);
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
            self.update_ps_advanced((pair_idx + 0) as usize, hi_byte, rds_data);
            self.update_ps_advanced((pair_idx + 1) as usize, lo_byte, rds_data);
        } else {
            self.update_ps_simple(pair_idx + 0, hi_byte, rds_data);
            self.update_ps_simple(pair_idx + 1, lo_byte, rds_data);
        }
    }

    // Type 1 groups: Program Item Number and slow labeling codes
    fn decode_group_type_1(&mut self, group: &Group, rds_data: &mut RdsData) {
        // See RBDS Standard section 3.1.5.2.
        #[bitfield(bits = 16)]
        struct GroupType1BlockB {
            common: BlockBCommon,   // Common block B fields.
            radio_paging_codes: B5, // See Annex M.
        }

        let block_b = GroupType1BlockB::from_bytes(group.b.unwrap().to_be_bytes());
        if block_b.common().group_type().version() == GroupVersion::A && group.c.is_some() {
            rds_data.slc = SlcData::from_bytes(group.c.unwrap().to_be_bytes());
            rds_data.valid.set_slc(true);
        }

        if group.d.is_some() {
            rds_data.program_item_number = RdsPic::from_bytes(group.d.unwrap().to_be_bytes());
            rds_data.valid.set_pic(true);
        } else {
            // Per spec (3.2.1.7): If a type 1 group is transmitted without a
            // valid PIN, the day of the month shall be set to zero. In this
            // case a receiver which evaluates PIN shall ignore the other
            // information in block 4.
            rds_data.program_item_number = RdsPic::default();
        }
    }

    // Type 2 groups: Radiotext.
    fn decode_group_type_2a(&mut self, group: &Group, rds_data: &mut RdsData) {
        // See RBDS Standard setion 3.1.5.3.
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
        let rt = if block_b.text_flag() == RtVariant::A {
            &mut rds_data.rt.a
        } else {
            &mut rds_data.rt.b
        };
        let addr = 4 * block_b.text_segment_addr();
        rt.update_rt_simple(group, 4, addr as usize, &rtchars);
        if rds_data.rt.decode_rt != block_b.text_flag() {
            rt.bump_rt_validation_count();
        }
        rt.update_rt_advance(group, 4, addr as usize, &mut rtchars);
        rds_data.valid.set_rt(true);
        rds_data.rt.decode_rt = block_b.text_flag();
    }

    // Type 2 groups: Radiotext.
    fn decode_group_type_2b(&mut self, group: &Group, rds_data: &mut RdsData) {
        // See RBDS Standard setion 3.1.5.3.
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
        let rt = if block_b.text_flag() == RtVariant::A {
            &mut rds_data.rt.a
        } else {
            &mut rds_data.rt.b
        };
        let addr = 4 * block_b.text_segment_addr();
        rt.update_rt_simple(group, 2, addr as usize, &rtchars);
        if rds_data.rt.decode_rt != block_b.text_flag() {
            rt.bump_rt_validation_count();
        }
        rt.update_rt_advance(group, 2, addr as usize, &mut rtchars);
        rds_data.valid.set_rt(true);
        rds_data.rt.decode_rt = block_b.text_flag();
    }

    fn decode_oda(&mut self, _group: &Group, _rds_data: &mut RdsData) {}

    fn decode_group_type_3a(&mut self, group: &Group, rds_data: &mut RdsData) {
        // See RBDS Standard section 3.1.5.4.
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

        let entry = rds_data.oda.get_mut(&app_id);
        if entry.is_some() {
            let e = entry.unwrap();
            e.group_type = block_b.common().group_type();
        } else {
            if !rds_data.oda.is_full() {
                let _ = rds_data.oda.insert(
                    app_id,
                    OdaEntry {
                        group_type: block_b.common().group_type(),
                        packet_count: 0,
                    },
                );
            }
        }
    }

    fn decode_group_type_3b(&mut self, group: &Group, rds_data: &mut RdsData) {
        // See RBDS Standard section 3.1.5.5.
        self.decode_oda(group, rds_data);
    }

    fn decode_group_type_4a(&mut self, group: &Group, rds_data: &mut RdsData) {
        // See RBDS Standard section 3.1.5.6.
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

        rds_data.clock.mjd = ((block_b.date_msb() as u32) << 15) + block_c.date() as u32;
        rds_data.clock.hour = ((block_c.hour_msb() as u8) << 4) + block_d.hour();
        rds_data.clock.minute = block_d.minute();
        if block_d.local_offset_dir() == 0 {
            rds_data.clock.utc_offset_half_hours = block_d.local_offset_val() as i8;
        } else {
            rds_data.clock.utc_offset_half_hours = -(block_d.local_offset_val() as i8);
        }
    }

    fn decode_group_type_4b(&mut self, group: &Group, rds_data: &mut RdsData) {
        // See RBDS Standard section 3.1.5.7.
        self.decode_oda(group, rds_data);
    }

    fn decode_tdc_block(&mut self, block: u16, rds_data: &mut RdsData) {
        // See RBDS Standard section 4.18.

        let channel = rds_data.tdc.current_channel as usize;
        // `channel` comes from a 5-bit value, so shouldn't be greater than 31.
        assert!(channel < NUM_TDC);

        rds_data.valid.set_tdc(true);
        rds_data.tdc.data[channel].write((block >> 8) as u8);
        rds_data.tdc.data[channel].write((block & 0xff) as u8);
    }

    fn decode_group_type_5a(&mut self, group: &Group, rds_data: &mut RdsData) {
        // See RBDS Standard section 3.1.5.8.
        #[bitfield(bits = 16)]
        struct BlockB {
            common: BlockBCommon, // Common block B fields.
            // Address code identifies "channel number" (out of 32) to which the data are addressed.
            address: B5,
        }
        let block_b = BlockB::from_bytes(group.b.unwrap().to_be_bytes());

        if is_group_type_used(&rds_data.oda, block_b.common().group_type()) {
            self.decode_oda(group, rds_data);
            return;
        }
        rds_data.tdc.current_channel = block_b.address();
        if group.c.is_some() {
            self.decode_tdc_block(group.c.unwrap(), rds_data);
        }
        if group.d.is_some() {
            self.decode_tdc_block(group.d.unwrap(), rds_data);
        }
    }

    fn decode_group_type_5b(&mut self, group: &Group, rds_data: &mut RdsData) {
        // See RBDS Standard section 3.1.5.8.
        const GROUP_TYPE: GroupType = GroupType::from_bytes([5 << 1 + GroupVersion::B as u8]);
        if is_group_type_used(&rds_data.oda, GROUP_TYPE) {
            self.decode_oda(group, rds_data);
            return;
        }
    }

    // Type 6 groups: In-house applications or ODA/
    // See RBDS Standard section 3.1.5.9.
    fn decode_group_type_6(&mut self, group: &Group, rds_data: &mut RdsData) {
        #[bitfield(bits = 16)]
        struct BlockB {
            common: BlockBCommon, // Common block B fields.
            unused: B5,
        }
        let block_b = BlockB::from_bytes(group.b.unwrap().to_be_bytes());
        if is_group_type_used(&rds_data.oda, block_b.common().group_type()) {
            self.decode_oda(group, rds_data);
            return;
        }

        // According to RBDS spec.: "Consumer receivers should ignore the in-house
        // information coded in these groups".
    }

    // Type 7A groups: Radio Paging or ODA
    fn decode_group_type_7a(&mut self, group: &Group, rds_data: &mut RdsData) {
        // See RBDS Standard section 3.1.5.10.
        const GROUP_TYPE: GroupType = GroupType::from_bytes([7 << 1 + GroupVersion::A as u8]);
        if is_group_type_used(&rds_data.oda, GROUP_TYPE) {
            self.decode_oda(group, rds_data);
            return;
        }

        // No stations seem to broadcast this data. Will implement if/when needed.
    }

    // Type 7B groups: Open data application
    fn decode_group_type_7b(&mut self, group: &Group, rds_data: &mut RdsData) {
        // See RBDS Standard section 3.1.5.11.
        self.decode_oda(group, rds_data);
    }

    // Type 8 groups: Traffic Message Channel or ODA
    fn decode_group_type_8(&mut self, group: &Group, rds_data: &mut RdsData) {
        // See RBDS Standard section 3.1.5.12.
        let gt = group.get_type();
        if is_group_type_used(&rds_data.oda, gt) {
            self.decode_oda(group, rds_data);
            return;
        }
        if gt.version() == GroupVersion::A {
            // Decode TMC data. This requires obtaining a copy of EN ISO 14819-1:2013.
        }
    }

    // Type 9 groups: Emergency warning systems or ODA.
    fn decode_group_type_9(&mut self, group: &Group, rds_data: &mut RdsData) {
        // See RBDS Standard section 3.1.5.13.
        let gt = group.get_type();
        if is_group_type_used(&rds_data.oda, gt) {
            self.decode_oda(group, rds_data);
            return;
        }

        if gt.version() == GroupVersion::B {
            return;
        }

        if group.c.is_none() || group.d.is_none() {
            return;
        }

        rds_data.valid.set_ews(true);
        rds_data
            .ews
            .set_block_b_lsb((group.b.unwrap() & 0b11111) as u8);
        rds_data.ews.set_block_c(group.c.unwrap());
        rds_data.ews.set_block_d(group.d.unwrap());
    }

    fn decode_ptyn(&mut self, group: &Group, rds_data: &mut RdsData) {
        // See RBDS Standard section 3.1.5.14.
        #[bitfield(bits = 16)]
        struct BlockB {
            common: BlockBCommon, // Common block B fields.
            ab_flag: bool,
            reserved: B3,
            segment_addr: B1,
        }
        let block_b = BlockB::from_bytes(group.b.unwrap().to_be_bytes());

        rds_data.valid.set_ptyn(true);
        if rds_data.ptyn.last_ab != block_b.ab_flag() {
            rds_data.ptyn.display.fill(0);
            rds_data.ptyn.last_ab = block_b.ab_flag();
        }

        let base: usize = 4 * block_b.segment_addr() as usize;
        if group.c.is_some() {
            rds_data.ptyn.display[base + 0] = (group.c.unwrap() >> 8) as u8;
            rds_data.ptyn.display[base + 1] = (group.c.unwrap() & 0xff) as u8;
        }
        if group.d.is_some() {
            rds_data.ptyn.display[base + 2] = (group.d.unwrap() >> 8) as u8;
            rds_data.ptyn.display[base + 3] = (group.d.unwrap() & 0xff) as u8;
        }
    }

    // Type 10 groups: Program Type Name.
    fn decode_group_type_10a(&mut self, group: &Group, rds_data: &mut RdsData) {
        // See RBDS Standard section 3.1.5.14.
        self.decode_ptyn(group, rds_data);
    }

    // Type 10 groups: Open data.
    fn decode_group_type_10b(&mut self, group: &Group, rds_data: &mut RdsData) {
        // See RBDS Standard section 3.1.5.14.
        if group.get_type().version() == GroupVersion::A {
            self.decode_ptyn(group, rds_data);
        } else {
            self.decode_oda(group, rds_data);
        }
    }

    // Type 11 groups: Open Data Application
    fn decode_group_type_11(&mut self, group: &Group, rds_data: &mut RdsData) {
        // See RBDS Standard section 3.1.5.15.
        self.decode_oda(group, rds_data);
    }

    fn decode_group_type_12(&mut self, group: &Group, rds_data: &mut RdsData) {
        // See RBDS Standard section 3.1.5.16.
        self.decode_oda(group, rds_data);
    }

    // Type 13A groups: Enhanced Radio Paging or ODA.
    fn decode_group_type_13a(&mut self, group: &Group, _rds_data: &mut RdsData) {
        // See RBDS Standard section 3.1.5.17.
        #[bitfield(bits = 16)]
        struct BlockB {
            common: BlockBCommon, // Common block B fields.
            information: B2,
            sty: B3,
        }
        let _block_b = BlockB::from_bytes(group.b.unwrap().to_be_bytes());

        // The type 13A group may be used for ODA when it is not used for Radio
        // Paging, and its group structure is then as shown in 3.1.4.2

        // TODO: How to determine if this is used for radio paging???
    }

    // Type 13B groups: Open Data Application
    fn decode_group_type_13b(&mut self, group: &Group, rds_data: &mut RdsData) {
        // See RBDS Standard section 3.1.5.18.
        self.decode_oda(group, rds_data);
    }

    // Type 14 groups: Enhanced Other Networks information.
    fn decode_group_type_14a(&mut self, group: &Group, _rds_data: &mut RdsData) {
        // See RBDS Standard section 3.1.5.19.
        #[bitfield(bits = 16)]
        struct BlockB {
            common: BlockBCommon, // Common block B fields.
            tp_on: bool,          // TP (ON).
            variant_code: B4,
        }
        let _block_b = BlockB::from_bytes(group.b.unwrap().to_be_bytes());
        // TODO: finish me.
    }

    // Type 14 groups: Enhanced Other Networks information.
    fn decode_group_type_14b(&mut self, group: &Group, _rds_data: &mut RdsData) {
        // See RBDS Standard section 3.1.5.19.
        #[bitfield(bits = 16)]
        struct BlockB {
            common: BlockBCommon, // Common block B fields.
            tp_on: bool,          // TP (ON).
            ta_on: bool,          // TA (ON).
            unused: B3,
        }
        let _block_b = BlockB::from_bytes(group.b.unwrap().to_be_bytes());
        // TODO: finish me.
    }

    fn decode_group_type_15(&mut self, _group: &Group, _rds_data: &RdsData) {}

    pub fn decode(&mut self, group: &Group, rds_data: &mut RdsData) {
        if group.a.is_some() {
            rds_data.program_information =
                ProgramInformation::from_bytes(group.a.unwrap().to_be_bytes());
        }
        if group.b.is_none() {
            return;
        }

        // All groups have block B common fields.
        let block_b = GroupType0BlockB::from_bytes(group.b.unwrap().to_be_bytes());
        self.decode_block_b_common(&block_b.common(), rds_data);

        match (
            block_b.common().group_type().code(),
            block_b.common().group_type().version(),
        ) {
            (0, GroupVersion::A) | (0, GroupVersion::B) => {
                self.decode_group_type_0(&group, rds_data);
            }
            (1, GroupVersion::A) | (1, GroupVersion::B) => {
                self.decode_group_type_1(&group, rds_data);
            }
            (2, GroupVersion::A) => {
                self.decode_group_type_2a(&group, rds_data);
            }
            (2, GroupVersion::B) => {
                self.decode_group_type_2b(&group, rds_data);
            }
            (3, GroupVersion::A) => {
                self.decode_group_type_3a(&group, rds_data);
            }
            (3, GroupVersion::B) => {
                self.decode_group_type_3b(&group, rds_data);
            }
            (4, GroupVersion::A) => {
                self.decode_group_type_4a(&group, rds_data);
            }
            (4, GroupVersion::B) => {
                self.decode_group_type_4b(&group, rds_data);
            }
            (5, GroupVersion::A) => {
                self.decode_group_type_5a(&group, rds_data);
            }
            (5, GroupVersion::B) => {
                self.decode_group_type_5b(&group, rds_data);
            }
            (6, GroupVersion::A) | (6, GroupVersion::B) => {
                self.decode_group_type_6(&group, rds_data);
            }
            (7, GroupVersion::A) => {
                self.decode_group_type_7a(&group, rds_data);
            }
            (7, GroupVersion::B) => {
                self.decode_group_type_7b(&group, rds_data);
            }
            (8, GroupVersion::A) | (8, GroupVersion::B) => {
                self.decode_group_type_8(&group, rds_data);
            }
            (9, GroupVersion::A) | (9, GroupVersion::B) => {
                self.decode_group_type_9(&group, rds_data);
            }
            (10, GroupVersion::A) => {
                self.decode_group_type_10a(&group, rds_data);
            }
            (10, GroupVersion::B) => {
                self.decode_group_type_10b(&group, rds_data);
            }
            (11, GroupVersion::A) | (11, GroupVersion::B) => {
                self.decode_group_type_11(&group, rds_data);
            }
            (12, GroupVersion::A) | (12, GroupVersion::B) => {
                self.decode_group_type_12(&group, rds_data);
            }
            (13, GroupVersion::A) => {
                self.decode_group_type_13a(&group, rds_data);
            }
            (13, GroupVersion::B) => {
                self.decode_group_type_13b(&group, rds_data);
            }
            (14, GroupVersion::A) => {
                self.decode_group_type_14a(&group, rds_data);
            }
            (14, GroupVersion::B) => {
                self.decode_group_type_14b(&group, rds_data);
            }
            (15, GroupVersion::A) | (15, GroupVersion::B) => {
                self.decode_group_type_15(&group, rds_data);
            }
            _ => {
                // Other group types not implemented yet
            }
        }
    }
}
