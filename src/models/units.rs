use serde::{Deserialize, Serialize};
use std::fmt;
/// Weight stored as hectograms: 1 kg = 10 hg
pub const HG_PER_KG: f64 = 10.0;
/// Distance stored as meters: 1 km = 1000 m
pub const M_PER_KM: f64 = 1000.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Weight(pub u16);

impl fmt::Display for Weight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f64::from(self.0) % HG_PER_KG < f64::EPSILON {
            write!(f, "{} kg", f64::from(self.0) / HG_PER_KG)
        } else {
            write!(f, "{:.1} kg", f64::from(self.0) / HG_PER_KG)
        }
    }
}

/// Distance stored as meters. 1 km = 1000 m.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Distance(pub u32);

impl fmt::Display for Distance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f64::from(self.0) >= M_PER_KM {
            if f64::from(self.0) % M_PER_KM < f64::EPSILON {
                write!(f, "{} km", f64::from(self.0) / M_PER_KM)
            } else {
                write!(f, "{:.2} km", f64::from(self.0) / M_PER_KM)
            }
        } else {
            write!(f, "{} m", self.0)
        }
    }
}

/// Parse a user-entered kg string into a Weight (hectograms).
pub fn parse_weight_kg(input: &str) -> Option<Weight> {
    let val: f64 = input.parse().ok()?;
    if !val.is_finite() || val <= 0.0 {
        return None;
    }
    let hg = (val * HG_PER_KG).round();
    if hg < 1.0 || hg > f64::from(u16::MAX) {
        return None;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    Some(Weight(hg as u16))
}
/// Parse a user-entered duration string (seconds, MM:SS, or HH:MM:SS) into seconds.
#[must_use]
pub fn parse_duration_seconds(input: &str) -> Option<u64> {
    let s = input.trim();
    if s.is_empty() {
        return None;
    }
    let parts: Vec<&str> = s.split(':').collect();
    match parts.as_slice() {
        [secs] => secs.parse::<u64>().ok(),
        [mins, secs] => {
            let m: u64 = mins.parse().ok()?;
            let s: u64 = secs.parse().ok()?;
            if s >= 60 {
                return None;
            }
            Some(m * 60 + s)
        }
        [hours, mins, secs] => {
            let h: u64 = hours.parse().ok()?;
            let m: u64 = mins.parse().ok()?;
            let s: u64 = secs.parse().ok()?;
            if m >= 60 || s >= 60 {
                return None;
            }
            Some(h * 3600 + m * 60 + s)
        }
        _ => None,
    }
}
/// Parse a user-entered km string into a Distance (meters).
pub fn parse_distance_km(input: &str) -> Option<Distance> {
    let val: f64 = input.parse().ok()?;
    if !val.is_finite() || val <= 0.0 {
        return None;
    }
    let m = (val * M_PER_KM).round();
    if m < 1.0 || m > f64::from(u32::MAX) {
        return None;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    Some(Distance(m as u32))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn weight_display_whole_kg() {
        assert_eq!(Weight(10).to_string(), "1 kg");
        assert_eq!(Weight(20).to_string(), "2 kg");
        assert_eq!(Weight(1000).to_string(), "100 kg");
    }
    #[test]
    fn weight_display_fractional_kg() {
        assert_eq!(Weight(15).to_string(), "1.5 kg");
        assert_eq!(Weight(25).to_string(), "2.5 kg");
        assert_eq!(Weight(1).to_string(), "0.1 kg");
    }
    #[test]
    fn weight_display_zero() {
        assert_eq!(Weight(0).to_string(), "0 kg");
    }
    #[test]
    fn parse_weight_kg_valid() {
        assert_eq!(parse_weight_kg("1"), Some(Weight(10)));
        assert_eq!(parse_weight_kg("1.5"), Some(Weight(15)));
        assert_eq!(parse_weight_kg("100"), Some(Weight(1000)));
        assert_eq!(parse_weight_kg("0.1"), Some(Weight(1)));
    }
    #[test]
    fn parse_weight_kg_invalid() {
        assert_eq!(parse_weight_kg(""), None);
        assert_eq!(parse_weight_kg("abc"), None);
        assert_eq!(parse_weight_kg("-1"), None);
        assert_eq!(parse_weight_kg("0"), None);
        assert_eq!(parse_weight_kg("nan"), None);
    }
    #[test]
    fn distance_display_metres() {
        assert_eq!(Distance(0).to_string(), "0 m");
        assert_eq!(Distance(500).to_string(), "500 m");
        assert_eq!(Distance(999).to_string(), "999 m");
    }
    #[test]
    fn distance_display_whole_km() {
        assert_eq!(Distance(1000).to_string(), "1 km");
        assert_eq!(Distance(5000).to_string(), "5 km");
    }
    #[test]
    fn distance_display_fractional_km() {
        assert_eq!(Distance(1500).to_string(), "1.50 km");
        assert_eq!(Distance(2750).to_string(), "2.75 km");
    }
    #[test]
    fn parse_distance_km_valid() {
        assert_eq!(parse_distance_km("1"), Some(Distance(1000)));
        assert_eq!(parse_distance_km("0.5"), Some(Distance(500)));
        assert_eq!(parse_distance_km("10"), Some(Distance(10000)));
    }
    #[test]
    fn parse_distance_km_invalid() {
        assert_eq!(parse_distance_km(""), None);
        assert_eq!(parse_distance_km("abc"), None);
        assert_eq!(parse_distance_km("-1"), None);
        assert_eq!(parse_distance_km("0"), None);
    }
    #[test]
    fn parse_weight_kg_large_value_clamped() {
        assert_eq!(parse_weight_kg("6553.6"), None);
        assert_eq!(parse_weight_kg("6553.5"), Some(Weight(65535)));
    }
    #[test]
    fn parse_distance_km_large_value_clamped() {
        assert!(parse_distance_km("100").is_some());
        assert_eq!(parse_distance_km("-1"), None);
    }
    #[test]
    fn parse_weight_kg_nan_and_infinity() {
        assert_eq!(parse_weight_kg("NaN"), None);
        assert_eq!(parse_weight_kg("inf"), None);
        assert_eq!(parse_weight_kg("-inf"), None);
        assert_eq!(parse_weight_kg("Infinity"), None);
    }
    #[test]
    fn parse_distance_km_nan_and_infinity() {
        assert_eq!(parse_distance_km("NaN"), None);
        assert_eq!(parse_distance_km("inf"), None);
        assert_eq!(parse_distance_km("-inf"), None);
        assert_eq!(parse_distance_km("Infinity"), None);
    }
    #[test]
    fn parse_distance_km_overflow_u32_rejected() {
        let too_large = format!("{}", (f64::from(u32::MAX) / 1000.0) + 1.0);
        assert_eq!(parse_distance_km(&too_large), None);
    }
    #[test]
    fn parse_duration_seconds_empty_returns_none() {
        assert_eq!(parse_duration_seconds(""), None);
        assert_eq!(parse_duration_seconds("   "), None);
    }
    #[test]
    fn parse_duration_seconds_plain_seconds() {
        assert_eq!(parse_duration_seconds("0"), Some(0));
        assert_eq!(parse_duration_seconds("30"), Some(30));
        assert_eq!(parse_duration_seconds("90"), Some(90));
    }
    #[test]
    fn parse_duration_seconds_mm_ss_format() {
        assert_eq!(parse_duration_seconds("1:30"), Some(90));
        assert_eq!(parse_duration_seconds("0:00"), Some(0));
        assert_eq!(parse_duration_seconds("59:59"), Some(3599));
    }
    #[test]
    fn parse_duration_seconds_mm_ss_invalid_secs() {
        assert_eq!(parse_duration_seconds("1:60"), None);
        assert_eq!(parse_duration_seconds("0:99"), None);
    }
    #[test]
    fn parse_duration_seconds_hh_mm_ss_format() {
        assert_eq!(parse_duration_seconds("1:00:00"), Some(3600));
        assert_eq!(parse_duration_seconds("1:01:01"), Some(3661));
        assert_eq!(parse_duration_seconds("0:59:59"), Some(3599));
    }
    #[test]
    fn parse_duration_seconds_hh_mm_ss_invalid_parts() {
        assert_eq!(parse_duration_seconds("1:60:00"), None);
        assert_eq!(parse_duration_seconds("1:00:60"), None);
    }
    #[test]
    fn parse_duration_seconds_too_many_parts_returns_none() {
        assert_eq!(parse_duration_seconds("1:2:3:4"), None);
    }
    #[test]
    fn parse_duration_seconds_non_numeric_returns_none() {
        assert_eq!(parse_duration_seconds("abc"), None);
        assert_eq!(parse_duration_seconds("1:ab"), None);
    }
}
