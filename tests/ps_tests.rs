use rds_rs::{
    Content, Decoder, DiCodes, Group, PS_TEXT_LEN, ProgramType, ValidFields, rds_to_utf8_lossy,
};

const PS_LEN: usize = PS_TEXT_LEN + 2;

#[allow(clippy::unusual_byte_groupings)]
#[cfg(test)]
mod tests {
    use rds_rs::RdsData;

    use super::*;

    #[test]
    fn test_block_0a_decode() {
        // A test Block 0A decode
        //                  |pi_code|
        let block_a: u16 = 0xaf56;
        //                  |code|v|t| pty |TA|MS|DI|Sa|
        let block_b: u16 = 0b0000_0_0_00110__1__1__1_10; // Second text segment.
        //                  |pi_code|
        let block_c: u16 = 0xaf56;
        //                  |C1|C2|
        let block_d: u16 = 0x50_73; // ['P', 's']

        let mut rds_data = RdsData::default();
        let mut decoder = Decoder::new(false);

        let valid = decoder.decode(
            &Group {
                a: Some(block_a),
                b: Some(block_b),
                c: Some(block_c),
                d: Some(block_d),
            },
            &mut rds_data,
        );
        assert_eq!(
            valid,
            ValidFields::new()
                .with_pi(true)
                .with_pty(true)
                .with_ta(true)
                .with_tp(true)
                .with_ms(true)
                .with_ps(true)
        );
        // Verify block A value.
        assert_eq!(valid, rds_data.valid);
        assert_eq!(rds_data.program_information.country_code(), 0xa);
        assert_eq!(rds_data.program_information.program_type(), 0xf);
        assert_eq!(
            rds_data.program_information.program_reference_number(),
            0x56
        );
        // Verify block B values.
        assert!(rds_data.tn.traffic.ta());
        assert!(!rds_data.tn.traffic.tp());
        assert_eq!(rds_data.content, Content::Music);
        assert_eq!(rds_data.tn.program_type, ProgramType::ClassicRock);
        assert_eq!(rds_data.did_pty, DiCodes::new().with_artificial_head(true));

        // Verify block C values.
        // Verify that this 0A group, where block C hold a PI code, isn't
        // interpreted as a 0B group where they are alt-freqs.
        assert_eq!(rds_data.alt_freqs.iter().count(), 0);

        // Verify block D values.
        assert_eq!(
            rds_to_utf8_lossy::<PS_LEN>(&rds_data.tn.ps.display),
            "    Ps  "
        );

        // Now a second (different) block D with two more characters.
        //                   |code|v|t| pty |TA|MS|DI|Sa|
        let block_b2: u16 = 0b0000_0_0_00110__1__1__1_11; // Second text segment.
        let _ = decoder.decode(
            &Group {
                a: Some(block_a),
                b: Some(block_b2),
                c: Some(block_c),
                d: Some(0x74_2E), // ['t', '.']
            },
            &mut rds_data,
        );
        assert_eq!(
            rds_to_utf8_lossy::<PS_LEN>(&rds_data.tn.ps.display),
            "    Pst."
        );
        assert_eq!(
            rds_data.did_pty,
            DiCodes::new().with_artificial_head(true).with_stereo(true)
        );
    }

    #[test]
    fn test_block_0b_decode() {
        // A test Block 0B decode
        //                  |pi_code|
        let block_a: u16 = 0x0000; // Will verify PI parsed from block C.
        //                  |code|v|t| pty |TA|MS|DI|Sa|
        let block_b: u16 = 0b0000_1_0_00110__1__1__1_10; // Second text segment.
        //                  |Nu|F1|
        let block_c: u16 = 0xE1_01; // 1 freq: 0x01 = 87.6 MHz
        //                  |C1|C2|
        let block_d: u16 = 0x50_73; // ['P', 's']

        // Group 0B is the same as 0A, except block C contains two frequencies instead
        // of the PIC. Don't retest the other values - already tested in the 0A test.

        let mut rds_data = RdsData::default();
        let mut decoder = Decoder::new(false);

        assert_eq!(
            decoder.decode(
                &Group {
                    a: Some(block_a),
                    b: Some(block_b),
                    c: Some(block_c),
                    d: Some(block_d),
                },
                &mut rds_data,
            ),
            ValidFields::new()
                .with_af(true)
                .with_af(true)
                .with_ms(true)
                .with_pi(true)
                .with_ps(true)
                .with_pty(true)
                .with_ta(true)
                .with_tp(true)
        );

        // Alternative frequency decoding is tested in `af_tests.rs` and unit tests.

        assert_eq!(
            rds_to_utf8_lossy::<PS_LEN>(&rds_data.tn.ps.display),
            "    Ps  "
        );
    }
}
