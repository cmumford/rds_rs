#![allow(dead_code)]

use crate::alt_freq_decoder::get_uhf_frequency;
use crate::alt_freq_table::{Freq, FreqType};
use crate::oda::{OdaEntry, decode_oda, is_oda_group_type_used, is_valid_oda_app_id};
use crate::ptyn::decode_ptyn;
use crate::radiotext::RtVariant;
use crate::rds::RdsData;
use crate::types::{
    Content, Group, GroupType, GroupVersion, NUM_TDC, Pin, ProgramInformation, ProgramType,
    SlcData, ValidFields,
};
use core::ops::BitOr;
use modular_bitfield_msb::prelude::*;

// See RBDS Standard section 3.1.5.3.
#[bitfield(bits = 16)]
struct GroupType2BlockB {
    group_type: GroupType,     // Group type (code + version).
    tp: bool,                  // TP bit.
    program_type: ProgramType, // PTY: Program type.
    text_flag: RtVariant,
    text_segment_addr: B4,
}

impl BitOr for ValidFields {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        const N: usize = core::mem::size_of::<ValidFields>();
        let l = self.into_bytes();
        let r = rhs.into_bytes();
        let mut m = [0u8; N];
        for i in 0..N {
            m[i] = l[i] | r[i];
        }
        Self::from_bytes(m)
    }
}

impl Group {
    fn get_type(&self) -> GroupType {
        GroupType2BlockB::from_bytes(self.b.unwrap().to_be_bytes()).group_type()
    }
}

fn decode_block_b_common(block: &GroupType2BlockB, rds_data: &mut RdsData) -> ValidFields {
    rds_data.tn.traffic.set_tp(block.tp());
    rds_data.tn.program_type = block.program_type();
    ValidFields::new().with_tp(true).with_pty(true)
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
        group_type: GroupType,      // Group type (code + version).
        traffic_program: bool,      // TP bit.
        program_type: ProgramType,  // PTY: Program type.
        traffic_announcement: bool, // TA bit: section 3.2.1.3.
        ms: Content,                // M/S bit: section 3.2.1.4.
        di_bit: bool,               // DI bit: section 3.2.1.5.
        seg_addr: B2,               // Prog. service name and DI segment addr.
    }

    let mut valid = ValidFields::new();
    let block_b = BlockB::from_bytes(group.b.unwrap().to_be_bytes());
    if block_b.group_type().version() == GroupVersion::B && group.c.is_some() {
        let _ = rds_data
            .alt_freq_decoder
            .decode_freq_block(group.c, &mut rds_data.alt_freqs);
        valid.set_af(true);
    }
    // Decoder identification and Dynamic PTY indicator / DI codes.
    // The d bits come MSB first.
    match block_b.seg_addr() {
        0 => rds_data.did_pty.set_dynamic_pty(block_b.di_bit()),
        1 => rds_data.did_pty.set_compressed(block_b.di_bit()),
        2 => rds_data.did_pty.set_artificial_head(block_b.di_bit()),
        3 => rds_data.did_pty.set_stereo(block_b.di_bit()),
        _ => return valid,
    }
    if group.d.is_none() {
        return valid;
    }
    rds_data.tn.traffic.set_ta(block_b.traffic_announcement());
    valid.set_ta(true);
    rds_data.content = block_b.ms();
    valid.set_ms(true);

    let pair_idx = 2 * block_b.seg_addr();
    let ps_bytes = group.d.unwrap().to_be_bytes();
    if advanced_ps_decoding {
        if rds_data
            .tn
            .ps
            .update_advanced((pair_idx + 0) as usize, ps_bytes[0])
        {
            valid.set_ps(true);
        }
        if rds_data
            .tn
            .ps
            .update_advanced((pair_idx + 1) as usize, ps_bytes[1])
        {
            valid.set_ps(true);
        }
    } else {
        rds_data.tn.ps.update_simple(pair_idx as usize, ps_bytes);
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
    rds_data.tn.pin = group
        .d
        .map(|d| Pin::from_bytes(d.to_be_bytes()))
        .unwrap_or_default();
    if group.d.is_some() {
        valid.set_pin(true);
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
fn decode_group_type_14a(group: &Group, rds_data: &mut RdsData) -> ValidFields {
    // See RBDS Standard section 3.1.5.19.
    #[bitfield(bits = 16)]
    struct BlockB {
        group_type: GroupType,     // Group type (code + version).
        traffic_program: bool,     // TP bit.
        program_type: ProgramType, // PTY: Program type.
        tp_on: bool,               // TP (ON).
        variant_code: B4,
    }
    let block_b = BlockB::from_bytes(group.b.unwrap().to_be_bytes());
    let mut valid = ValidFields::new().with_tp_on(true);
    rds_data.on.traffic.set_tp(block_b.tp_on());
    match block_b.variant_code() {
        0..=3 => {
            let idx: usize = 2 * (block_b.variant_code() as usize);
            if group.c.is_some() {
                rds_data
                    .tn
                    .ps
                    .update_simple(idx, group.c.unwrap().to_be_bytes());
                valid.set_ps_on(true);
            }
        }
        4 => {
            let _ = rds_data
                .on_freq_decoder
                .decode_freq_block(group.c, &mut rds_data.on_freqs);
            valid.set_on_freqs(true);
        }
        5..=9 => {
            if group.c.is_some() {
                let freqs = group.c.unwrap().to_be_bytes();
                rds_data.map_freqs.add(&Freq {
                    frequency: get_uhf_frequency(freqs[1]),
                    freq_type: FreqType::SameProgram,
                });
                valid.set_map_freqs(true);
            }
        }
        13 => {
            if group.c.is_some() {
                rds_data.on.traffic.set_ta((group.c.unwrap() & 0b1) != 0);
                valid.set_ta_on(true);
            }
        }
        14 => {
            if group.c.is_some() {
                rds_data.on.pin = group
                    .c
                    .map(|c| Pin::from_bytes(c.to_be_bytes()))
                    .unwrap_or_default();
                valid.set_pin_on(true);
            }
        }
        _ => {}
    }
    valid
}

// Type 14 groups: Enhanced Other Networks information.
fn decode_group_type_14b(group: &Group, rds_data: &mut RdsData) -> ValidFields {
    // See RBDS Standard section 3.1.5.19.
    #[bitfield(bits = 16)]
    struct BlockB {
        group_type: GroupType,     // Group type (code + version).
        tp: bool,                  // TP bit.
        program_type: ProgramType, // PTY: Program type.
        tp_on: bool,               // TP (ON).
        ta_on: bool,               // TA (ON).
        unused: B3,
    }
    let block_b = BlockB::from_bytes(group.b.unwrap().to_be_bytes());
    let valid = ValidFields::new()
        .with_tp(true)
        .with_ta_on(true)
        .with_tp_on(true);
    rds_data.tn.traffic.set_tp(block_b.tp());
    rds_data.on.traffic.set_ta(block_b.ta_on());
    rds_data.on.traffic.set_tp(block_b.tp_on());
    // TODO: Parse PI code.
    valid
}

// Type 15 groups: Fast basic tuning and switching information.
fn decode_group_type_15(_group: &Group, _rds_data: &RdsData) -> ValidFields {
    ValidFields::new()
}

pub struct Decoder {
    advanced_ps_decoding: bool,
}

impl<'a> Decoder {
    pub fn new(advanced_ps_decoding: bool) -> Self {
        Decoder {
            advanced_ps_decoding: advanced_ps_decoding,
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
            valid.set_pi(true);
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

#[cfg(test)]
mod tests;
