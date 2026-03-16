#![no_main]

use libfuzzer_sys::fuzz_target;
use rds::{Decoder, Group, RdsData};

fuzz_target!(|data: &[u8]| {
    if data.len() < 9 {
        return;
    }

    let has_a = (data[0] & 0b0001) != 0;
    let has_b = (data[0] & 0b0010) != 0;
    let has_c = (data[0] & 0b0100) != 0;
    let has_d = (data[0] & 0b1000) != 0;

    let payload = &data[1..9];

    let a = has_a.then(|| u16::from_le_bytes([payload[0], payload[1]]));
    let b = has_b.then(|| u16::from_le_bytes([payload[2], payload[3]]));
    let c = has_c.then(|| u16::from_le_bytes([payload[4], payload[5]]));
    let d = has_d.then(|| u16::from_le_bytes([payload[6], payload[7]]));

    let group = Group { a, b, c, d };

    let mut rds_data = RdsData::default();
    let mut decoder = Decoder::new(false);
    decoder.decode(&group, &mut rds_data);
});
