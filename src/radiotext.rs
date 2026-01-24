use modular_bitfield_msb::prelude::*;

use crate::types::Group;

/// Radiotext (RT) decoding state for one variant (A or B)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Radiotext {
    /// Final decoded text (64 bytes)
    pub display: [u8; 64],
}

impl Default for Radiotext {
    fn default() -> Self {
        let mut display = [0u8; 64];
        display.fill(b' ');

        Self { display }
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

    pub fn update_rt_advance(&mut self, group: &Group, count: u8, addr: u8, chars: &[u8]) {}

    pub fn bump_rt_validation_count(&mut self) {}
}
