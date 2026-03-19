use serde::{Deserialize, Serialize};
use std::fmt;

/// Weight stored as hectograms (100 g units). 1 kg = 10 hg.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Weight(pub u16);

impl fmt::Display for Weight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_multiple_of(10) {
            write!(f, "{} kg", self.0 / 10)
        } else {
            write!(f, "{:.1} kg", f64::from(self.0) / 10.0)
        }
    }
}

/// Distance stored as meters. 1 km = 1000 m.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Distance(pub u32);

impl fmt::Display for Distance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 >= 1000 {
            if self.0.is_multiple_of(1000) {
                write!(f, "{} km", self.0 / 1000)
            } else {
                write!(f, "{:.2} km", f64::from(self.0) / 1000.0)
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
    let hg = (val * 10.0).round();
    if hg < 1.0 || hg > f64::from(u16::MAX) {
        return None;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    Some(Weight(hg as u16))
}

/// Parse a user-entered km string into a Distance (meters).
pub fn parse_distance_km(input: &str) -> Option<Distance> {
    let val: f64 = input.parse().ok()?;
    if !val.is_finite() || val <= 0.0 {
        return None;
    }
    let m = (val * 1000.0).round();
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
}
