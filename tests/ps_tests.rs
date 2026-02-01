use rds::{Content, Decoder, DiCodes, Group, ValidFields, rds_to_utf8_lossy};

#[cfg(test)]

mod tests {
    use rds::RdsData;

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
                .with_pi_code(true)
                .with_af(true)
                .with_pty(true)
                .with_ta_code(true)
                .with_tp_code(true)
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
        assert_eq!(rds_data.traffic.ta(), true);
        assert_eq!(rds_data.traffic.tp(), false);
        assert_eq!(rds_data.content, Content::Music);
        assert_eq!(rds_data.did_pty, DiCodes::new().with_artificial_head(true));

        // Verify block C values.
        // Verify that this 0A group, where block C hold a PI code, isn't
        // interpreted as a 0B group where they are alt-freqs.
        assert_eq!(rds_data.alternative_freqs.count, 0);

        // Verify block D values.
        assert_eq!(rds_to_utf8_lossy(&rds_data.ps.display), "    Ps  ");

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
        assert_eq!(rds_to_utf8_lossy(&rds_data.ps.display), "    Pst.");
        assert_eq!(
            rds_data.did_pty,
            DiCodes::new().with_artificial_head(true).with_stereo(true)
        );
    }
}
