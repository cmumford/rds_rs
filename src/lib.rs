mod af_codes;
mod af_decode_table;
mod af_table;
mod af_table_group;
mod alt_freq_decoder;
mod alt_freq_table;
mod decoder;
mod eon;
mod oda;
mod ps;
mod ptyn;
mod radiotext;
mod rds;
mod text_prob;
mod types;

pub use decoder::Decoder;
pub use radiotext::{BLANK_CHAR, LINE_BREAK_CHAR, MAX_RADIOTEXT_LEN, RtVariant, rds_to_utf8_lossy};
pub use rds::RdsData;
pub use types::{
    AltFreqAttribute, AltFreqEncoding, Band, Clock, Content, DiCodes, Frequency, Group, GroupType,
    GroupVersion, ProgramType, ValidFields,
};

#[cfg(test)]

mod tests {
    use crate::alt_freq_decoder::get_lf_mf_frequency;

    #[test]
    fn test_get_lf_mf_frequency() {
        assert_eq!(get_lf_mf_frequency(1), 153_000);
        assert_eq!(get_lf_mf_frequency(15), 279_000);
        assert_eq!(get_lf_mf_frequency(16), 531_000);
        assert_eq!(get_lf_mf_frequency(135), 1_602_000);
    }
}
