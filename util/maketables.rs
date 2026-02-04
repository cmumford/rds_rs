const START_FREQ: u32 = 87_600_000;

fn get_freq(idx: usize) -> Option<u32> {
    if idx < 1 || idx > 204 {
        return None;
    }
    Some(START_FREQ + (idx - 1) as u32 * 100_000)
}

fn create_freq_enum_name(idx: usize) -> String {
    let freq = get_freq(idx).unwrap();
    let n = freq / 100000;
    format!("Freq_{}", n)
}

fn main() {
    for idx in 0..256 {
        match get_freq(idx) {
            Some(freq) => println!(
                "{} = {} // {:.01} MHz",
                create_freq_enum_name(idx),
                idx,
                (freq as f32) / 1_000_000_f32
            ),
            None if idx == 0 => println!("Unused = {idx}, // Not to be used"),
            None if idx == 205 => println!("Filler_{idx} = {idx}, // Filler code."),
            None if (idx >= 206 && idx <= 223) || idx >= 251 => {
                println!("NotAssigned{idx} = {idx}, // Not to be used.")
            }
            None if idx >= 224 && idx <= 249 => {
                let cnt = idx - 224;
                println!("AfToFollow{cnt} = {idx}, // AF's to follow: {cnt}.")
            }
            None if idx == 250 => println!("LfMfFollows = {idx}, // An LF/MF frequency follows."),
            None => println!("Freq{idx} = {idx},"),
        }
    }
}
