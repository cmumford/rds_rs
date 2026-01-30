#![allow(dead_code)]

use crate::radiotext::{BLANK_CHAR, RtVariant};
use crate::rds::RdsData;
use crate::types::{
    Content, Group, GroupType, GroupVersion, NUM_TDC, OdaEntry, ProgramInformation, ProgramType,
    RdsPic, SlcData, ValidFields,
};
use heapless::LinearMap;
use modular_bitfield_msb::prelude::*;
use std::ops::BitOr;

const INVALID_ODA_APP_ID: u16 = 0x0;

// See RBDS Standard section 3.1.5.3.
#[bitfield(bits = 16)]
struct GroupType2BlockB {
    group_type: GroupType,     // Group type (code + version).
    traffic_program: bool,     // TP bit.
    program_type: ProgramType, // PTY: Program type.
    text_flag: RtVariant,
    text_segment_addr: B4,
}

impl BitOr for ValidFields {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        const N: usize = std::mem::size_of::<ValidFields>();
        let l = self.into_bytes();
        let r = rhs.into_bytes();
        let mut m = [0u8; N];
        for i in 0..N {
            m[i] = l[i] | r[i];
        }
        Self::from_bytes(m)
    }
}

/// Is the ODA application ID valid?
fn is_valid_oda_app_id(app_id: u16) -> bool {
    return app_id != INVALID_ODA_APP_ID;
}

fn is_oda_group_type_used(map: &LinearMap<u16, OdaEntry, 10>, gt: GroupType) -> bool {
    for (_key, val) in map.iter() {
        if val.group_type == gt {
            return true;
        }
    }
    return false;
}

impl Group {
    fn get_type(&self) -> GroupType {
        GroupType2BlockB::from_bytes(self.b.unwrap().to_be_bytes()).group_type()
    }
}

fn decode_ms(blockb: u16, rds_data: &mut RdsData) -> ValidFields {
    #[bitfield(bits = 16)]
    struct Block {
        group_type: GroupType,     // Group type (code + version).
        traffic_program: bool,     // TP bit.
        program_type: ProgramType, // PTY: Program type.
        ta: bool,
        content: Content,
        unused: B3,
    }
    let block_b = Block::from_bytes(blockb.to_be_bytes());
    rds_data.content = block_b.content();
    ValidFields::new().with_ms(true)
}

fn decode_block_b_common(block: &GroupType2BlockB, rds_data: &mut RdsData) -> ValidFields {
    let mut valid = ValidFields::new();
    rds_data.traffic.set_tp(block.traffic_program());
    valid.set_tp_code(true);

    rds_data.program_type = block.program_type();
    valid.set_pty(true);
    valid
}

fn decode_alt_freq(group: &Group, rds_data: &mut RdsData) -> ValidFields {
    if group.c.is_none() {
        return ValidFields::new();
    }
    rds_data
        .alternative_freqs
        .decode_freq_group_block(group.c.unwrap());
    ValidFields::new().with_af(true)
}

fn decode_ta(blockb: u16, rds_data: &mut RdsData) -> ValidFields {
    #[bitfield(bits = 16)]
    struct Block {
        group_type: GroupType,     // Group type (code + version).
        traffic_program: bool,     // TP bit.
        program_type: ProgramType, // PTY: Program type.
        ta_flag: bool,
        unused: B4,
    }
    let block_b = Block::from_bytes(blockb.to_be_bytes());
    rds_data.traffic.set_ta(block_b.ta_flag());
    ValidFields::new().with_ta_code(true)
}

fn update_ps_simple(char_idx: u8, current_ps_byte: u8, rds_data: &mut RdsData) {
    assert!(char_idx < 8);
    rds_data.ps.display[char_idx as usize] = current_ps_byte;
}

/// Update the Program Service text in our buffers from the shadow registers.
///
/// This implementation of the Program Service update attempts to display only
/// complete messages for stations who rotate text through the PS field in
/// violation of the RBDS standard as well as providing enhanced error detection.
///
/// This function is from the Silicon Labs sample application.
fn update_ps_advanced(char_idx: usize, byte: u8, rds_data: &mut RdsData) -> bool {
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
        rds_data
            .ps
            .display
            .copy_from_slice(&rds_data.ps.pvt.hi_prob);
    }
    complete
}

