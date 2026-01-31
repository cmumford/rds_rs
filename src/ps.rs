use crate::rds::RdsData;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PsData {
    pub display: [u8; 8],
    pub pvt: PsPrivate,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PsPrivate {
    pub hi_prob: [u8; 8],
    pub lo_prob: [u8; 8],
    pub hi_prob_cnt: [u8; 8],
}

const PS_VALIDATE_LIMIT: u8 = 2;

impl PsPrivate {
    fn update_ps_advanced_ch(&mut self, idx: usize, byte: u8) -> bool {
        let mut in_transition = false; // Indicates if the PS text is in transition.

        if self.hi_prob[idx] == byte {
            // The new byte matches the high probability byte.
            if self.hi_prob_cnt[idx] < PS_VALIDATE_LIMIT {
                self.hi_prob_cnt[idx] += 1;
            } else {
                // we have received this byte enough to max out our counter and push it
                // into the low probability array as well.
                self.hi_prob_cnt[idx] = PS_VALIDATE_LIMIT;
                self.lo_prob[idx] = byte;
            }
        } else if self.lo_prob[idx] == byte {
            // The new byte is a match with the low probability byte. Swap them, reset
            // the counter and flag the text as in transition. Note that the counter for
            // this character goes higher than the validation limit because it will get
            // knocked down later.
            if self.hi_prob_cnt[idx] >= PS_VALIDATE_LIMIT {
                in_transition = true;
                self.hi_prob_cnt[idx] = PS_VALIDATE_LIMIT + 1;
            } else {
                self.hi_prob_cnt[idx] = PS_VALIDATE_LIMIT;
            }
            self.lo_prob[idx] = self.hi_prob[idx];
            self.hi_prob[idx] = byte;
        } else if self.hi_prob_cnt[idx] == 0 {
            // The new byte is replacing an empty byte in the high probability array.
            self.hi_prob[idx] = byte;
            self.hi_prob_cnt[idx] = 1;
        } else {
            // The new byte doesn't match anything, put it in the low probability array.
            self.lo_prob[idx] = byte;
        }
        in_transition
    }

    /// Update the Program Service text in our buffers from the shadow registers.
    ///
    /// This implementation of the Program Service update attempts to display only
    /// complete messages for stations who rotate text through the PS field in
    /// violation of the RBDS standard as well as providing enhanced error detection.
    ///
    /// This function is from the Silicon Labs sample application.
    pub fn update_ps_advanced(&mut self, idx: usize, byte: u8) -> bool {
        let mut complete = true; // Indicates the PS text is ready to be displayed.

        if self.update_ps_advanced_ch(idx, byte) {
            // When the text is changing, decrement the count for all characters to
            // prevent displaying part of a message that is in transition.
            for count in self.hi_prob_cnt.iter_mut() {
                if *count > 1 {
                    *count -= 1;
                }
            }
        }

        // The PS text is incomplete if any character in the high probability array
        // has been seen fewer times than the validation limit.
        for count in self.hi_prob_cnt.iter_mut() {
            if *count < PS_VALIDATE_LIMIT {
                complete = false;
                break;
            }
        }

        complete
    }
}

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
    let complete = rds_data.ps.pvt.update_ps_advanced(char_idx, byte);
    if complete {
        // If the PS text in the high probability array is complete copy it to the
        // display array.
        rds_data
            .ps
            .display
            .copy_from_slice(&rds_data.ps.pvt.hi_prob);
    }
    complete
}
