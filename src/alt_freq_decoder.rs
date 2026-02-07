use crate::alt_freq_table::{AfTable, Freq, FreqType};
use thiserror::Error;

// Section 3.2.1.6.1 describes how 8-bit values are mapped to either
// UHF frequencies, LF/MF frequencies, or othe special codes. These
// are described in three different tables:
//
// Table 10: VHF code table
// Table 11: Special meanings code table
// Table 12: LF/MF code table - for ITU regions 1 and 3 (9 kHz spacing)
//
// These entries distill down to these categories.
#[derive(Debug, Clone, PartialEq)]
enum CodeType {
    Unassigned,   // Unused/unassigned.
    Frequency,    // A UHF frequency value.
    AltFreqCount, // Number of AF's to follow.
    LfMfFollows,  // Next entry is a LF/MF freq.
    Filler,       // A no-op.
}

// Categorize a code from table 10/11 above.
fn categorize_uhf_code(code: u8) -> CodeType {
    match code {
        1..=204 => CodeType::Frequency,
        205 => CodeType::Filler,
        224..=249 => CodeType::AltFreqCount,
        250 => CodeType::LfMfFollows,
        _ => CodeType::Unassigned,
    }
}

fn decode_freq_cnt(code: u8) -> u8 {
    code - 224_u8
}

fn get_lf_mf_frequency(idx: u8) -> u32 {
    if idx >= 1 && idx < 16 {
        return 153_000 + ((idx as u32) - 1) * 9000;
    }
    return 531_000 + ((idx as u32) - 16) * 9000;
}

fn get_uhf_frequency(idx: u8) -> u32 {
    87_600_000 + ((idx - 1) as u32) * 100000
}

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("Use of invalid code")]
    InvalidCode,
    #[error("Invalid frequency count")]
    InvalidFreqCount,
}

#[derive(Debug, Default, Clone, PartialEq)]
enum EcodingMethod {
    #[default]
    Unknown, // Not enough data yet to determine.
    MethodA, // See RBDS Spec section 3.2.1.6.3.
    MethodB, // See RBDS Spec section 3.2.1.6.4.
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct AfDecoder {
    awaiting_freq_cnt: u8,          // Number of expected frequencies in table.
    next_freq_is_lf_mf: bool,       // Is the next frquency LF/MF?
    first_freq_code: u8,            // The first frequency code in the table.
    first_freq_in_table: bool,      // Has first frequency been added to the table?
    encoding_method: EcodingMethod, // Table encoding method.
}

impl AfDecoder {
    fn reset(&mut self) {
        self.awaiting_freq_cnt = 0;
        self.next_freq_is_lf_mf = false;
        self.first_freq_code = 0;
        self.first_freq_in_table = false;
        self.encoding_method = EcodingMethod::Unknown;
    }

    fn write_first_freq_to_table(&mut self, table: &mut AfTable) {
        assert!(!self.first_freq_in_table);
        assert_ne!(self.first_freq_code, 0);
        self.first_freq_in_table = true;
        let _ = table.add(&Freq {
            frequency: get_uhf_frequency(self.first_freq_code),
            freq_type: FreqType::SameProgram,
        });
    }

    fn decrement_awaiting_freq_cnt(&mut self) {
        if self.awaiting_freq_cnt > 0 {
            self.awaiting_freq_cnt -= 1;
        }
        if self.awaiting_freq_cnt == 0 {
            self.reset();
        }
    }

    // Only called when we know encoding is method A.
    fn decode_for_method_a_code(
        &mut self,
        code: u8,
        table: &mut AfTable,
    ) -> Result<(), DecodeError> {
        if !self.first_freq_in_table && self.first_freq_code != 0 {
            self.write_first_freq_to_table(table);
        }
        match categorize_uhf_code(code) {
            CodeType::Unassigned => {
                return Err(DecodeError::InvalidCode);
            }
            CodeType::AltFreqCount => {
                return Err(DecodeError::InvalidCode);
            }
            CodeType::LfMfFollows => {
                if self.next_freq_is_lf_mf {
                    return Err(DecodeError::InvalidCode);
                }
                self.next_freq_is_lf_mf = true;
            }
            CodeType::Frequency => {
                if self.awaiting_freq_cnt == 0 {
                    return Err(DecodeError::InvalidFreqCount);
                }
                let freq: u32;
                if self.next_freq_is_lf_mf {
                    self.next_freq_is_lf_mf = false;
                    freq = get_lf_mf_frequency(code);
                } else {
                    freq = get_uhf_frequency(code);
                }
                let _ = table.add(&Freq {
                    frequency: freq,
                    freq_type: FreqType::SameProgram,
                });
                self.decrement_awaiting_freq_cnt();
            }
            CodeType::Filler => (),
        }
        Ok(())
    }

