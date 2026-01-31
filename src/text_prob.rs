#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TextProb<const N: usize> {
    pub hi_prob: [u8; 8],
    pub lo_prob: [u8; 8],
    pub hi_prob_cnt: [u8; 8],
}

const VALIDATE_LIMIT: u8 = 2;

impl<const N: usize> TextProb<N> {
    pub fn update(&mut self, idx: usize, byte: u8) -> bool {
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
        in_transition
    }
}
