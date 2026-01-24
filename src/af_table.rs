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