// Type 0 groups: Basic tuning and switching information.
fn decode_group_type_0(
    group: &Group,
    rds_data: &mut RdsData,
    advanced_ps_decoding: bool,
) -> ValidFields {
    // See RBDS Standard section 3.1.5.1.
    #[bitfield(bits = 16)]
    struct BlockB {
        group_type: GroupType,        // Group type (code + version).
        traffic_program: bool,        // TP bit.
        program_type: ProgramType,    // PTY: Program type.
        traffic_announcement: bool,   // TA bit: section 3.2.1.3.
        ms: bool,                     // M/S bit: section 3.2.1.4.
        decoder_identification: bool, // DI bit: section 3.2.1.5.
        c: B2,                        // Prog. service name and DI segment addr.
    }

    let mut valid = ValidFields::new();
    let block_b = BlockB::from_bytes(group.b.unwrap().to_be_bytes());
    if block_b.group_type().version() == GroupVersion::A {
        valid = valid | decode_alt_freq(group, rds_data);
    }
    if group.d.is_none() {
        return valid;
    }
    valid = valid | decode_ta(group.b.unwrap(), rds_data);
    valid = valid | decode_ms(group.b.unwrap(), rds_data);

    let pair_idx = 2 * block_b.c();
    let d_val = group.d.unwrap();
    let hi_byte = (d_val >> 8) as u8;
    let lo_byte = (d_val & 0xFF) as u8;
    if advanced_ps_decoding {
        let mut updated = update_ps_advanced((pair_idx + 0) as usize, hi_byte, rds_data);
        if updated {
            valid.set_ps(true);
        }
        updated = update_ps_advanced((pair_idx + 1) as usize, lo_byte, rds_data);
        if updated {
            valid.set_ps(true);
        }
    } else {
        update_ps_simple(pair_idx + 0, hi_byte, rds_data);
        update_ps_simple(pair_idx + 1, lo_byte, rds_data);
        valid.set_ps(true);
    }
    valid
}

// Type 1 groups: Program Item Number and slow labeling codes.
fn decode_group_type_1(group: &Group, rds_data: &mut RdsData) -> ValidFields {
    // See RBDS Standard section 3.1.5.2.
    #[bitfield(bits = 16)]
    struct GroupType1BlockB {
        group_type: GroupType,     // Group type (code + version).
        traffic_program: bool,     // TP bit.
        program_type: ProgramType, // PTY: Program type.
        radio_paging_codes: B5,    // See Annex M.
    }

    let mut valid = ValidFields::new();
    let block_b = GroupType1BlockB::from_bytes(group.b.unwrap().to_be_bytes());
    if block_b.group_type().version() == GroupVersion::A && group.c.is_some() {
        rds_data.slc = SlcData::from_bytes(group.c.unwrap().to_be_bytes());
        valid.set_slc(true);
    }

    // Per spec (3.2.1.7): If a type 1 group is transmitted without a
    // valid PIN, the day of the month shall be set to zero. In this
    // case a receiver which evaluates PIN shall ignore the other
    // information in block 4.
    rds_data.program_item_number = group
        .d
        .map(|d| RdsPic::from_bytes(d.to_be_bytes()))
        .unwrap_or_default();
    if group.d.is_some() {
        valid.set_pic(true);
    }
    valid
}

// Type 2 groups: Radiotext.
// See RBDS Standard setion 3.1.5.3.
fn decode_group_type_2a(group: &Group, rds_data: &mut RdsData) -> ValidFields {
    let block_b = GroupType2BlockB::from_bytes(group.b.unwrap().to_be_bytes());
    let chars: [Option<[u8; 2]>; 2] = [
        group.c.map(|c| c.to_be_bytes()),
        group.d.map(|d| d.to_be_bytes()),
    ];
    let rt = match block_b.text_flag() {
        RtVariant::A => &mut rds_data.rt.a,
        RtVariant::B => &mut rds_data.rt.b,
    };
    let addr = (block_b.text_segment_addr() as usize) * 4;
    rt.update_rt_simple(addr, &chars);
    if rds_data.rt.decode_rt != block_b.text_flag() {
        rt.bump_rt_validation_count();
    }
    rt.update_rt_advance(addr, &chars);
    rds_data.rt.decode_rt = block_b.text_flag();
    ValidFields::new().with_rt(true)
}

