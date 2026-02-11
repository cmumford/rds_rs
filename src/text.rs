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

/// Convert an array of bytes from the code table to a string.
/// The string is limited to 64 bytes in length. Probably some
/// characters are multi-byte UTF, so the string is longer than.
/// 64 bytes.
pub fn rds_to_utf8_lossy(bytes: &[u8]) -> String<128> {
    bytes
        .iter()
        .map(|&b| match b {
            _ => TABLE2[b as usize],
        })
        .collect()
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_rt_convert_ascii() {
        let input_str =
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789:{}[]();!\"*+-'./%&";
        let input_bytes = input_str.as_bytes();
        let result = rds_to_utf8_lossy(input_bytes);
        assert_eq!(result.as_str(), input_str);
    }

    #[test]
    fn test_rt_convert_ebu_common_language() {
        let input_str = "$£";
        let result = rds_to_utf8_lossy(&[0b10101011, 0b10101010]);
        assert_eq!(result.as_str(), input_str);
    }
}
