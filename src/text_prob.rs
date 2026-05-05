const VALIDATE_LIMIT: u8 = 2;
use crate::text::BLANK_CHAR;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextProb<const N: usize> {
    pub hi_prob: [u8; N],
    pub lo_prob: [u8; N],
    pub hi_prob_cnt: [u8; N],
}

impl<const N: usize> TextProb<N> {
    pub const fn new() -> Self {
        Self {
            hi_prob: [0; N],
            lo_prob: [0; N],
            hi_prob_cnt: [0; N],
        }
    }

    pub fn default() -> Self {
        Self::new()
    }

    pub fn update(&mut self, idx: usize, byte: u8) {
        let mut in_transition = false; // Indicates if the text is in transition.

        if self.hi_prob[idx] == byte {
            // The new byte matches the high probability byte.
            if self.hi_prob_cnt[idx] < VALIDATE_LIMIT {
                self.hi_prob_cnt[idx] += 1;
            } else {
                // we have received this byte enough to max out our counter and push it
                // into the low probability array as well.
                self.hi_prob_cnt[idx] = VALIDATE_LIMIT;
                self.lo_prob[idx] = byte;
            }
        } else if self.lo_prob[idx] == byte {
            // The new byte is a match with the low probability byte. Swap them, reset
            // the counter and flag the text as in transition. Note that the counter for
            // this character goes higher than the validation limit because it will get
            // knocked down later.
            if self.hi_prob_cnt[idx] >= VALIDATE_LIMIT {
                in_transition = true;
                self.hi_prob_cnt[idx] = VALIDATE_LIMIT + 1;
            } else {
                self.hi_prob_cnt[idx] = VALIDATE_LIMIT;
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
        if in_transition {
            // When the text is changing, decrement the count for all characters to
            // prevent displaying part of a message that is in transition.
            for count in &mut self.hi_prob_cnt {
                if *count > 1 {
                    *count -= 1;
                }
            }
        }
    }

    pub fn is_complete(&self) -> bool {
        // Text is incomplete if any character in the high probability array
        // has been seen fewer times than the validation limit.
        for count in &self.hi_prob_cnt {
            if *count < VALIDATE_LIMIT {
                return false;
            }
        }
        true
    }

    pub fn bump_rt_validation_count(&mut self) {
        for i in 0..self.hi_prob_cnt.len() {
            if self.hi_prob[i] == 0 {
                self.hi_prob[i] = BLANK_CHAR;
                self.hi_prob_cnt[i] += 1;
            }
            for i in 0..self.hi_prob_cnt.len() {
                self.hi_prob_cnt[i] += 1;
            }
        }
        // Wipe out the cached text.
        self.hi_prob_cnt.fill(0);
        self.hi_prob.fill(0);
        self.lo_prob.fill(0);
    }
}

impl<const N: usize> Default for TextProb<N> {
    fn default() -> Self {
        Self::new()
    }
}
