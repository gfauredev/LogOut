use serde::{Deserialize, Serialize};

// Base URL for exercise images from the free-exercise-db repository
const EXERCISES_IMAGE_BASE_URL: &str = "https://raw.githubusercontent.com/yuhonas/free-exercise-db/main/exercises/";

// Version control for data structures to handle migrations
pub const DATA_VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Exercise {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force: Option<String>,
    pub level: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mechanic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equipment: Option<String>,
    #[serde(rename = "primaryMuscles")]
    pub primary_muscles: Vec<String>,
    #[serde(rename = "secondaryMuscles")]
    pub secondary_muscles: Vec<String>,
    pub instructions: Vec<String>,
    pub category: String,
    pub images: Vec<String>,
}

impl Exercise {
    /// Get the first image URL if available
    pub fn get_first_image_url(&self) -> Option<String> {
        self.images
            .first()
            .map(|img| format!("{}{}", EXERCISES_IMAGE_BASE_URL, img))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkoutSet {
    pub reps: u32,
    /// Weight stored as decagrams (10 g units). Display with `format_weight`.
    pub weight_dg: Option<u16>,
    pub duration: Option<u32>, // in seconds
}

/// Format a weight stored in decagrams for display as kg.
pub fn format_weight(dg: u16) -> String {
    if dg % 100 == 0 {
        format!("{} kg", dg / 100)
    } else {
        format!("{:.1} kg", dg as f64 / 100.0)
    }
}

/// Format a distance stored in metres for display.
pub fn format_distance(metres: u32) -> String {
    if metres >= 1000 {
        if metres % 1000 == 0 {
            format!("{} km", metres / 1000)
        } else {
            format!("{:.2} km", metres as f64 / 1000.0)
        }
    } else {
        format!("{} m", metres)
    }
}

/// Parse a user-entered kg string into decagrams.
pub fn parse_weight_kg(input: &str) -> Option<u16> {
    let val: f64 = input.parse().ok()?;
    if !val.is_finite() || val <= 0.0 { return None; }
    let dg = (val * 100.0).round() as u32;
    if dg > u16::MAX as u32 { return None; }
    Some(dg as u16)
}

/// Parse a user-entered km string into metres.
pub fn parse_distance_km(input: &str) -> Option<u32> {
    let val: f64 = input.parse().ok()?;
    if !val.is_finite() || val <= 0.0 { return None; }
    let m = (val * 1000.0).round();
    if m > u32::MAX as f64 { return None; }
    Some(m as u32)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkoutExercise {
    pub exercise_id: String,
    pub exercise_name: String,
    pub sets: Vec<WorkoutSet>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Workout {
    pub id: String,
    pub date: String,
    pub exercises: Vec<WorkoutExercise>,
    pub notes: Option<String>,
    #[serde(default)]
    pub version: u32,
}

// Models for active session tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExerciseLog {
    pub exercise_id: String,
    pub exercise_name: String,
    pub category: String,
    pub start_time: u64,
    pub end_time: Option<u64>,
    /// Weight in decagrams
    pub weight_dg: Option<u16>,
    pub reps: Option<u32>,
    /// Distance in metres
    pub distance_m: Option<u32>,
}

impl ExerciseLog {
    /// Calculate duration in seconds
    pub fn duration_seconds(&self) -> Option<u64> {
        self.end_time.map(|end| end.saturating_sub(self.start_time))
    }

    /// Check if this log is complete (has end time)
    pub fn is_complete(&self) -> bool {
        self.end_time.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkoutSession {
    pub id: String,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub exercise_logs: Vec<ExerciseLog>,
    #[serde(default)]
    pub version: u32,
}

impl WorkoutSession {
    /// Create a new workout session
    pub fn new() -> Self {
        let timestamp = get_current_timestamp();
        Self {
            id: format!("session_{}", timestamp),
            start_time: timestamp,
            end_time: None,
            exercise_logs: Vec::new(),
            version: DATA_VERSION,
        }
    }

    /// Check if session is active (not finished)
    pub fn is_active(&self) -> bool {
        self.end_time.is_none()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomExercise {
    pub id: String,
    pub name: String,
    pub category: String,
    pub force: Option<String>,
    pub equipment: Option<String>,
    pub primary_muscles: Vec<String>,
}

/// Get current timestamp compatible with WASM
pub fn get_current_timestamp() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::now() / 1000.0) as u64
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

/// Format a duration in seconds as HH:MM:SS or MM:SS
pub fn format_time(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{:02}:{:02}", minutes, secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── format_weight ─────────────────────────────────────────────────────────

    #[test]
    fn format_weight_whole_kg() {
        assert_eq!(format_weight(100), "1 kg");
        assert_eq!(format_weight(200), "2 kg");
        assert_eq!(format_weight(10000), "100 kg");
    }

    #[test]
    fn format_weight_fractional_kg() {
        assert_eq!(format_weight(150), "1.5 kg");
        assert_eq!(format_weight(175), "1.8 kg");
        assert_eq!(format_weight(1), "0.0 kg");
    }

    #[test]
    fn format_weight_zero() {
        assert_eq!(format_weight(0), "0 kg");
    }

    // ── parse_weight_kg ───────────────────────────────────────────────────────

    #[test]
    fn parse_weight_kg_valid() {
        assert_eq!(parse_weight_kg("1"), Some(100));
        assert_eq!(parse_weight_kg("1.5"), Some(150));
        assert_eq!(parse_weight_kg("100"), Some(10000));
        assert_eq!(parse_weight_kg("0.01"), Some(1));
    }

    #[test]
    fn parse_weight_kg_invalid() {
        assert_eq!(parse_weight_kg(""), None);
        assert_eq!(parse_weight_kg("abc"), None);
        assert_eq!(parse_weight_kg("-1"), None);
        assert_eq!(parse_weight_kg("0"), None);
        assert_eq!(parse_weight_kg("nan"), None);
    }

    // ── format_distance ───────────────────────────────────────────────────────

    #[test]
    fn format_distance_metres() {
        assert_eq!(format_distance(0), "0 m");
        assert_eq!(format_distance(500), "500 m");
        assert_eq!(format_distance(999), "999 m");
    }

    #[test]
    fn format_distance_whole_km() {
        assert_eq!(format_distance(1000), "1 km");
        assert_eq!(format_distance(5000), "5 km");
    }

    #[test]
    fn format_distance_fractional_km() {
        assert_eq!(format_distance(1500), "1.50 km");
        assert_eq!(format_distance(2750), "2.75 km");
    }

    // ── parse_distance_km ─────────────────────────────────────────────────────

    #[test]
    fn parse_distance_km_valid() {
        assert_eq!(parse_distance_km("1"), Some(1000));
        assert_eq!(parse_distance_km("0.5"), Some(500));
        assert_eq!(parse_distance_km("10"), Some(10000));
    }

    #[test]
    fn parse_distance_km_invalid() {
        assert_eq!(parse_distance_km(""), None);
        assert_eq!(parse_distance_km("abc"), None);
        assert_eq!(parse_distance_km("-1"), None);
        assert_eq!(parse_distance_km("0"), None);
    }

    // ── format_time ───────────────────────────────────────────────────────────

    #[test]
    fn format_time_minutes_seconds() {
        assert_eq!(format_time(0), "00:00");
        assert_eq!(format_time(59), "00:59");
        assert_eq!(format_time(60), "01:00");
        assert_eq!(format_time(3599), "59:59");
    }

    #[test]
    fn format_time_hours() {
        assert_eq!(format_time(3600), "01:00:00");
        assert_eq!(format_time(3661), "01:01:01");
        assert_eq!(format_time(7322), "02:02:02");
    }

    // ── ExerciseLog ───────────────────────────────────────────────────────────

    #[test]
    fn exercise_log_is_complete() {
        let mut log = ExerciseLog {
            exercise_id: "ex1".into(),
            exercise_name: "Push-up".into(),
            category: "strength".into(),
            start_time: 1000,
            end_time: None,
            weight_dg: None,
            reps: None,
            distance_m: None,
        };
        assert!(!log.is_complete());
        log.end_time = Some(1060);
        assert!(log.is_complete());
    }

    #[test]
    fn exercise_log_duration_seconds() {
        let log = ExerciseLog {
            exercise_id: "ex1".into(),
            exercise_name: "Push-up".into(),
            category: "strength".into(),
            start_time: 1000,
            end_time: Some(1060),
            weight_dg: None,
            reps: None,
            distance_m: None,
        };
        assert_eq!(log.duration_seconds(), Some(60));
    }

    #[test]
    fn exercise_log_duration_seconds_none_when_incomplete() {
        let log = ExerciseLog {
            exercise_id: "ex1".into(),
            exercise_name: "Push-up".into(),
            category: "strength".into(),
            start_time: 1000,
            end_time: None,
            weight_dg: None,
            reps: None,
            distance_m: None,
        };
        assert_eq!(log.duration_seconds(), None);
    }

    // ── WorkoutSession ────────────────────────────────────────────────────────

    #[test]
    fn workout_session_is_active() {
        let mut session = WorkoutSession {
            id: "s1".into(),
            start_time: 1000,
            end_time: None,
            exercise_logs: vec![],
            version: DATA_VERSION,
        };
        assert!(session.is_active());
        session.end_time = Some(2000);
        assert!(!session.is_active());
    }

    #[test]
    fn workout_session_new_has_no_end_time() {
        let session = WorkoutSession::new();
        assert!(session.is_active());
        assert!(session.id.starts_with("session_"));
        assert_eq!(session.version, DATA_VERSION);
    }

    // ── Exercise ──────────────────────────────────────────────────────────────

    #[test]
    fn exercise_get_first_image_url_some() {
        let ex = Exercise {
            id: "ex1".into(),
            name: "Squat".into(),
            force: None,
            level: "beginner".into(),
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec![],
            category: "strength".into(),
            images: vec!["Squat/0.jpg".into()],
        };
        assert_eq!(
            ex.get_first_image_url(),
            Some("https://raw.githubusercontent.com/yuhonas/free-exercise-db/main/exercises/Squat/0.jpg".into())
        );
    }

    #[test]
    fn exercise_get_first_image_url_none() {
        let ex = Exercise {
            id: "ex1".into(),
            name: "Squat".into(),
            force: None,
            level: "beginner".into(),
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec![],
            category: "strength".into(),
            images: vec![],
        };
        assert_eq!(ex.get_first_image_url(), None);
    }
}
