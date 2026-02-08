use heapless::index_set::FnvIndexSet;

// Alternative frequencies encoded using method A are limited to 25
// frequencies. Method B can transmit more. The impementation of
// FnvIndexSet requires the size be a power of 2.
const MAX_NUM_ENTRIES: usize = 25;
const TABLE_SIZE: usize = MAX_NUM_ENTRIES.next_power_of_two();

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
    entries: FnvIndexSet<Freq, TABLE_SIZE>,
}

impl AfTable {
    pub fn add(&mut self, freq: &Freq) -> bool {
        if self.entries.len() == MAX_NUM_ENTRIES && !self.entries.contains(freq) {
            return false;
        }
        self.entries.insert(*freq).unwrap()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Freq> + '_ {
        self.entries.iter()
    }
}
