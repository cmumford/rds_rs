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

fn get_uhf_frequency(idx: u8) -> u32 {
    87_600_000 + ((idx - 1) as u32) * 100000
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
    fn new() -> Self {
        return AfDecoder {
            awaiting_freq_cnt: 0,
            next_freq_is_lf_mf: false,
        };
    }

    fn reset(&mut self) {
        self.awaiting_freq_cnt = 0;
        self.next_freq_is_lf_mf = false;
    }

    fn decode_freq_code(&mut self, code: u8, table: &mut AfTable) -> Result<(), DecodeError> {
        let code_type = categorize_uhf_code(code);
        if code_type == CodeType::Unassigned {
            return Err(DecodeError::InvalidCode);
        }
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
                self.reset();
                return Err(DecodeError::InvalidCode);
            }
            let _ = table.add(get_lf_mf_frequency(code));
            return Ok(());
        }
        if code_type == CodeType::LfMfFollows {
            self.next_freq_is_lf_mf = true;
            return Ok(());
        }
        assert!(code_type == CodeType::Frequency);
        let _ = table.add(get_uhf_frequency(code));

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
    use crate::alt_freq_decoder::{
        AfDecoder, decode_freq_cnt, get_lf_mf_frequency, get_uhf_frequency,
    };
    use crate::alt_freq_table::AfTable;

    #[test]
    fn test_get_lf_mf_frequency() {
        assert_eq!(get_lf_mf_frequency(1), 153_000);
        assert_eq!(get_lf_mf_frequency(15), 279_000);
        assert_eq!(get_lf_mf_frequency(16), 531_000);
        assert_eq!(get_lf_mf_frequency(135), 1_602_000);
    }

    #[test]
    fn test_get_uhf_frequency() {
        assert_eq!(get_uhf_frequency(1), 87_600_000);
        assert_eq!(get_uhf_frequency(2), 87_700_000);
        assert_eq!(get_uhf_frequency(204), 107_900_000);
    }

    #[test]
    fn test_decode_freq_cnt() {
        assert_eq!(decode_freq_cnt(224), 0);
        assert_eq!(decode_freq_cnt(225), 1);
        assert_eq!(decode_freq_cnt(249), 25);
    }

    #[test]
    fn test_decoder_one_freq() {
        let mut table = AfTable::default();
        let mut decoder = AfDecoder::default();
        let result = decoder.decode_freq_block(Some(0xE1_01), &mut table);
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        assert_eq!(table.entries.len(), 1);
        let actual: Vec<_> = table.entries.iter().copied().collect();
        assert_eq!(actual, [87_600_000]);
    }

    // test to very scenario from Example A in RBDS Specification section 3.2.1.6.3.
    #[test]
    fn test_decoder_example_a() {
        let mut table = AfTable::default();
        let mut decoder = AfDecoder::default();
        let blocks = [0xE5_01, 0x02_03, 0x04_05];
        for block in blocks {
            let result = decoder.decode_freq_block(Some(block), &mut table);
            assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        }
        assert_eq!(table.entries.len(), 5);
        let actual: Vec<_> = table.entries.iter().copied().collect();
        assert_eq!(
            actual,
            [87_600_000, 87_700_000, 87_800_000, 87_900_000, 88_000_000]
        );
    }

    // test to very scenario from Example B in RBDS Specification section 3.2.1.6.3.
    #[test]
    fn test_decoder_example_b() {
        let mut table = AfTable::default();
        let mut decoder = AfDecoder::default();
        let blocks = [0xE5_01, 0x02_03, 0x04_CE];
        for block in blocks {
            let result = decoder.decode_freq_block(Some(block), &mut table);
            assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        }
        assert_eq!(table.entries.len(), 4);
        let actual: Vec<_> = table.entries.iter().copied().collect();
        assert_eq!(actual, [87_600_000, 87_700_000, 87_800_000, 87_900_000]);
    }

    // test to very scenario from Example C in RBDS Specification section 3.2.1.6.3.
    #[test]
    fn test_decoder_example_c() {
        let mut table = AfTable::default();
        let mut decoder = AfDecoder::default();
        let blocks = [0xE5_01, 0x02_03, 0xFA_10];
        for block in blocks {
            let result = decoder.decode_freq_block(Some(block), &mut table);
            assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        }
        assert_eq!(table.entries.len(), 4);
        let actual: Vec<_> = table.entries.iter().copied().collect();
        assert_eq!(actual, [87_600_000, 87_700_000, 87_800_000, 531_000]);
    }
}
