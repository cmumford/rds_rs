use crate::types::Frequency;

/// Decoded table of alternative frequencies
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AltFreqTable {
    /// Tuned frequency (used in Method B)
    pub tuned_freq: Frequency,
    /// Number of valid entries in `entries`
    pub count: u8,
    /// Alternative frequencies
    pub entries: [Frequency; 25],
}

impl AltFreqTable {
    fn find_af_freq_idx(&self, freq: &Frequency) -> i8 {
        for (i, entry) in self.entries.iter().enumerate() {
            if entry == freq {
                return i as i8;
            }
        }
        return -1;
    }

    fn freq_in_af_table(&self, freq: &Frequency) -> bool {
        return self.find_af_freq_idx(freq) != -1;
    }

    pub fn insert_alt_freq(&mut self, freq: &Frequency) -> bool {
        if self.count as usize >= self.entries.len() {
            // Array is full, do nothing (for now).
            return false;
        }

        if self.freq_in_af_table(freq) {
            return false;
        }

        self.entries[self.count as usize] = *freq;
        self.count += 1;
        true
    }
}
