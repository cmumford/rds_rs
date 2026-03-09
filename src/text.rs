use heapless::String;

pub const LINE_BREAK_CHAR: u8 = 0x0a;
pub const BLANK_CHAR: u8 = ' ' as u8;

// Code table from IEC 62106:1000 Figure E.1
#[rustfmt::skip]
const TABLE2: [char; 256] = [
' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ' , ' ', ' ', '␊', ' ', '␌' , '␍', ' ', ' ',
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

/// Convert an array of bytes from the code table to a string. If the destination
/// string is too small then characters will be silently dropped. If desired
/// call rds_to_utf8_required_bytes() to determine the number of bytes that
/// the output string needs to be to contain all characters specified in `bytes`.
pub fn rds_to_utf8_lossy<const N: usize>(bytes: &[u8]) -> String<N> {
    let mut s = String::<N>::new();
    for &b in bytes {
        let ch = TABLE2[b as usize];
        if s.push(ch).is_err() {
            break;
        }
    }
    s
}

/// Returns the exact number of UTF-8 bytes needed to represent the RDS-decoded string,
/// **not** including a trailing null terminator (unless you explicitly want one).
///
/// This is useful when you want to pre-allocate a `String` or fixed-size buffer
/// without risking truncation or over-allocation.
pub fn rds_to_utf8_required_bytes(bytes: &[u8]) -> usize {
    bytes
        .iter()
        .map(|&b| {
            let ch = TABLE2[b as usize];
            ch.len_utf8() // returns 1, 2, 3, or 4
        })
        .sum()
}

pub fn is_whitespace_byte(b: u8) -> bool {
    b <= 32 || b == 0xff || b == 127
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_rt_convert_ascii() {
        const INPUT_STR: &str =
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789:{}[]();!\"*+-'./%&";
        const INPUT_LEN: usize = INPUT_STR.len();
        const INPUT_BYTES: &[u8] = INPUT_STR.as_bytes();
        let result = rds_to_utf8_lossy::<INPUT_LEN>(INPUT_BYTES);
        assert_eq!(result.as_str(), INPUT_STR);
    }

    #[test]
    fn test_rt_convert_drop_chars() {
        const INPUT_STR: &str = "12345678";
        const INPUT_BYTES: &[u8] = INPUT_STR.as_bytes();
        let result = rds_to_utf8_lossy::<4>(INPUT_BYTES);
        assert_eq!(result.as_str(), "1234");
    }

    #[test]
    fn test_rt_convert_ebu_common_language() {
        let result = rds_to_utf8_lossy::<3>(&[0b10101011, 0b10101010]);
        assert_eq!(result.as_str(), "$£");
    }

    #[test]
    fn test_rds_to_utf8_required_bytes_ascii() {
        assert_eq!(rds_to_utf8_required_bytes("ab".as_bytes()), 2);
    }

    #[test]
    fn test_rds_to_utf8_required_bytes_ebu_common_language() {
        // [0b10101011, 0b10101010] → "$£"
        assert_eq!(rds_to_utf8_required_bytes(&[0b10101011, 0b10101010]), 3);
    }

    #[test]
    fn test_rds_to_utf8_required_bytes_empty() {
        assert_eq!(rds_to_utf8_required_bytes(&[]), 0);
    }

    #[test]
    fn test_is_whitespace_byte() {
        assert_eq!(is_whitespace_byte(0), true);
        assert_eq!(is_whitespace_byte(BLANK_CHAR), true);
        assert_eq!(is_whitespace_byte(127), true); // DEL character
        assert_eq!(is_whitespace_byte(255), true);

        assert_eq!(is_whitespace_byte(b'A'), false);
        assert_eq!(is_whitespace_byte(128), false);
    }
}
