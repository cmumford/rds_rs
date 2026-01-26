use crate::af_codes::*;
use crate::af_decode_table::AltFreqDecodeTable;
use crate::types::{AltFreqAttribute, AltFreqEncoding, Band, Frequency};

fn is_freq_code_count(freq_code: u8) -> bool {
    AF_MIN_COUNT_CODE <= freq_code && freq_code <= AF_MAX_COUNT_CODE
}

fn freq_code_to_count(freq_code: u8) -> u8 {
    1 + freq_code - AF_MIN_COUNT_CODE
}

pub fn af_code_to_freq(freq_code: u8, band: Band) -> u16 {
    if band == Band::Uhf {
        // If a UHF band.
        return 876u16 + (freq_code as u16) - 1;
    }

    if freq_code < 16 {
        // If LF
        return (153 + 9 * (freq_code - 1)) as u16;
    }

    531u16 + 9 * ((freq_code as u16) - 16) // MF
}

/// Group of multiple decoded AF tables.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AltFreqTableGroup {
    pub pvt_current_table_idx: i8,       // Index of current decode table.
    pub count: u8,                       // Number of tables in use.
    pub table: [AltFreqDecodeTable; 20], // Decoded alternative frequencies.
}

impl AltFreqTableGroup {
    pub fn decode_freq_group_block(&mut self, block: u16) {
        let first_byte = (block >> 8) as u8;
        let second_byte = (block & 0xFF) as u8;

        if is_freq_code_count(first_byte) {
            self.decode_start_block(freq_code_to_count(first_byte), second_byte);
        } else {
            self.decode_nth_block(first_byte, second_byte);
        }
    }

    /// Find a table if the table's tuned frequency matches `tuned_freq`.
    fn find_af_table_idx(&mut self, tuned_freq: &Frequency) -> i8 {
        for i in 0..self.count {
            if self.table[i as usize].table.tuned_freq == *tuned_freq {
                return i as i8;
            }
        }
        return -1;
    }

    fn decode_start_block(&mut self, num_freqs_in_table: u8, second_byte: u8) {
        let mut encoding_method = AltFreqEncoding::Unknown;

        if self.count == 1 && self.table[0].encoding == AltFreqEncoding::MethodA {
            // There is only every one "A" table, so reuse this one.
            self.pvt_current_table_idx = 0;
            encoding_method = AltFreqEncoding::MethodA;
        } else {
            self.pvt_current_table_idx = -1;
        }

        if num_freqs_in_table == 1 {
            // Only Method A encoding has a single-entry table, and there is only
            // one table with this method, so we know it.
            self.pvt_current_table_idx = 0;
            encoding_method = AltFreqEncoding::MethodA;
        }

        if self.pvt_current_table_idx == -1 {
            // TODO: Make AF Method A more robust. Technically the second byte could
            // be a special code. Handle this correctly.
            let freq: Frequency = Frequency {
                band: Band::Uhf,
                attribute: AltFreqAttribute::SameProgram,
                freq: af_code_to_freq(second_byte, Band::Uhf),
            };

            self.pvt_current_table_idx = self.find_af_table_idx(&freq);
            if self.pvt_current_table_idx == -1 {
                if self.count == (self.table.len() as u8) {
                    // All tables are in use - can't allocate a new one.
                    return;
                }
                // Allocate a new table.
                self.pvt_current_table_idx = self.count as i8;
                let table = &mut self.table[self.pvt_current_table_idx as usize];
                table.encoding = encoding_method;

                if table.encoding == AltFreqEncoding::Unknown {
                    // Don't know if method A or B yet, so save in tuned_freq. Will
                    // move to entries if encoding method turns out to be method A.
                    table.table.tuned_freq = freq;
                }
            }
        }

        let table = &mut self.table[self.pvt_current_table_idx as usize];
        table.decode_freq_table_start_block(num_freqs_in_table, second_byte);
    }

    /// Decode freqnency blocks 2..n of the AF table.
    fn decode_nth_block(&mut self, first_byte: u8, second_byte: u8) {
        if self.pvt_current_table_idx < 0 {
            return;
        }

        let table = &mut self.table[self.pvt_current_table_idx as usize];

        table.decode_freq_table_nth_block(first_byte, second_byte);
    }
}
