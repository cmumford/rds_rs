use crate::alt_freq_table::AfTable;
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
    Unassigned,   // Unused/unassigned/filler.
    Frequency,    // A UHF frequency value.
    AltFreqCount, // Number of AF's to follow.
    LfMfFollows,  // Next entry is a LF/MF freq.
}

enum FreqBand {
    Lf,      // A LF frequency value.
    Mf,      // A MF frequency value.
    Invalid, // Invalid freqency.
}

// Categorize a code from table 10/11 above.
fn categorize_uhf_code(code: u8) -> CodeType {
    match code {
        1..=204 => CodeType::Frequency,
        224..=249 => CodeType::AltFreqCount,
        250 => CodeType::LfMfFollows,
        _ => CodeType::Unassigned,
    }
}

// Categorize a code from table 12 above.
fn categorize_lf_mf(code: u8) -> FreqBand {
    match code {
        0..=15 => FreqBand::Lf,
        16..=135 => FreqBand::Mf,
        _ => FreqBand::Invalid,
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

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("Use of invalid code")]
    InvalidCode,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct AfDecoder {
    awaiting_freq_cnt: u8,
    next_freq_is_lf_mf: bool,
}

impl AfDecoder {
    fn reset(&mut self) {
        self.awaiting_freq_cnt = 0;
        self.next_freq_is_lf_mf = false;
    }

    fn decode_freq_code(&mut self, code: u8, table: &mut AfTable) -> Result<(), DecodeError> {
        let code_type = categorize_uhf_code(code);
        if self.awaiting_freq_cnt == 0 {
            if code_type != CodeType::AltFreqCount {
                // Not an error because decoding may have started after the
                // AF table start. Waiting until the next table start which is
                // indicated by the frequency count.
                return Ok(());
            }
            self.awaiting_freq_cnt = decode_freq_cnt(code);
            return Ok(());
        }
        if self.next_freq_is_lf_mf {
            self.next_freq_is_lf_mf = false;
            if code_type != CodeType::Frequency {
                return Err(DecodeError::InvalidCode);
            }
        }
        Ok(())
    }

    // Decode a C block of frequencies or control codes. If none was received,
    // too many errors, etc., then pass in None to reset the decoder.
    fn decode_freq_block(
        &mut self,
        block_c: Option<u16>,
        table: &mut AfTable,
    ) -> Result<(), DecodeError> {
        if block_c.is_none() {
            self.reset();
            return Ok(());
        }
        let _ = self.decode_freq_code((block_c.unwrap() >> 8) as u8, table);
        let _ = self.decode_freq_code((block_c.unwrap() & 0xff) as u8, table);
        Ok(())
    }
}

#[cfg(test)]

mod tests {
    use crate::alt_freq_decoder::{decode_freq_cnt, get_lf_mf_frequency};

    #[test]
    fn test_get_lf_mf_frequency() {
        assert_eq!(get_lf_mf_frequency(1), 153_000);
        assert_eq!(get_lf_mf_frequency(15), 279_000);
        assert_eq!(get_lf_mf_frequency(16), 531_000);
        assert_eq!(get_lf_mf_frequency(135), 1_602_000);
    }

    #[test]
    fn test_decode_freq_cnt() {
        assert_eq!(decode_freq_cnt(224), 0);
        assert_eq!(decode_freq_cnt(225), 1);
        assert_eq!(decode_freq_cnt(249), 25);
    }
}