// Type 2 groups: Radiotext.
// See RBDS Standard setion 3.1.5.3.
fn decode_group_type_2b(group: &Group, rds_data: &mut RdsData) -> ValidFields {
    if group.d.is_none() {
        return ValidFields::new();
    }
    let block_b = GroupType2BlockB::from_bytes(group.b.unwrap().to_be_bytes());
    let chars: [Option<[u8; 2]>; 1] = [group.d.map(|d| d.to_be_bytes())];
    let rt = match block_b.text_flag() {
        RtVariant::A => &mut rds_data.rt.a,
        RtVariant::B => &mut rds_data.rt.b,
    };
    let addr = (block_b.text_segment_addr() as usize) * 2;
    rt.update_rt_simple(addr, &chars);
    if rds_data.rt.decode_rt != block_b.text_flag() {
        rt.bump_rt_validation_count();
    }
    rt.update_rt_advance(addr, &chars);
    rds_data.rt.decode_rt = block_b.text_flag();
    ValidFields::new().with_rt(true)
}

fn decode_oda(_group: &Group, gt: GroupType, rds_data: &mut RdsData) -> ValidFields {
    let mut app_id: u16 = INVALID_ODA_APP_ID;
    for (key, val) in rds_data.oda.iter() {
        if val.group_type == gt {
            app_id = *key;
            break;
        }
    }
    if app_id == INVALID_ODA_APP_ID {
        return ValidFields::new();
    }
    // TODO: Finish this. Either use callback, or another way for caller to know new ODA has arrived.
    ValidFields::new()
}

// Type 3A groups: Application identification for Open data.
fn decode_group_type_3a(group: &Group, rds_data: &mut RdsData) -> ValidFields {
    let valid = ValidFields::new();
    // See RBDS Standard section 3.1.5.4.
    #[bitfield(bits = 16)]
    #[derive(Default, Clone, PartialEq, Eq)]
    struct GroupType3ABlockB {
        group_type: GroupType,        // Group type (code + version).
        traffic_program: bool,        // TP bit.
        program_type: ProgramType,    // PTY: Program type.
        application_group: GroupType, // See Annex M.
    }

    if group.d.is_none() {
        return valid;
    }
    let block_b = GroupType3ABlockB::from_bytes(group.b.unwrap().to_be_bytes());
    let app_id = group.d.unwrap();

    // Per spec 3.1.5.4:
    // > The AID code 0000 (Hex) may be used to indicate that the respective
    // > group type is being used for the normal feature specified in this
    // > standard. Application Identification codes 0001 to FFFF (Hex) indicate
    // > applications as specified in the ODA Directory
    if !is_valid_oda_app_id(app_id) {
        return valid;
    }

    let entry = rds_data.oda.get_mut(&app_id);
    if entry.is_some() {
        let e = entry.unwrap();
        e.group_type = block_b.group_type();
    } else {
        if !rds_data.oda.is_full() {
            let _ = rds_data.oda.insert(
                app_id,
                OdaEntry {
                    group_type: block_b.group_type(),
                    packet_count: 0,
                },
            );
        }
    }
    valid
}

// Type 3B groups: Open Data Application.
fn decode_group_type_3b(group: &Group, rds_data: &mut RdsData) -> ValidFields {
    // See RBDS Standard section 3.1.5.5.
    decode_oda(group, group.get_type(), rds_data)
}

// Type 4A groups : Clock-time and date.
fn decode_group_type_4a(group: &Group, rds_data: &mut RdsData) -> ValidFields {
    // See RBDS Standard section 3.1.5.6.
    #[bitfield(bits = 16)]
    struct BlockB {
        group_type: GroupType,     // Group type (code + version).
        traffic_program: bool,     // TP bit.
        program_type: ProgramType, // PTY: Program type.
        spare: B3,                 // Unused.
        date_msb: B2,              // Top two MSB bits of julian date.
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
        return ValidFields::new();
    }
    let block_b = BlockB::from_bytes(group.b.unwrap().to_be_bytes());
    let block_c = BlockC::from_bytes(group.b.unwrap().to_be_bytes());
    let block_d = BlockD::from_bytes(group.b.unwrap().to_be_bytes());

    rds_data.clock.mjd = ((block_b.date_msb() as u32) << 15) + block_c.date() as u32;
    rds_data.clock.hour = ((block_c.hour_msb() as u8) << 4) + block_d.hour();
    rds_data.clock.minute = block_d.minute();
    rds_data.clock.utc_offset_half_hours = if block_d.local_offset_dir() == 0 {
        block_d.local_offset_val() as i8
    } else {
        -(block_d.local_offset_val() as i8)
    };
    ValidFields::new().with_clock(true)
}

