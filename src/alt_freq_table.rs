use heapless::index_set::FnvIndexSet;

// No more than 25 alternative frequencies are transmitted according to
// 3.2.1.6.2. The impementation of FnvIndexSet requires the size be a
// power of 2, so do an additional check before inserting.
const MAX_ENTRIES: usize = 25;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FreqType {
    #[default]
    SameProgram,
    RegionalVariant,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Freq {
    pub frequency: u32, // Frequency in Hz.
    pub freq_type: FreqType,
}

/// Decoded table of alternative frequencies.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct AfTable {
    pub entries: FnvIndexSet<Freq, 32>,
}

impl AfTable {
    pub fn add(&mut self, freq: &Freq) -> bool {
        if self.entries.len() == MAX_ENTRIES && !self.entries.contains(freq) {
            return false;
        }
        self.entries.insert(*freq).unwrap()
    }
}
