use modular_bitfield_msb::prelude::*;

pub const MAX_RADIOTEXT_LEN: usize = 64;
const END_OF_MESSAGE_CHAR: u8 = 0x0d;
const LINE_BREAK_CHAR: u8 = 0x0a;
const BLANK_CHAR: u8 = ' ' as u8;

// Code table from IEC 62106:1000 Figure E.1
// TODO: Complete this table.
const TABLE1: [[char; 16]; 16] = [
    [
        ' ', ' ', ' ', '0', '@', 'P', '॥', 'p', 'a', 'â', 'a', '°', 'A', 'Â', 'Ã', 'ã',
    ],
    [
        ' ', ' ', '!', '1', 'A', 'Q', 'a', 'q', '-', '-', '⍺', '-', '-', '-', '-', '-',
    ],
    [
        ' ', ' ', '\"', '2', 'B', 'R', 'b', 'r', '-', '-', '©', '-', '-', '-', '-', '-',
    ],
    [
        ' ', ' ', '#', '3', 'C', 'S', 'c', 's', '-', '-', '-', '-', '-', '-', '-', '-',
    ],
    [
        ' ', ' ', '�', '4', 'D', 'T', 'd', 't', '-', '-', '-', '±', '-', '-', '-', '-',
    ],
    [
        ' ', ' ', '%', '5', 'E', 'U', 'e', 'u', '-', '-', '-', '-', '-', '-', '-', '-',
    ],
    [
        ' ', ' ', '&', '6', 'F', 'V', 'f', 'v', '-', '-', '-', '-', '-', '-', '-', '-',
    ],
    [
        ' ', ' ', '\'', '7', 'G', 'W', 'g', 'w', '-', '-', '-', '-', '-', '-', '-', '-',
    ],
    [
        ' ', ' ', '(', '8', 'H', 'X', 'h', 'x', '-', '-', '-', 'µ', '-', '-', '-', '-',
    ],
    [
        ' ', ' ', ')', '9', 'I', 'Y', 'i', 'y', '-', '-', '-', '¿', '-', '-', '-', '-',
    ],
    [
        ' ', ' ', '*', ':', 'J', 'Z', 'j', 'z', '-', '-', '-', '÷', '-', '-', '-', '-',
    ],
    [
        ' ', ' ', '+', ';', 'K', '[', 'k', '{', '-', '-', '$', '°', '-', '-', '-', '-',
    ],
    [
        ' ', ' ', '�', '<', 'L', '\\', 'l', '|', '-', '-', '-', '¼', '-', '-', '-', '-',
    ],
    [
        ' ', ' ', '-', '=', 'M', ']', 'm', '}', '-', '-', '-', '½', '-', '-', '-', '-',
    ],
    [
        ' ', ' ', '.', '>', 'N', '-', 'n', '-', '-', '-', '-', '¾', '-', '-', '-', '-',
    ],
    [
        ' ', ' ', '/', '?', 'O', '_', 'o', ' ', '-', '-', '-', '-', 'L', '-', '-', '-',
    ],
];

/// Radiotext (RT) decoding state for one variant (A or B)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Radiotext {
    /// Final decoded text.
    pub display: [u8; MAX_RADIOTEXT_LEN],
    pvt: RadioTextPvt,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct RadioTextPvt {
    hi_prob: [u8; MAX_RADIOTEXT_LEN], // Temporary Radiotext (high probability).
    lo_prob: [u8; MAX_RADIOTEXT_LEN], // Temporary Radiotext (low probability).
    hi_prob_cnt: [u8; MAX_RADIOTEXT_LEN], // Hit count of high probability Radiotext.
}

