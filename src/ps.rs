use crate::rds::RdsData;
use crate::text_prob::TextProb;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PsData {
    pub display: [u8; 8],
    pub pvt: TextProb<8>,
}

const PS_VALIDATE_LIMIT: u8 = 2;

pub fn update_ps_simple(char_idx: u8, current_ps_byte: u8, rds_data: &mut RdsData) {
    assert!(char_idx < 8);
    rds_data.ps.display[char_idx as usize] = current_ps_byte;
}

/// Update the Program Service text in our buffers from the shadow registers.
///
/// This implementation of the Program Service update attempts to display only
/// complete messages for stations who rotate text through the PS field in
/// violation of the RBDS standard as well as providing enhanced error detection.
///
/// This function is from the Silicon Labs sample application.
pub fn update_ps_advanced(char_idx: usize, byte: u8, rds_data: &mut RdsData) -> bool {
    if rds_data.ps.pvt.update(char_idx, byte) {
        // When the text is changing, decrement the count for all characters to
        // prevent displaying part of a message that is in transition.
        for count in rds_data.ps.pvt.hi_prob_cnt.iter_mut() {
            if *count > 1 {
                *count -= 1;
            }
        }
    }
    // The PS text is incomplete if any character in the high probability array
    // has been seen fewer times than the validation limit.
    for count in rds_data.ps.pvt.hi_prob_cnt.iter_mut() {
        if *count < PS_VALIDATE_LIMIT {
            return false;
        }
    }
    // If the PS text in the high probability array is complete copy it to the
    // display array.
    rds_data
        .ps
        .display
        .copy_from_slice(&rds_data.ps.pvt.hi_prob);
    true
}