    // Only called when we know encoding is method A.
    // and only for blocks 2..n and not the first block.
    fn decode_for_method_a(
        &mut self,
        code_pair: [u8; 2],
        table: &mut AfTable,
    ) -> Result<(), DecodeError> {
        let _ = self.decode_for_method_a_code(code_pair[0], table);
        let _ = self.decode_for_method_a_code(code_pair[1], table);
        Ok(())
    }

    // Only called when we know encoding is method B.
    // and only for blocks 2..n and not the first block.
    fn decode_for_method_b(
        &mut self,
        code_pair: [u8; 2],
        table: &mut AfTable,
    ) -> Result<(), DecodeError> {
        let freq_type = if code_pair[0] < code_pair[1] {
            FreqType::SameProgram
        } else {
            FreqType::RegionalVariant
        };
        if code_pair[0] == self.first_freq_code {
            assert_ne!(code_pair[1], self.first_freq_code);
            let _ = table.add(&Freq {
                frequency: get_uhf_frequency(code_pair[1]),
                freq_type: freq_type,
            });
        } else if code_pair[1] == self.first_freq_code {
            assert_ne!(code_pair[0], self.first_freq_code);
            let _ = table.add(&Freq {
                frequency: get_uhf_frequency(code_pair[0]),
                freq_type: freq_type,
            });
        } else {
            assert!(false, "Freq does not match tuned freq");
        }
        self.decrement_awaiting_freq_cnt();
        self.decrement_awaiting_freq_cnt();
        Ok(())
    }

    fn decode_for_unknown_method(
        &mut self,
        code_pair: [u8; 2],
        table: &mut AfTable,
    ) -> Result<(), DecodeError> {
        let ct1 = categorize_uhf_code(code_pair[0]);
        let ct2 = categorize_uhf_code(code_pair[1]);

        if self.first_freq_code == 0 {
            assert_eq!(self.first_freq_in_table, false);
            match ct1 {
                CodeType::Unassigned => return Err(DecodeError::InvalidCode),
                CodeType::AltFreqCount => {
                    self.awaiting_freq_cnt = decode_freq_cnt(code_pair[0]);
                    if ct2 == CodeType::Frequency {
                        // Could be Method A or B - don't know yet. When the next block
                        // is decoded that will be determined.
                        self.first_freq_code = code_pair[1];
                        if self.awaiting_freq_cnt < 3 {
                            self.encoding_method = EcodingMethod::MethodA;
                            self.write_first_freq_to_table(table);
                        }
                        self.decrement_awaiting_freq_cnt();
                        return Ok(());
                    } else {
                        // This could be a LF/MF code (or other), so send to method A decoder.
                        self.encoding_method = EcodingMethod::MethodA;
                        return self.decode_for_method_a_code(code_pair[1], table);
                    }
                }
                _ => {
                    // No freq count. Probably started decoding in a table stream
                    // so wait for next freq count code.
                    return Ok(());
                }
            }
        }

        // If here then processing block 2..n.
        if ct1 != CodeType::Frequency || ct2 != CodeType::Frequency {
            self.encoding_method = EcodingMethod::MethodA;
        } else if code_pair[0] == self.first_freq_code || code_pair[1] == self.first_freq_code {
            self.encoding_method = EcodingMethod::MethodB;
        } else {
            self.encoding_method = EcodingMethod::MethodA;
        }

        match self.encoding_method {
            EcodingMethod::MethodA => self.decode_for_method_a(code_pair, table),
            EcodingMethod::MethodB => self.decode_for_method_b(code_pair, table),
            _ => panic!("Shouldn't get here"),
        }
    }

    // Decode a C block of frequencies or control codes. If none was received,
    // too many errors, etc., then pass in None to reset the decoder.
    pub fn decode_freq_block(
        &mut self,
        block_c: Option<u16>,
        table: &mut AfTable,
    ) -> Result<(), DecodeError> {
        if block_c.is_none() {
            self.reset();
            return Ok(());
        }
        let code_pair = block_c.unwrap().to_be_bytes();
        match self.encoding_method {
            EcodingMethod::MethodA => self.decode_for_method_a(code_pair, table),
            EcodingMethod::MethodB => self.decode_for_method_b(code_pair, table),
            EcodingMethod::Unknown => self.decode_for_unknown_method(code_pair, table),
        }
    }
}

#[cfg(test)]
mod tests;
