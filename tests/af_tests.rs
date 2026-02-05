use rds::{Decoder, Group, ValidFields};

#[cfg(test)]

mod tests {
    use rds::RdsData;

    use super::*;

    // Test values from example A from RBDS spec 3.2.1.6.3 AF method A.
    #[test]
    fn test_af_decode_example_a() {
        // Group 0B is the same as 0A, except block C contains two frequencies instead
        // of the PIC. Don't retest the other values - already tested in the 0A test.

        let mut rds_data = RdsData::default();
        let mut decoder = Decoder::new(false);

        let mut decode_with_block_c = |block_c: u16| {
            // A test Block 0B decode
            //                  |pi_code|
            let block_a: u16 = 0x0000;
            //                  |code|v|t| pty |TA|MS|DI|Sa|
            let block_b: u16 = 0b0000_1_0_00110__1__1__1_10;
            //                  |C1|C2|
            let block_d: u16 = 0x50_73; // ['P', 's']

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
                    .with_pi_code(true)
                    .with_ps(true)
                    .with_pty(true)
                    .with_ta_code(true)
                    .with_tp_code(true)
            );
        };

        decode_with_block_c(0xE5_01); // five frequencies, F1 = 87.6 MHz.
        decode_with_block_c(0x02_03);
        decode_with_block_c(0x04_04);
    }
}
