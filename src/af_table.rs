use crate::types::Frequency;
use heapless::Vec;

/// Decoded table of alternative frequencies
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AltFreqTable {
    /// Tuned frequency (used in Method B)
    pub tuned_freq: Frequency,
    /// Alternative frequencies
    pub entries: Vec<Frequency, 25>,
}

impl AltFreqTable {
    pub fn insert_alt_freq(&mut self, freq: &Frequency) -> bool {
        if self.entries.contains(freq) {
            return true;
        }
        self.entries.push(*freq).is_ok()
    }
}