// Type 4B groups: Open data application.
fn decode_group_type_4b(group: &Group, rds_data: &mut RdsData) -> ValidFields {
    // See RBDS Standard section 3.1.5.7.
    decode_oda(group, group.get_type(), rds_data)
}

fn decode_tdc_block(block: u16, rds_data: &mut RdsData) {
    // See RBDS Standard section 4.18.

    let channel = rds_data.tdc.current_channel as usize;
    // `channel` comes from a 5-bit value, so shouldn't be greater than 31.
    assert!(channel < NUM_TDC);

    rds_data.tdc.data[channel].write((block >> 8) as u8);
    rds_data.tdc.data[channel].write((block & 0xff) as u8);
}

// Type 5 groups: Transparent data channels or ODA.
fn decode_group_type_5a(group: &Group, rds_data: &mut RdsData) -> ValidFields {
    // See RBDS Standard section 3.1.5.8.
    #[bitfield(bits = 16)]
    struct BlockB {
        group_type: GroupType,     // Group type (code + version).
        traffic_program: bool,     // TP bit.
        program_type: ProgramType, // PTY: Program type.
        // Address code identifies "channel number" (out of 32) to which the data are addressed.
        address: B5,
    }
    let block_b = BlockB::from_bytes(group.b.unwrap().to_be_bytes());

    if is_oda_group_type_used(&rds_data.oda, block_b.group_type()) {
        return decode_oda(group, block_b.group_type(), rds_data);
    }
    let mut valid = ValidFields::new();
    rds_data.tdc.current_channel = block_b.address();
    if group.c.is_some() {
        decode_tdc_block(group.c.unwrap(), rds_data);
        valid.set_tdc(true);
    }
    if group.d.is_some() {
        decode_tdc_block(group.d.unwrap(), rds_data);
        valid.set_tdc(true);
    }
    valid
}

// Type 5 groups: ODA.
fn decode_group_type_5b(group: &Group, rds_data: &mut RdsData) -> ValidFields {
    // See RBDS Standard section 3.1.5.8.
    const GROUP_TYPE: GroupType = GroupType::from_bytes([5 << 1 + GroupVersion::B as u8]);
    if is_oda_group_type_used(&rds_data.oda, GROUP_TYPE) {
        return decode_oda(group, GROUP_TYPE, rds_data);
    }
    ValidFields::new()
}

// Type 6 groups: In-house applications or ODA/
// See RBDS Standard section 3.1.5.9.
fn decode_group_type_6(group: &Group, rds_data: &mut RdsData) -> ValidFields {
    #[bitfield(bits = 16)]
    struct BlockB {
        group_type: GroupType,     // Group type (code + version).
        traffic_program: bool,     // TP bit.
        program_type: ProgramType, // PTY: Program type.
        unused: B5,
    }
    let block_b = BlockB::from_bytes(group.b.unwrap().to_be_bytes());
    if is_oda_group_type_used(&rds_data.oda, block_b.group_type()) {
        return decode_oda(group, block_b.group_type(), rds_data);
    }

    // According to RBDS spec.: "Consumer receivers should ignore the in-house
    // information coded in these groups".
    ValidFields::new()
}

// Type 7A groups: Radio Paging or ODA.
fn decode_group_type_7a(group: &Group, rds_data: &mut RdsData) -> ValidFields {
    // See RBDS Standard section 3.1.5.10.
    const GROUP_TYPE: GroupType = GroupType::from_bytes([7 << 1 + GroupVersion::A as u8]);
    if is_oda_group_type_used(&rds_data.oda, GROUP_TYPE) {
        return decode_oda(group, GROUP_TYPE, rds_data);
    }

    // No stations seem to broadcast this data. Will implement if/when needed.
    ValidFields::new()
}

// Type 7B groups: Open data application.
fn decode_group_type_7b(group: &Group, rds_data: &mut RdsData) -> ValidFields {
    // See RBDS Standard section 3.1.5.11.
    decode_oda(group, group.get_type(), rds_data)
}

