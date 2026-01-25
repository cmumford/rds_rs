use modular_bitfield_msb::prelude::*;

use crate::types::Group;

/// Radiotext (RT) decoding state for one variant (A or B)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Radiotext {
    /// Final decoded text (64 bytes)
    pub display: [u8; 64],
    pvt: RadioTextPvt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RadioTextPvt {
    hi_prob: [u8; 64],     // Temporary Radiotext (high probability).
    lo_prob: [u8; 64],     // Temporary Radiotext (low probability).
    hi_prob_cnt: [u8; 64], // Hit count of high probability Radiotext.
}

impl Default for Radiotext {
    fn default() -> Self {
        let mut spaces = [0u8; 64];
        spaces.fill(b' ');

        Self {
            display: spaces,
            pvt: RadioTextPvt {
                hi_prob: spaces,
                lo_prob: spaces,
                hi_prob_cnt: spaces,
            },
        }
    }
}

/// Which RT variant is currently being decoded.
#[derive(BitfieldSpecifier, Debug, Default, Clone, Copy, PartialEq, Eq)]
#[bits = 1]
pub enum RtVariant {
    #[default]
    A,
    B,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RtData {
    pub a: Radiotext,
    pub b: Radiotext,
    pub current_variant: RtVariant,
}

impl Radiotext {
    pub fn update_rt_simple(&mut self, group: &Group, count: usize, addr: usize, chars: &[u8]) {
        for i in 0..count {
            // Choose the appropriate block. Count > 2 check is necessary for 2B groups.
            if (i < 2) && (count > 2) {
                if group.c.is_none() {
                    continue;
                }
            } else {
                if group.d.is_none() {
                    continue;
                }
            }

            // Store the data in our temporary array.
            self.display[addr + i] = chars[i];
            if chars[i as usize] == 0x0d {
                // The end of message character has been received.
                // Wipe out the rest of the text.
                for j in (addr + i + 1)..self.display.len() {
                    self.display[j] = 0;
                }
                break;
            }
        }

        // Any null character before this should become a space.
        for i in 0..addr {
            if self.display[i] == 0 {
                self.display[i] = ' ' as u8;
            }
        }
    }

    /// The advanced implementation of the Radiotext update.
    ///
    /// This implementation of the Radiotext update attempts to further error
    /// correct the data by making sure that the data has been identical for
    /// multiple receptions of each byte.
    pub fn update_rt_advance(&mut self, group: &Group, count: usize, addr: usize, byte: &mut [u8]) {
        const RT_VALIDATE_LIMIT: u8 = 2;

        let mut text_changing = false; // Indicates if the Radiotext is changing.

        for i in 0..count {
            // Choose the appropriate block. Count > 2 check is necessary for 2B groups.
            if (i < 2) && (count > 2) {
                if group.c.is_none() {
                    continue;
                }
            } else {
                if group.d.is_none() {
                    continue;
                }
            }
            if byte[i] == 0 {
                byte[i] = ' ' as u8; // translate nulls to spaces.
            }

            // The new byte matches the high probability byte.
            if self.pvt.hi_prob[addr + i] == byte[i] {
                if self.pvt.hi_prob_cnt[addr + i] < RT_VALIDATE_LIMIT {
                    self.pvt.hi_prob_cnt[addr + i] += 1;
                } else {
                    // we have received this byte enough to max out our counter and push
                    // it into the low probability array as well.
                    self.pvt.hi_prob_cnt[addr + i] = RT_VALIDATE_LIMIT;
                    self.pvt.lo_prob[addr + i] = byte[i];
                }
            } else if self.pvt.lo_prob[addr + i] == byte[i] {
                // The new byte is a match with the low probability byte. Swap them,
                // reset the counter and flag the text as in transition. Note that the
                // counter for this character goes higher than the validation limit
                // because it will get knocked down later.
                if self.pvt.hi_prob_cnt[addr + i] >= RT_VALIDATE_LIMIT {
                    text_changing = true;
                    self.pvt.hi_prob_cnt[addr + i] = RT_VALIDATE_LIMIT + 1;
                } else {
                    self.pvt.hi_prob_cnt[addr + i] = RT_VALIDATE_LIMIT;
                }
                self.pvt.lo_prob[addr + i] = self.pvt.hi_prob[addr + i];
                self.pvt.hi_prob[addr + i] = byte[i];
            } else if !self.pvt.hi_prob_cnt[addr + i] == 0 {
                // The new byte is replacing an empty byte in the high proability array.
                self.pvt.hi_prob[addr + i] = byte[i];
                self.pvt.hi_prob_cnt[addr + i] = 1;
            } else {
                // The new byte doesn't match anything, put it in the low probability
                // array.
                self.pvt.lo_prob[addr + i] = byte[i];
            }
        }

        if !text_changing {
            return;
        }

        // When the text is changing, decrement the count for all characters to
        // prevent displaying part of a message that is in transition.
        for i in 0..self.pvt.hi_prob_cnt.len() {
            if self.pvt.hi_prob_cnt[i] > 1 {
                self.pvt.hi_prob_cnt[i] -= 1;
            }
        }
    }

    pub fn bump_rt_validation_count(&mut self) {
        for i in 0..self.pvt.hi_prob_cnt.len() {
            if self.pvt.hi_prob[i] == 0 {
                self.pvt.hi_prob[i] = ' ' as u8;
                self.pvt.hi_prob_cnt[i] += 1;
            }
            for i in 0..self.pvt.hi_prob_cnt.len() {
                self.pvt.hi_prob_cnt[i] += 1;
            }
        }
        // Wipe out the cached text.
        self.pvt.hi_prob_cnt.fill(0);
        self.pvt.hi_prob.fill(0);
        self.pvt.lo_prob.fill(0);
    }
}
