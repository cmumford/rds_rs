use modular_bitfield_msb::prelude::*;

pub const MAX_RADIOTEXT_LEN: usize = 64;
const END_OF_MESSAGE_CHAR: u8 = 0x0d;
pub const LINE_BREAK_CHAR: u8 = 0x0a;
pub const BLANK_CHAR: u8 = ' ' as u8;
const RT_VALIDATE_LIMIT: u8 = 2;

// Code table from IEC 62106:1000 Figure E.1
#[rustfmt::skip]
const TABLE2: [char; 256] = [
'␀', ' ', ' ', ' ', ' ', ' ', ' ', ' ' , ' ', ' ', '␊', ' ', '␌' , '␍', ' ', ' ',
' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ' , ' ', ' ', ' ', '␛', ' ' , ' ', ' ', ' ',
' ', '!', '"', '#', '¤', '%', '&', '\'', '(', ')', '*', '+', ',' , '-', '.', '/',
'0', '1', '2', '3', '4', '5', '6', '7' , '8', '9', ':', ';', '<' , '=', '>', '?',
'@', 'A', 'B', 'C', 'D', 'E', 'F', 'G' , 'H', 'I', 'J', 'K', 'L' , 'M', 'N', 'O',
'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W' , 'X', 'Y', 'Z', '[', '\\', ']', '―', '_',
'║', 'a', 'b', 'c', 'd', 'e', 'f', 'g' , 'h', 'i', 'j', 'k', 'l' , 'm', 'n', 'o',
'p', 'q', 'r', 's', 't', 'u', 'v', 'w' , 'x', 'y', 'z', '{', '|' , '}', '¯', '␡',
'á', 'à', 'é', 'è', 'í', 'ì', 'ó', 'ò' , 'ú', 'ù', 'Ñ', 'Ç', 'Ş' , 'ß', '¡', 'Ĳ',
'â', 'ä', 'ê', 'ë', 'î', 'ï', 'ô', 'ö' , 'û', 'ü', 'ñ', 'ç', 'ş' , 'ğ', 'ı', 'ĳ',
'ª', 'α', '©', '‰', 'Ğ', 'ě', 'ň', 'ő' , 'π', '€', '£', '$', '←' , '↑', '→', '↓',
'º', '¹', '²', '³', '±', 'İ', 'ń', 'ű' , 'µ', '¿', '÷', '°', '¼' , '½', '¾', '§',
'Á', 'À', 'É', 'È', 'Í', 'Ì', 'Ó', 'Ò' , 'Ú', 'Ù', 'Ř', 'Č', 'Š' , 'Ž', 'Ð', 'Ŀ',
'Â', 'Ä', 'Ê', 'Ë', 'Î', 'Ï', 'Ô', 'Ö' , 'Û', 'Ü', 'ř', 'č', 'š' , 'ž', 'đ', 'ŀ',
'Ã', 'Å', 'Æ', 'Œ', 'ŷ', 'Ý', 'Õ', 'Ø' , 'Þ', 'Ŋ', 'Ŕ', 'Ć', 'Ś' , 'Ź', 'Ŧ', 'ð',
'ã', 'å', 'æ', 'œ', 'ŵ', 'ý', 'õ', 'ø' , 'þ', 'ŋ', 'ŕ', 'ć', 'ś' , 'ź', 'ŧ', ' '];

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

impl Default for RadioTextPvt {
    fn default() -> Self {
        Self {
            hi_prob: [BLANK_CHAR; MAX_RADIOTEXT_LEN],
            lo_prob: [BLANK_CHAR; MAX_RADIOTEXT_LEN],
            hi_prob_cnt: [BLANK_CHAR; MAX_RADIOTEXT_LEN],
        }
    }
}

impl Default for Radiotext {
    fn default() -> Self {
        Self {
            display: [BLANK_CHAR; MAX_RADIOTEXT_LEN],
            pvt: RadioTextPvt::default(),
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
            _ => TABLE2[b as usize],
        })
        .collect()
}

impl RadioTextPvt {
    fn update_rt_advance_ch(&mut self, idx: usize, byte: u8) -> bool {
        let mut text_changing = false;
        // The new byte matches the high probability byte.
        if self.hi_prob[idx] == byte {
            if self.hi_prob_cnt[idx] < RT_VALIDATE_LIMIT {
                self.hi_prob_cnt[idx] += 1;
            } else {
                // we have received this byte enough to max out our counter and push
                // it into the low probability array as well.
                self.hi_prob_cnt[idx] = RT_VALIDATE_LIMIT;
                self.lo_prob[idx] = byte;
            }
        } else if self.lo_prob[idx] == byte {
            // The new byte is a match with the low probability byte. Swap them,
            // reset the counter and flag the text as in transition. Note that the
            // counter for this character goes higher than the validation limit
            // because it will get knocked down later.
            if self.hi_prob_cnt[idx] >= RT_VALIDATE_LIMIT {
                text_changing = true;
                self.hi_prob_cnt[idx] = RT_VALIDATE_LIMIT + 1;
            } else {
                self.hi_prob_cnt[idx] = RT_VALIDATE_LIMIT;
            }
            self.lo_prob[idx] = self.hi_prob[idx];
            self.hi_prob[idx] = byte;
        } else if !self.hi_prob_cnt[idx] == 0 {
            // The new byte is replacing an empty byte in the high proability array.
            self.hi_prob[idx] = byte;
            self.hi_prob_cnt[idx] = 1;
        } else {
            // The new byte doesn't match anything, put it in the low probability
            // array.
            self.lo_prob[idx] = byte;
        }
        text_changing
    }

    /// The advanced implementation of the Radiotext update.
    ///
    /// This implementation of the Radiotext update attempts to further error
    /// correct the data by making sure that the data has been identical for
    /// multiple receptions of each byte.
    pub fn update_rt_advance(&mut self, addr: usize, byte: &[Option<[u8; 2]>]) {
        let mut text_changing = false; // Indicates if the Radiotext is changing.
        let mut idx = addr;

        let mut add_pair = |pair: &Option<[u8; 2]>| {
            if pair.is_none() {
                return;
            }
            for ch in pair.unwrap() {
                if self.update_rt_advance_ch(idx, ch) {
                    text_changing = true;
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
        for i in 0..self.hi_prob_cnt.len() {
            if self.hi_prob_cnt[i] > 1 {
                self.hi_prob_cnt[i] -= 1;
            }
        }
    }
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

    pub fn update_rt_advance(&mut self, addr: usize, byte: &[Option<[u8; 2]>]) {
        self.pvt.update_rt_advance(addr, byte)
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
