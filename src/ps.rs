use crate::text_prob::TextProb;

pub const PS_TEXT_LEN: usize = 8;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PsData {
    pub display: [u8; PS_TEXT_LEN],
    pub pvt: TextProb<PS_TEXT_LEN>,
}

impl PsData {
    pub fn update_simple(&mut self, char_idx: usize, bytes: [u8; 2]) {
        assert!(char_idx + 2 <= self.display.len(), "char_idx OOB");
        self.display[char_idx..char_idx + 2].copy_from_slice(&bytes);
    }

    /// Update the Program Service text in our buffers from the shadow registers.
    ///
    /// This implementation of the Program Service update attempts to display only
    /// complete messages for stations who rotate text through the PS field in
    /// violation of the RBDS standard as well as providing enhanced error detection.
    ///
    /// This function is from the Silicon Labs sample application.
    pub fn update_advanced(&mut self, char_idx: usize, byte: u8) -> bool {
        self.pvt.update(char_idx, byte);

        if !self.pvt.is_complete() {
            return false;
        }
        // If the PS text in the high probability array is complete copy it to the
        // display array.
        self.display.copy_from_slice(&self.pvt.hi_prob);
        true
    }
}
