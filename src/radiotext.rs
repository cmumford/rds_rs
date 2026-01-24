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

impl RtData {
    pub fn update_rt_simple(&mut self, group: &Group, count: u8, addr: u8, chars: &[u8]) {}

    pub fn update_rt_advance(&mut self, group: &Group, count: u8, addr: u8, chars: &[u8]) {}

    pub fn bump_rt_validation_count(&mut self) {}
}