// Type 8 groups: Traffic Message Channel or ODA
fn decode_group_type_8(group: &Group, rds_data: &mut RdsData) -> ValidFields {
    // See RBDS Standard section 3.1.5.12.
    let gt = group.get_type();
    if is_oda_group_type_used(&rds_data.oda, gt) {
        return decode_oda(group, gt, rds_data);
    }
    if gt.version() == GroupVersion::A {
        // Decode TMC data. This requires obtaining a copy of EN ISO 14819-1:2013.
    }
    ValidFields::new()
}

// Type 9 groups: Emergency warning systems or ODA.
fn decode_group_type_9(group: &Group, rds_data: &mut RdsData) -> ValidFields {
    // See RBDS Standard section 3.1.5.13.
    let gt = group.get_type();
    if is_oda_group_type_used(&rds_data.oda, gt) {
        return decode_oda(group, gt, rds_data);
    }

    let mut valid = ValidFields::new();
    if gt.version() == GroupVersion::B {
        return valid;
    }

    if group.c.is_none() || group.d.is_none() {
        return valid;
    }

    rds_data
        .ews
        .set_block_b_lsb((group.b.unwrap() & 0b11111) as u8);
    rds_data.ews.set_block_c(group.c.unwrap());
    rds_data.ews.set_block_d(group.d.unwrap());
    valid.set_ews(true);
    valid
}