impl Default for Radiotext {
    fn default() -> Self {
        let mut spaces = [0u8; MAX_RADIOTEXT_LEN];
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

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct RtData {
    pub a: Radiotext,         // RT A text.
    pub b: Radiotext,         // RT B text.
    pub decode_rt: RtVariant, // Which RT text currently being decoded.
}

pub fn rds_to_utf8_lossy(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&b| match b {
            LINE_BREAK_CHAR => ' ',
            _ => {
                let x: usize = (b & 0xf) as usize;
                let y: usize = (b >> 4) as usize;
                TABLE1[x][y]
            }
        })
        .collect()
}

impl Radiotext {
    pub fn reset(&mut self) {
        self.display.fill(BLANK_CHAR);
    }

    // Write (up to) a pair of two character tuples (up to four characters) into this
    // instance starting at character index `addr`. One or both of the two-character
    // pairs may be missing, and if so do nothing.
    pub fn update_rt_simple(&mut self, addr: usize, chars: &[Option<[u8; 2]>]) {
        let mut idx = addr;

        // Write two characters if provided.
        // Return true if the end of message character (0xD) is received.
        let mut add_pair = |pair: &Option<[u8; 2]>| -> bool {
            if pair.is_none() {
                return false;
            }
            for ch in pair.unwrap() {
                if ch == END_OF_MESSAGE_CHAR {
                    self.display[idx..].fill(BLANK_CHAR);
                    return true;
                }
                self.display[idx] = ch;
                idx += 1;
            }
            false
        };

        let eom = add_pair(&chars[0]);
        if !eom && chars.len() > 1 {
            add_pair(&chars[1]);
        }
    }

    /// The advanced implementation of the Radiotext update.
    ///
    /// This implementation of the Radiotext update attempts to further error
    /// correct the data by making sure that the data has been identical for
    /// multiple receptions of each byte.
    pub fn update_rt_advance(&mut self, addr: usize, byte: &[Option<[u8; 2]>]) {
        const RT_VALIDATE_LIMIT: u8 = 2;
        let mut text_changing = false; // Indicates if the Radiotext is changing.
        let mut idx = addr;

        let mut add_pair = |pair: &Option<[u8; 2]>| {
            if pair.is_none() {
                return;
            }
            for ch in pair.unwrap() {
                // The new byte matches the high probability byte.
                if self.pvt.hi_prob[idx] == ch {
                    if self.pvt.hi_prob_cnt[idx] < RT_VALIDATE_LIMIT {
                        self.pvt.hi_prob_cnt[idx] += 1;
                    } else {
                        // we have received this byte enough to max out our counter and push
                        // it into the low probability array as well.
                        self.pvt.hi_prob_cnt[idx] = RT_VALIDATE_LIMIT;
                        self.pvt.lo_prob[idx] = ch;
                    }
                } else if self.pvt.lo_prob[idx] == ch {
                    // The new byte is a match with the low probability byte. Swap them,
                    // reset the counter and flag the text as in transition. Note that the
                    // counter for this character goes higher than the validation limit
                    // because it will get knocked down later.
                    if self.pvt.hi_prob_cnt[idx] >= RT_VALIDATE_LIMIT {
                        text_changing = true;
                        self.pvt.hi_prob_cnt[idx] = RT_VALIDATE_LIMIT + 1;
                    } else {
                        self.pvt.hi_prob_cnt[idx] = RT_VALIDATE_LIMIT;
                    }
                    self.pvt.lo_prob[idx] = self.pvt.hi_prob[idx];
                    self.pvt.hi_prob[idx] = ch;
                } else if !self.pvt.hi_prob_cnt[idx] == 0 {
                    // The new byte is replacing an empty byte in the high proability array.
                    self.pvt.hi_prob[idx] = ch;
                    self.pvt.hi_prob_cnt[idx] = 1;
                } else {
                    // The new byte doesn't match anything, put it in the low probability
                    // array.
                    self.pvt.lo_prob[idx] = ch;
                }
                idx += 1;
            }
        };

        add_pair(&byte[0]);
        if byte.len() > 1 {
            add_pair(&byte[1]);
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
                self.pvt.hi_prob[i] = BLANK_CHAR;
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
