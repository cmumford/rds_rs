use crate::types::Frequency;
use heapless::index_set::FnvIndexSet;

// No more than 25 alternative frequencies are transmitted according to
// 3.2.1.6.2. The impementation of FnvIndexSet requires the size be a
// power of 2, so do an additional check before inserting.
const MAX_ENTRIES: usize = 25;

/// Decoded table of alternative frequencies.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct AfTable {
    /// Tuned frequency (used in Method B)
    pub tuned_freq: Frequency,
    /// Alternative frequencies
    pub entries: FnvIndexSet<Frequency, 32>,
}

impl AfTable {
    pub fn insert(&mut self, freq: &Frequency) -> bool {
        if self.entries.len() == MAX_ENTRIES && !self.entries.contains(freq) {
            return false;
        }
        self.entries.insert(*freq).unwrap()
    }
}