fn decode_ptyn(group: &Group, rds_data: &mut RdsData) -> ValidFields {
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

// Type 10 groups: Program Type Name.
fn decode_group_type_10a(group: &Group, rds_data: &mut RdsData) -> ValidFields {
    // See RBDS Standard section 3.1.5.14.
    decode_ptyn(group, rds_data)
}

// Type 10 groups: Open data.
fn decode_group_type_10b(group: &Group, rds_data: &mut RdsData) -> ValidFields {
    // See RBDS Standard section 3.1.5.14.
    match group.get_type().version() {
        GroupVersion::A => decode_ptyn(group, rds_data),
        GroupVersion::B => decode_oda(group, group.get_type(), rds_data),
    }
}

// Type 11 groups: Open Data Application.
fn decode_group_type_11(group: &Group, rds_data: &mut RdsData) -> ValidFields {
    // See RBDS Standard section 3.1.5.15.
    decode_oda(group, group.get_type(), rds_data)
}

// Type 12 groups: Open Data Application.
fn decode_group_type_12(group: &Group, rds_data: &mut RdsData) -> ValidFields {
    // See RBDS Standard section 3.1.5.16.
    decode_oda(group, group.get_type(), rds_data)
}

// Type 13A groups: Enhanced Radio Paging or ODA.
fn decode_group_type_13a(group: &Group, _rds_data: &mut RdsData) -> ValidFields {
    // See RBDS Standard section 3.1.5.17.
    #[bitfield(bits = 16)]
    struct BlockB {
        group_type: GroupType,     // Group type (code + version).
        traffic_program: bool,     // TP bit.
        program_type: ProgramType, // PTY: Program type.
        information: B2,
        sty: B3,
    }
    let _block_b = BlockB::from_bytes(group.b.unwrap().to_be_bytes());

    // The type 13A group may be used for ODA when it is not used for Radio
    // Paging, and its group structure is then as shown in 3.1.4.2

    // TODO: How to determine if this is used for radio paging???
    ValidFields::new()
}

// Type 13B groups: Open Data Application.
fn decode_group_type_13b(group: &Group, rds_data: &mut RdsData) -> ValidFields {
    // See RBDS Standard section 3.1.5.18.
    decode_oda(group, group.get_type(), rds_data)
}

// Type 14 groups: Enhanced Other Networks information.
fn decode_group_type_14a(group: &Group, _rds_data: &mut RdsData) -> ValidFields {
    // See RBDS Standard section 3.1.5.19.
    #[bitfield(bits = 16)]
    struct BlockB {
        group_type: GroupType,     // Group type (code + version).
        traffic_program: bool,     // TP bit.
        program_type: ProgramType, // PTY: Program type.
        tp_on: bool,               // TP (ON).
        variant_code: B4,
    }
    let _block_b = BlockB::from_bytes(group.b.unwrap().to_be_bytes());
    // TODO: finish me.
    ValidFields::new()
}

// Type 14 groups: Enhanced Other Networks information.
fn decode_group_type_14b(group: &Group, _rds_data: &mut RdsData) -> ValidFields {
    // See RBDS Standard section 3.1.5.19.
    #[bitfield(bits = 16)]
    struct BlockB {
        group_type: GroupType,     // Group type (code + version).
        traffic_program: bool,     // TP bit.
        program_type: ProgramType, // PTY: Program type.
        tp_on: bool,               // TP (ON).
        ta_on: bool,               // TA (ON).
        unused: B3,
    }
    let _block_b = BlockB::from_bytes(group.b.unwrap().to_be_bytes());
    // TODO: finish me.
    ValidFields::new()
}

// Type 15 groups: Fast basic tuning and switching information.
fn decode_group_type_15(_group: &Group, _rds_data: &RdsData) -> ValidFields {
    ValidFields::new()
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

    /// Decode a group of RDS data and update the supplied RDS data object.
    /// Will return a ValidFields bitfield which describes the updated RDS
    /// data fields.
    /// Note: RdsData::valid also describes the valid data fields, but those
    /// are cumulative over all calls to `decode()` with the same RDS data
    /// object.
    pub fn decode(&mut self, group: &Group, rds_data: &mut RdsData) -> ValidFields {
        let mut valid = ValidFields::default();

        if group.a.is_some() {
            rds_data.program_information =
                ProgramInformation::from_bytes(group.a.unwrap().to_be_bytes());
            valid.set_pi_code(true);
        }
        if group.b.is_none() {
            rds_data.valid = rds_data.valid | valid;
            return valid;
        }

        // We don't yet know what block/version this is, but decode as 2B as all
        // blocks share the first four common fields.
        let block_b = GroupType2BlockB::from_bytes(group.b.unwrap().to_be_bytes());
        valid = valid | decode_block_b_common(&block_b, rds_data);

        let new_valid = match (block_b.group_type().code(), block_b.group_type().version()) {
            (0, GroupVersion::A) | (0, GroupVersion::B) => {
                decode_group_type_0(&group, rds_data, self.advanced_ps_decoding)
            }
            (1, GroupVersion::A) | (1, GroupVersion::B) => decode_group_type_1(&group, rds_data),
            (2, GroupVersion::A) => decode_group_type_2a(&group, rds_data),
            (2, GroupVersion::B) => decode_group_type_2b(&group, rds_data),
            (3, GroupVersion::A) => decode_group_type_3a(&group, rds_data),
            (3, GroupVersion::B) => decode_group_type_3b(&group, rds_data),
            (4, GroupVersion::A) => decode_group_type_4a(&group, rds_data),
            (4, GroupVersion::B) => decode_group_type_4b(&group, rds_data),
            (5, GroupVersion::A) => decode_group_type_5a(&group, rds_data),
            (5, GroupVersion::B) => decode_group_type_5b(&group, rds_data),
            (6, GroupVersion::A) | (6, GroupVersion::B) => decode_group_type_6(&group, rds_data),
            (7, GroupVersion::A) => decode_group_type_7a(&group, rds_data),
            (7, GroupVersion::B) => decode_group_type_7b(&group, rds_data),
            (8, GroupVersion::A) | (8, GroupVersion::B) => decode_group_type_8(&group, rds_data),
            (9, GroupVersion::A) | (9, GroupVersion::B) => decode_group_type_9(&group, rds_data),
            (10, GroupVersion::A) => decode_group_type_10a(&group, rds_data),
            (10, GroupVersion::B) => decode_group_type_10b(&group, rds_data),
            (11, GroupVersion::A) | (11, GroupVersion::B) => decode_group_type_11(&group, rds_data),
            (12, GroupVersion::A) | (12, GroupVersion::B) => decode_group_type_12(&group, rds_data),
            (13, GroupVersion::A) => decode_group_type_13a(&group, rds_data),
            (13, GroupVersion::B) => decode_group_type_13b(&group, rds_data),
            (14, GroupVersion::A) => decode_group_type_14a(&group, rds_data),
            (14, GroupVersion::B) => decode_group_type_14b(&group, rds_data),
            (15, GroupVersion::A) | (15, GroupVersion::B) => decode_group_type_15(&group, rds_data),
            _ => {
                // Other group types not implemented yet
                ValidFields::new()
            }
        };
        valid = valid | new_valid; // Merge in group decoding fields
        rds_data.valid = rds_data.valid | valid; // And into RDS object.
        valid
    }
}
