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
    Some((val * 1000.0).round() as u32)
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
