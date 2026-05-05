use rds_rs::Clock;

#[cfg(test)]
mod tests {
    use super::*;

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
