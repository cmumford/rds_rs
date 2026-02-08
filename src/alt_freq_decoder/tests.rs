#[cfg(test)]

mod tests {
    use crate::alt_freq_decoder::{
        AfDecoder, EncodingMethod, decode_freq_cnt, get_lf_mf_frequency, get_uhf_frequency,
    };
    use crate::alt_freq_table::{AfTable, Freq, FreqType};
    use heapless::Vec;

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
        assert_eq!(table.iter().count(), 1);
        let actual: Vec<Freq, 45> = table.iter().copied().collect();
        assert_eq!(
            actual,
            [Freq {
                frequency: 87_600_000,
                freq_type: FreqType::SameProgram
            }]
        );
        assert_eq!(decoder.awaiting_freq_cnt, 0);
        assert_eq!(decoder.encoding_method, EncodingMethod::Unknown);
    }

    // test to very scenario from Example A in RBDS Specification section 3.2.1.6.3.
    #[test]
    fn test_decoder_method_a_example_a() {
        let mut table = AfTable::default();
        let mut decoder = AfDecoder::default();
        let blocks = [0xE5_01, 0x02_03, 0x04_05];
        for block in blocks {
            let result = decoder.decode_freq_block(Some(block), &mut table);
            assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        }
        assert_eq!(decoder.awaiting_freq_cnt, 0);
        assert_eq!(table.iter().count(), 5);
        let actual: Vec<Freq, 45> = table.iter().copied().collect();
        assert_eq!(
            actual,
            [
                Freq {
                    frequency: 87_600_000,
                    freq_type: FreqType::SameProgram
                },
                Freq {
                    frequency: 87_700_000,
                    freq_type: FreqType::SameProgram
                },
                Freq {
                    frequency: 87_800_000,
                    freq_type: FreqType::SameProgram
                },
                Freq {
                    frequency: 87_900_000,
                    freq_type: FreqType::SameProgram
                },
                Freq {
                    frequency: 88_000_000,
                    freq_type: FreqType::SameProgram
                },
            ]
        );
    }

    // test to very scenario from Example B in RBDS Specification section 3.2.1.6.3.
    #[test]
    fn test_decoder_method_a_example_b() {
        let mut table = AfTable::default();
        let mut decoder = AfDecoder::default();
        let blocks = [0xE4_01, 0x02_03, 0x04_CE];
        for block in blocks {
            let result = decoder.decode_freq_block(Some(block), &mut table);
            assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        }
        assert_eq!(table.iter().count(), 4);
        assert_eq!(decoder.awaiting_freq_cnt, 0);
        assert_eq!(decoder.encoding_method, EncodingMethod::Unknown);
        let actual: Vec<Freq, 45> = table.iter().copied().collect();
        assert_eq!(
            actual,
            [
                Freq {
                    frequency: 87_600_000,
                    freq_type: FreqType::SameProgram
                },
                Freq {
                    frequency: 87_700_000,
                    freq_type: FreqType::SameProgram
                },
                Freq {
                    frequency: 87_800_000,
                    freq_type: FreqType::SameProgram
                },
                Freq {
                    frequency: 87_900_000,
                    freq_type: FreqType::SameProgram
                },
            ]
        );
    }

    // test to very scenario from Example C in RBDS Specification section 3.2.1.6.3.
    #[test]
    fn test_decoder_method_a_example_c() {
        let mut table = AfTable::default();
        let mut decoder = AfDecoder::default();
        let blocks = [0xE4_01, 0x02_03, 0xFA_10];
        for block in blocks {
            let result = decoder.decode_freq_block(Some(block), &mut table);
            assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        }
        assert_eq!(table.iter().count(), 4);
        assert_eq!(decoder.awaiting_freq_cnt, 0);
        let actual: Vec<Freq, 45> = table.iter().copied().collect();
        assert_eq!(
            actual,
            [
                Freq {
                    frequency: 87_600_000,
                    freq_type: FreqType::SameProgram
                },
                Freq {
                    frequency: 87_700_000,
                    freq_type: FreqType::SameProgram
                },
                Freq {
                    frequency: 87_800_000,
                    freq_type: FreqType::SameProgram
                },
                Freq {
                    frequency: 531_000,
                    freq_type: FreqType::SameProgram
                },
            ]
        );
    }

    #[test]
    fn test_decoder_method_b_example_1() {
        let mut table = AfTable::default();
        let mut decoder = AfDecoder::default();
        let blocks = [0xEB_12, 0x12_78, 0x12_8E, 0x0D_12, 0x97_12, 0x12_0F];
        for block in blocks {
            let result = decoder.decode_freq_block(Some(block), &mut table);
            assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        }
        assert_eq!(table.iter().count(), 5);
        assert_eq!(decoder.awaiting_freq_cnt, 0);
        assert_eq!(decoder.encoding_method, EncodingMethod::Unknown);
        let mut actual: Vec<Freq, 45> = table.iter().copied().collect();
        actual.sort_by_key(|f| f.frequency);
        assert_eq!(
            actual,
            [
                Freq {
                    frequency: 88_800_000,
                    freq_type: FreqType::SameProgram
                },
                Freq {
                    frequency: 89_000_000,
                    freq_type: FreqType::RegionalVariant
                },
                Freq {
                    frequency: 99_500_000,
                    freq_type: FreqType::SameProgram
                },
                Freq {
                    frequency: 101_700_000,
                    freq_type: FreqType::SameProgram
                },
                Freq {
                    frequency: 102_600_000,
                    freq_type: FreqType::RegionalVariant
                },
            ]
        );
    }

    #[test]
    fn test_decoder_method_b_example_2() {
        let mut table = AfTable::default();
        let mut decoder = AfDecoder::default();
        let blocks = [0xE9_78, 0x12_78, 0x78_86, 0xAD_78, 0x78_10];
        for block in blocks {
            let result = decoder.decode_freq_block(Some(block), &mut table);
            assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        }
        assert_eq!(table.iter().count(), 4);
        assert_eq!(decoder.awaiting_freq_cnt, 0);
        assert_eq!(decoder.encoding_method, EncodingMethod::Unknown);
        let mut actual: Vec<Freq, 45> = table.iter().copied().collect();
        actual.sort_by_key(|f| f.frequency);

        assert_eq!(
            actual,
            [
                Freq {
                    frequency: 89_100_000,
                    freq_type: FreqType::RegionalVariant
                },
                Freq {
                    frequency: 89_300_000,
                    freq_type: FreqType::SameProgram
                },
                Freq {
                    frequency: 100_900_000,
                    freq_type: FreqType::SameProgram
                },
                Freq {
                    frequency: 104_800_000,
                    freq_type: FreqType::RegionalVariant
                },
            ]
        );
    }

    #[test]
    fn test_decoder_method_b_example_3() {
        // This decoded the two method B encoded tables from
        // RBDS spec 3.2.1.6.4 AF method B. These comprise a single AF
        // table of nine entries.
        let mut table = AfTable::default();
        let mut decoder = AfDecoder::default();
        let blocks = [
            0xEB_12, 0x12_78, 0x12_8E, 0x0D_12, 0x97_12, 0x12_0F, 0xE9_78, 0x12_78, 0x78_86,
            0xAD_78, 0x78_10,
        ];
        for block in blocks {
            let result = decoder.decode_freq_block(Some(block), &mut table);
            assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        }
        assert_eq!(table.iter().count(), 9);
        assert_eq!(decoder.awaiting_freq_cnt, 0);
        assert_eq!(decoder.encoding_method, EncodingMethod::Unknown);
        let mut actual: Vec<Freq, 45> = table.iter().copied().collect();
        actual.sort_by_key(|f| f.frequency);
        assert_eq!(
            actual,
            [
                Freq {
                    frequency: 88_800_000,
                    freq_type: FreqType::SameProgram
                },
                Freq {
                    frequency: 89_000_000,
                    freq_type: FreqType::RegionalVariant
                },
                Freq {
                    frequency: 89_100_000,
                    freq_type: FreqType::RegionalVariant
                },
                Freq {
                    frequency: 89_300_000,
                    freq_type: FreqType::SameProgram
                },
                Freq {
                    frequency: 99_500_000,
                    freq_type: FreqType::SameProgram
                },
                Freq {
                    frequency: 100_900_000,
                    freq_type: FreqType::SameProgram
                },
                Freq {
                    frequency: 101_700_000,
                    freq_type: FreqType::SameProgram
                },
                Freq {
                    frequency: 102_600_000,
                    freq_type: FreqType::RegionalVariant
                },
                Freq {
                    frequency: 104_800_000,
                    freq_type: FreqType::RegionalVariant
                },
            ]
        );
    }
}
