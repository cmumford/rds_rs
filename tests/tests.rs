use rds::{Clock, rds_to_utf8_lossy};

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

    #[test]
    fn test_clock_date() {
        // These values from RBDS specification Annex G.
        let clock = Clock {
            mjd: 45218,
            hour: 0,
            minute: 0,
            utc_offset_half_hours: 0,
        };
        assert_eq!(1982, clock.year());
        assert_eq!(9, clock.month());
        assert_eq!(6, clock.day());
    }
}
