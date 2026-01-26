use crate::af_codes::*;
use crate::af_table::AltFreqTable;
use crate::af_table_group::af_code_to_freq;
use crate::types::{AltFreqAttribute, AltFreqEncoding, Band, Frequency};

/// Internal state while decoding an AF table
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RdsAfDecodeTablePrivate {
    pub band: Band,
    pub prev_encoding: AltFreqEncoding,
    pub expected_cnt: u8,
}

/// One AF decoding context
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AltFreqDecodeTable {
    pub table: AltFreqTable,
    pub encoding: AltFreqEncoding,
    pub pvt: RdsAfDecodeTablePrivate,
}

fn freq_code_is_freq(freq_code: u8) -> bool {
    return AF_MIN_FREQ_CODE <= freq_code && freq_code <= AF_MAX_FREQ_CODE;
}

impl AltFreqDecodeTable {
    fn dec_af_expected_count(&mut self) {
        if self.pvt.expected_cnt == 0 {
            return;
        }
        self.pvt.expected_cnt -= 1;
    }

    pub fn add_alt_freq(&mut self, freq: &Frequency) -> bool {
        self.dec_af_expected_count();
        self.table.insert_alt_freq(freq)
    }

    fn handle_freq_code(&mut self, freq_code: u8) -> bool {
        if freq_code == AF_FILLER_CODE {
            self.dec_af_expected_count();
            return true;
        }
        if freq_code == AF_LF_MF_FOLLOWS {
            self.pvt.band = Band::LfMf;
            self.dec_af_expected_count();
            return true;
        }
        // All others outside of codes which map to frequencies are ignored.
        let handled = !freq_code_is_freq(freq_code);
        if handled {
            self.dec_af_expected_count();
        }
        return handled;
    }

    pub fn decode_freq_table_start_block(&mut self, num_freqs_in_table: u8, second_byte: u8) {
        self.pvt.expected_cnt = num_freqs_in_table;
        self.pvt.band = Band::Uhf; // Always start with UHF, then LF/MF.

        if self.pvt.prev_encoding != AltFreqEncoding::Unknown {
            self.encoding = self.pvt.prev_encoding;
        }

        if self.handle_freq_code(second_byte) {
            return;
        }
        let freq = Frequency {
            band: self.pvt.band,
            attribute: AltFreqAttribute::SameProgram,
            freq: af_code_to_freq(second_byte, self.pvt.band),
        };

        self.add_alt_freq(&freq);
    }

    pub fn decode_freq_table_nth_block(&mut self, first_byte: u8, second_byte: u8) {
        if self.pvt.expected_cnt == 0 {
            // Got more frequency codes than we were expecting. Probably missed
            // a block to start a new table, so do nothing.
            return;
        }

        let handled_first = self.handle_freq_code(first_byte);
        let first_freq = Frequency {
            band: self.pvt.band,
            attribute: AltFreqAttribute::SameProgram,
            freq: af_code_to_freq(first_byte, self.pvt.band),
        };

        let handled_second = self.handle_freq_code(second_byte);
        let second_freq = Frequency {
            band: self.pvt.band,
            attribute: AltFreqAttribute::SameProgram,
            freq: af_code_to_freq(second_byte, self.pvt.band),
        };

        if self.encoding == AltFreqEncoding::Unknown {
            if handled_first && handled_second {
                // Still don't know, figure out next entry.
                return;
            }
            if handled_first || handled_second {
                // If only one handled, but not the second then this must be method A.
                self.encoding = AltFreqEncoding::MethodA;
            } else if first_freq == self.table.tuned_freq || second_freq == self.table.tuned_freq {
                // If either frequencies match first freq then must be method B.
                self.encoding = AltFreqEncoding::MethodB;
            } else {
                // Neither match tuned freq, so must be method A.
                self.encoding = AltFreqEncoding::MethodA;

                if self.table.tuned_freq.freq != 0 {
                    // Move the frequency, which we saved because we didn't know if this
                    // was method A or B, into the table.
                    let freq = self.table.tuned_freq;
                    self.add_alt_freq(&freq);
                    self.table.tuned_freq = Frequency::default();
                }
            }
        }
    }
}
