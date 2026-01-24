use crate::types::AltFreqDecodeTable;

// See table 12 in RBDS spec section 3.2.1.6.1.
const AF_MIN_FREQ_CODE: u8 = 1;
const AF_MAX_FREQ_CODE: u8 = 204;
const AF_FILLER_CODE: u8 = 205;
const AF_MIN_COUNT_CODE: u8 = 225;
const AF_MAX_COUNT_CODE: u8 = 249;
const AF_LF_MF_FOLLOWS: u8 = 250;

fn is_freq_code_count(freq_code: u8) -> bool {
    AF_MIN_COUNT_CODE <= freq_code && freq_code <= AF_MAX_COUNT_CODE
}

fn freq_code_to_count(freq_code: u8) -> u8 {
    1 + freq_code - AF_MIN_COUNT_CODE
}

/// Group of multiple decoded AF tables
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AltFreqTableGroup {
    pub current_table_idx: i8,
    pub count: u8,
    pub tables: [AltFreqDecodeTable; 20],
}

impl AltFreqTableGroup {
    pub fn decode_freq_group_block(&mut self, block: u16) {
        let first_byte = (block >> 8) as u8;
        let second_byte = (block & 0xFF) as u8;

        // if is_freq_code_count(first_byte) {
        //     self.decode_start_block(group, freq_code_to_count(first_byte), second_byte);
        // } else {
        //     self.decode_nth_block(group, first_byte, second_byte);
        // }
    }
}
