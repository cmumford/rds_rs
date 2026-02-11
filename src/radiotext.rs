use crate::text::BLANK_CHAR;
use crate::text_prob::TextProb;
use modular_bitfield_msb::prelude::*;

pub const MAX_RADIOTEXT_LEN: usize = 64;
pub const END_OF_MESSAGE_CHAR: u8 = 0x0d;

/// Radiotext (RT) decoding state for one variant (A or B)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Radiotext {
    /// Final decoded text.
    pub display: [u8; MAX_RADIOTEXT_LEN],
    pvt: TextProb<MAX_RADIOTEXT_LEN>,
}

impl Default for Radiotext {
    fn default() -> Self {
        Self {
            display: [BLANK_CHAR; MAX_RADIOTEXT_LEN],
            pvt: TextProb::default(),
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
    pub a: Radiotext,         // RT A text.
    pub b: Radiotext,         // RT B text.
    pub decode_rt: RtVariant, // Which RT text currently being decoded.
}

impl Radiotext {
    pub fn reset(&mut self) {
        self.display.fill(BLANK_CHAR);
    }

    // Write (up to) a pair of two character tuples (up to four characters) into this
    // instance starting at character index `addr`. One or both of the two-character
    // pairs may be missing, and if so do nothing.
    pub fn update_rt_simple(&mut self, addr: usize, char_pairs: &[Option<[u8; 2]>]) {
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

        let eom = add_pair(&char_pairs[0]);
        if !eom && char_pairs.len() > 1 {
            add_pair(&char_pairs[1]);
        }
    }

    pub fn update_rt_advance(&mut self, addr: usize, char_pairs: &[Option<[u8; 2]>]) {
        let mut idx = addr;

        let mut add_pair = |pair: &Option<[u8; 2]>| {
            if pair.is_none() {
                return;
            }
            for ch in pair.unwrap() {
                self.pvt.update(idx, ch);
                idx += 1;
            }
        };

        add_pair(&char_pairs[0]);
        if char_pairs.len() > 1 {
            add_pair(&char_pairs[1]);
        }
        if !self.pvt.is_complete() {
            return;
        }
        self.display.copy_from_slice(&self.pvt.hi_prob);
    }

    pub fn bump_rt_validation_count(&mut self) {
        self.pvt.bump_rt_validation_count();
    }
}
