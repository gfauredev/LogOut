use serde::{Deserialize, Serialize};
use std::fmt;

// Image sub-path within the exercise database repository
const EXERCISES_IMAGE_SUB_PATH: &str = "exercises/";
// Version control for data structures to handle migrations
pub const DATA_VERSION: u16 = 0;

// ── Enums for exercise fields with fixed values ─────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    #[serde(rename = "cardio")]
    Cardio,
    #[serde(rename = "olympic weightlifting")]
    OlympicWeightlifting,
    #[serde(rename = "plyometrics")]
    Plyometrics,
    #[serde(rename = "powerlifting")]
    Powerlifting,
    #[serde(rename = "strength")]
    Strength,
    #[serde(rename = "stretching")]
    Stretching,
    #[serde(rename = "strongman")]
    Strongman,
}

impl Category {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cardio => "cardio",
            Self::OlympicWeightlifting => "olympic weightlifting",
            Self::Plyometrics => "plyometrics",
            Self::Powerlifting => "powerlifting",
            Self::Strength => "strength",
            Self::Stretching => "stretching",
            Self::Strongman => "strongman",
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Category {
    pub const ALL: &'static [Category] = &[
        Self::Strength,
        Self::Cardio,
        Self::Stretching,
        Self::Powerlifting,
        Self::Strongman,
        Self::Plyometrics,
        Self::OlympicWeightlifting,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Force {
    #[serde(rename = "pull")]
    Pull,
    #[serde(rename = "push")]
    Push,
    #[serde(rename = "static")]
    Static,
}

impl Force {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pull => "pull",
            Self::Push => "push",
            Self::Static => "static",
        }
    }
}

impl fmt::Display for Force {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Force {
    pub const ALL: &'static [Force] = &[Self::Pull, Self::Push, Self::Static];

    /// Returns true if reps are applicable for this force type.
    pub fn has_reps(self) -> bool {
        matches!(self, Self::Pull | Self::Push)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Level {
    #[serde(rename = "beginner")]
    Beginner,
    #[serde(rename = "intermediate")]
    Intermediate,
    #[serde(rename = "expert")]
    Expert,
}

impl Level {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Beginner => "beginner",
            Self::Intermediate => "intermediate",
            Self::Expert => "expert",
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mechanic {
    #[serde(rename = "compound")]
    Compound,
    #[serde(rename = "isolation")]
    Isolation,
}

impl Mechanic {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Compound => "compound",
            Self::Isolation => "isolation",
        }
    }
}

impl fmt::Display for Mechanic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Equipment {
    #[serde(rename = "bands")]
    Bands,
    #[serde(rename = "barbell")]
    Barbell,
    #[serde(rename = "body only")]
    BodyOnly,
    #[serde(rename = "cable")]
    Cable,
    #[serde(rename = "dumbbell")]
    Dumbbell,
    #[serde(rename = "e-z curl bar")]
    EzCurlBar,
    #[serde(rename = "exercise ball")]
    ExerciseBall,
    #[serde(rename = "foam roll")]
    FoamRoll,
    #[serde(rename = "kettlebells")]
    Kettlebells,
    #[serde(rename = "machine")]
    Machine,
    #[serde(rename = "medicine ball")]
    MedicineBall,
    #[serde(rename = "other")]
    Other,
}

impl Equipment {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bands => "bands",
            Self::Barbell => "barbell",
            Self::BodyOnly => "body only",
            Self::Cable => "cable",
            Self::Dumbbell => "dumbbell",
            Self::EzCurlBar => "e-z curl bar",
            Self::ExerciseBall => "exercise ball",
            Self::FoamRoll => "foam roll",
            Self::Kettlebells => "kettlebells",
            Self::Machine => "machine",
            Self::MedicineBall => "medicine ball",
            Self::Other => "other",
        }
    }
}

impl fmt::Display for Equipment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Equipment {
    pub const ALL: &'static [Equipment] = &[
        Self::Bands,
        Self::Barbell,
        Self::BodyOnly,
        Self::Cable,
        Self::Dumbbell,
        Self::EzCurlBar,
        Self::ExerciseBall,
        Self::FoamRoll,
        Self::Kettlebells,
        Self::Machine,
        Self::MedicineBall,
        Self::Other,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Muscle {
    #[serde(rename = "abdominals")]
    Abdominals,
    #[serde(rename = "abductors")]
    Abductors,
    #[serde(rename = "adductors")]
    Adductors,
    #[serde(rename = "biceps")]
    Biceps,
    #[serde(rename = "calves")]
    Calves,
    #[serde(rename = "chest")]
    Chest,
    #[serde(rename = "forearms")]
    Forearms,
    #[serde(rename = "glutes")]
    Glutes,
    #[serde(rename = "hamstrings")]
    Hamstrings,
    #[serde(rename = "lats")]
    Lats,
    #[serde(rename = "lower back")]
    LowerBack,
    #[serde(rename = "middle back")]
    MiddleBack,
    #[serde(rename = "neck")]
    Neck,
    #[serde(rename = "quadriceps")]
    Quadriceps,
    #[serde(rename = "shoulders")]
    Shoulders,
    #[serde(rename = "traps")]
    Traps,
    #[serde(rename = "triceps")]
    Triceps,
}

impl Muscle {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Abdominals => "abdominals",
            Self::Abductors => "abductors",
            Self::Adductors => "adductors",
            Self::Biceps => "biceps",
            Self::Calves => "calves",
            Self::Chest => "chest",
            Self::Forearms => "forearms",
            Self::Glutes => "glutes",
            Self::Hamstrings => "hamstrings",
            Self::Lats => "lats",
            Self::LowerBack => "lower back",
            Self::MiddleBack => "middle back",
            Self::Neck => "neck",
            Self::Quadriceps => "quadriceps",
            Self::Shoulders => "shoulders",
            Self::Traps => "traps",
            Self::Triceps => "triceps",
        }
    }
}

impl fmt::Display for Muscle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Muscle {
    pub const ALL: &'static [Muscle] = &[
        Self::Abdominals,
        Self::Abductors,
        Self::Adductors,
        Self::Biceps,
        Self::Calves,
        Self::Chest,
        Self::Forearms,
        Self::Glutes,
        Self::Hamstrings,
        Self::Lats,
        Self::LowerBack,
        Self::MiddleBack,
        Self::Neck,
        Self::Quadriceps,
        Self::Shoulders,
        Self::Traps,
        Self::Triceps,
    ];
}

// ── Weight and Distance value types ─────────────────────────────────────────

/// Weight stored as decagrams (10 g units). 1 kg = 100 dg.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Weight(pub u16);

impl fmt::Display for Weight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_multiple_of(100) {
            write!(f, "{} kg", self.0 / 100)
        } else {
            write!(f, "{:.1} kg", self.0 as f64 / 100.0)
        }
    }
}

/// Distance stored as decameters (10 m units). 1 km = 100 dam.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Distance(pub u16);

impl fmt::Display for Distance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 >= 100 {
            if self.0.is_multiple_of(100) {
                write!(f, "{} km", self.0 / 100)
            } else {
                write!(f, "{:.2} km", self.0 as f64 / 100.0)
            }
        } else {
            write!(f, "{} m", self.0 as u32 * 10)
        }
    }
}

/// Parse a user-entered kg string into a Weight (decagrams).
pub fn parse_weight_kg(input: &str) -> Option<Weight> {
    let val: f64 = input.parse().ok()?;
    if !val.is_finite() || val <= 0.0 {
        return None;
    }
    let dg = (val * 100.0).round();
    if dg < 0.0 || dg > u16::MAX as f64 {
        return None;
    }
    Some(Weight(dg as u16))
}

/// Parse a user-entered km string into a Distance (decameters).
pub fn parse_distance_km(input: &str) -> Option<Distance> {
    let val: f64 = input.parse().ok()?;
    if !val.is_finite() || val <= 0.0 {
        return None;
    }
    let dam = (val * 100.0).round();
    if dam < 0.0 || dam > u16::MAX as f64 {
        return None;
    }
    Some(Distance(dam as u16))
}

// ── Data structures ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Exercise {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force: Option<Force>,
    #[serde(default)]
    pub level: Option<Level>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mechanic: Option<Mechanic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equipment: Option<Equipment>,
    #[serde(rename = "primaryMuscles")]
    pub primary_muscles: Vec<Muscle>,
    #[serde(rename = "secondaryMuscles")]
    #[serde(default)]
    pub secondary_muscles: Vec<Muscle>,
    #[serde(default)]
    pub instructions: Vec<String>,
    pub category: Category,
    #[serde(default)]
    pub images: Vec<String>,
}

impl Exercise {
    /// Get the URL for a specific image by index.
    /// Images that are already full URLs (start with http:// or https://) are
    /// returned as-is.  Relative paths from the exercise-db are prefixed with
    /// the EXERCISES_IMAGE_BASE_URL.
    pub fn get_image_url(&self, index: usize) -> Option<String> {
        self.images.get(index).map(|img| {
            if img.starts_with("http://") || img.starts_with("https://") {
                img.clone()
            } else {
                format!(
                    "{}{}{}",
                    crate::utils::EXERCISE_DB_BASE_URL,
                    EXERCISES_IMAGE_SUB_PATH,
                    img
                )
            }
        })
    }

    /// Get the first image URL if available
    #[allow(dead_code)]
    pub fn get_first_image_url(&self) -> Option<String> {
        self.get_image_url(0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkoutSet {
    pub reps: u32,
    /// Weight in decagrams
    pub weight_dg: Option<Weight>,
    pub duration: Option<u32>, // in seconds
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
    pub version: u16,
}

// Models for active session tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExerciseLog {
    pub exercise_id: String,
    pub exercise_name: String,
    pub category: Category,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub weight_dg: Option<Weight>,
    pub reps: Option<u32>,
    /// Distance in decameters
    pub distance_dam: Option<Distance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force: Option<Force>,
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
    pub version: u16,
    /// Exercise IDs queued from a previous session (pre-added, not yet started).
    #[serde(default)]
    pub pending_exercise_ids: Vec<String>,
    /// Timestamp when the last rest period started (for persisting timer across tab switches).
    #[serde(default)]
    pub rest_start_time: Option<u64>,
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
            pending_exercise_ids: Vec::new(),
            rest_start_time: None,
        }
    }

    /// Check if session is active (not finished)
    pub fn is_active(&self) -> bool {
        self.end_time.is_none()
    }

    /// Returns true when the session was cancelled (no exercises logged).
    /// Cancelled sessions should be deleted, not stored.
    pub fn is_cancelled(&self) -> bool {
        self.exercise_logs.is_empty()
    }
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
            .unwrap_or_default()
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

    // ── Weight Display ──────────────────────────────────────────────────────────

    #[test]
    fn weight_display_whole_kg() {
        assert_eq!(Weight(100).to_string(), "1 kg");
        assert_eq!(Weight(200).to_string(), "2 kg");
        assert_eq!(Weight(10000).to_string(), "100 kg");
    }

    #[test]
    fn weight_display_fractional_kg() {
        assert_eq!(Weight(150).to_string(), "1.5 kg");
        assert_eq!(Weight(175).to_string(), "1.8 kg");
        assert_eq!(Weight(1).to_string(), "0.0 kg");
    }

    #[test]
    fn weight_display_zero() {
        assert_eq!(Weight(0).to_string(), "0 kg");
    }

    // ── parse_weight_kg ───────────────────────────────────────────────────────

    #[test]
    fn parse_weight_kg_valid() {
        assert_eq!(parse_weight_kg("1"), Some(Weight(100)));
        assert_eq!(parse_weight_kg("1.5"), Some(Weight(150)));
        assert_eq!(parse_weight_kg("100"), Some(Weight(10000)));
        assert_eq!(parse_weight_kg("0.01"), Some(Weight(1)));
    }

    #[test]
    fn parse_weight_kg_invalid() {
        assert_eq!(parse_weight_kg(""), None);
        assert_eq!(parse_weight_kg("abc"), None);
        assert_eq!(parse_weight_kg("-1"), None);
        assert_eq!(parse_weight_kg("0"), None);
        assert_eq!(parse_weight_kg("nan"), None);
    }

    // ── Distance Display ────────────────────────────────────────────────────────

    #[test]
    fn distance_display_metres() {
        assert_eq!(Distance(0).to_string(), "0 m");
        assert_eq!(Distance(50).to_string(), "500 m");
        assert_eq!(Distance(99).to_string(), "990 m");
    }

    #[test]
    fn distance_display_whole_km() {
        assert_eq!(Distance(100).to_string(), "1 km");
        assert_eq!(Distance(500).to_string(), "5 km");
    }

    #[test]
    fn distance_display_fractional_km() {
        assert_eq!(Distance(150).to_string(), "1.50 km");
        assert_eq!(Distance(275).to_string(), "2.75 km");
    }

    // ── parse_distance_km ─────────────────────────────────────────────────────

    #[test]
    fn parse_distance_km_valid() {
        assert_eq!(parse_distance_km("1"), Some(Distance(100)));
        assert_eq!(parse_distance_km("0.5"), Some(Distance(50)));
        assert_eq!(parse_distance_km("10"), Some(Distance(1000)));
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
            category: Category::Strength,
            start_time: 1000,
            end_time: None,
            weight_dg: None,
            reps: None,
            distance_dam: None,
            force: Some(Force::Push),
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
            category: Category::Strength,
            start_time: 1000,
            end_time: Some(1060),
            weight_dg: None,
            reps: None,
            distance_dam: None,
            force: Some(Force::Push),
        };
        assert_eq!(log.duration_seconds(), Some(60));
    }

    #[test]
    fn exercise_log_duration_seconds_none_when_incomplete() {
        let log = ExerciseLog {
            exercise_id: "ex1".into(),
            exercise_name: "Push-up".into(),
            category: Category::Strength,
            start_time: 1000,
            end_time: None,
            weight_dg: None,
            reps: None,
            distance_dam: None,
            force: Some(Force::Push),
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
            pending_exercise_ids: vec![],
            rest_start_time: None,
        };
        assert!(session.is_active());
        session.end_time = Some(2000);
        assert!(!session.is_active());
    }

    #[test]
    fn workout_session_is_cancelled_when_no_exercises() {
        let session = WorkoutSession {
            id: "s1".into(),
            start_time: 1000,
            end_time: None,
            exercise_logs: vec![],
            version: DATA_VERSION,
            pending_exercise_ids: vec![],
            rest_start_time: None,
        };
        assert!(session.is_cancelled());
    }

    #[test]
    fn workout_session_is_not_cancelled_when_has_exercises() {
        let log = ExerciseLog {
            exercise_id: "ex1".into(),
            exercise_name: "Push-up".into(),
            category: Category::Strength,
            start_time: 1000,
            end_time: Some(1060),
            weight_dg: None,
            reps: Some(10),
            distance_dam: None,
            force: Some(Force::Push),
        };
        let session = WorkoutSession {
            id: "s1".into(),
            start_time: 1000,
            end_time: None,
            exercise_logs: vec![log],
            version: DATA_VERSION,
            pending_exercise_ids: vec![],
            rest_start_time: None,
        };
        assert!(!session.is_cancelled());
    }

    /// A session with no exercises is cancelled and must be deleted (not saved).
    /// finish_session uses is_cancelled() to decide between delete_session and save_session.
    #[test]
    fn finish_session_cancelled_session_is_not_stored() {
        let session = WorkoutSession {
            id: "s1".into(),
            start_time: 1000,
            end_time: None,
            exercise_logs: vec![],
            version: DATA_VERSION,
            pending_exercise_ids: vec![],
            rest_start_time: None,
        };
        // The predicate that guards save vs. delete must return true for empty sessions.
        assert!(
            session.is_cancelled(),
            "Session with no exercises must be treated as cancelled"
        );
    }

    /// A session that has exercises is not cancelled and must be saved with an end_time.
    /// finish_session uses is_cancelled() to decide between delete_session and save_session.
    #[test]
    fn finish_session_with_exercises_is_stored() {
        let log = ExerciseLog {
            exercise_id: "ex1".into(),
            exercise_name: "Squat".into(),
            category: Category::Strength,
            start_time: 1000,
            end_time: Some(1120),
            weight_dg: None,
            reps: Some(5),
            distance_dam: None,
            force: Some(Force::Push),
        };
        let session = WorkoutSession {
            id: "s1".into(),
            start_time: 1000,
            end_time: None,
            exercise_logs: vec![log],
            version: DATA_VERSION,
            pending_exercise_ids: vec![],
            rest_start_time: None,
        };
        // The predicate must return false so the session is saved, not deleted.
        assert!(
            !session.is_cancelled(),
            "Session with exercises must not be treated as cancelled"
        );
    }

    /// A session with pending exercises but no completed exercises is still
    /// considered cancelled. The finish button should not be shown in this state.
    #[test]
    fn session_with_only_pending_exercises_is_cancelled() {
        let session = WorkoutSession {
            id: "s1".into(),
            start_time: 1000,
            end_time: None,
            exercise_logs: vec![],
            version: DATA_VERSION,
            pending_exercise_ids: vec!["ex1".into(), "ex2".into()],
            rest_start_time: None,
        };
        assert!(
            session.is_cancelled(),
            "Session with only pending exercises (no completed logs) must be treated as cancelled"
        );
    }

    #[test]
    fn workout_session_new_has_no_end_time() {
        let session = WorkoutSession::new();
        assert!(session.is_active());
        assert!(session.id.starts_with("session_"));
        assert_eq!(session.version, DATA_VERSION);
    }

    #[test]
    fn find_active_session_returns_first_without_end_time() {
        let sessions = vec![
            WorkoutSession {
                id: "s1".into(),
                start_time: 1000,
                end_time: Some(2000),
                exercise_logs: vec![],
                version: DATA_VERSION,
                pending_exercise_ids: vec![],
                rest_start_time: None,
            },
            WorkoutSession {
                id: "s2".into(),
                start_time: 3000,
                end_time: None,
                exercise_logs: vec![],
                version: DATA_VERSION,
                pending_exercise_ids: vec![],
                rest_start_time: None,
            },
        ];
        let active = sessions.iter().find(|s| s.is_active()).cloned();
        assert_eq!(active.unwrap().id, "s2");
    }

    #[test]
    fn find_active_session_returns_none_when_all_finished() {
        let sessions = vec![WorkoutSession {
            id: "s1".into(),
            start_time: 1000,
            end_time: Some(2000),
            exercise_logs: vec![],
            version: DATA_VERSION,
            pending_exercise_ids: vec![],
            rest_start_time: None,
        }];
        let active = sessions.iter().find(|s| s.is_active()).cloned();
        assert!(active.is_none());
    }

    #[test]
    fn find_active_session_returns_none_for_empty_list() {
        let sessions: Vec<WorkoutSession> = vec![];
        let active = sessions.iter().find(|s| s.is_active()).cloned();
        assert!(active.is_none());
    }

    // ── Exercise ──────────────────────────────────────────────────────────────

    #[test]
    fn exercise_get_first_image_url_some() {
        let ex = Exercise {
            id: "ex1".into(),
            name: "Squat".into(),
            force: None,
            level: Some(Level::Beginner),
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec![],
            category: Category::Strength,
            images: vec!["Squat/0.jpg".into()],
        };
        assert_eq!(
            ex.get_first_image_url(),
            Some("https://raw.githubusercontent.com/gfauredev/free-exercise-db/main/exercises/Squat/0.jpg".into())
        );
    }

    #[test]
    fn exercise_get_first_image_url_none() {
        let ex = Exercise {
            id: "ex1".into(),
            name: "Squat".into(),
            force: None,
            level: Some(Level::Beginner),
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec![],
            category: Category::Strength,
            images: vec![],
        };
        assert_eq!(ex.get_first_image_url(), None);
    }

    // ── Enum serialization ────────────────────────────────────────────────────

    #[test]
    fn category_round_trip() {
        let json = serde_json::to_string(&Category::OlympicWeightlifting).unwrap();
        assert_eq!(json, "\"olympic weightlifting\"");
        let back: Category = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Category::OlympicWeightlifting);
    }

    #[test]
    fn equipment_round_trip() {
        let json = serde_json::to_string(&Equipment::BodyOnly).unwrap();
        assert_eq!(json, "\"body only\"");
        let back: Equipment = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Equipment::BodyOnly);
    }

    #[test]
    fn muscle_round_trip() {
        let json = serde_json::to_string(&Muscle::LowerBack).unwrap();
        assert_eq!(json, "\"lower back\"");
        let back: Muscle = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Muscle::LowerBack);
    }

    #[test]
    fn force_has_reps() {
        assert!(Force::Push.has_reps());
        assert!(Force::Pull.has_reps());
        assert!(!Force::Static.has_reps());
    }

    // ── Safe float casts in parse functions ───────────────────────────────────

    #[test]
    fn parse_weight_kg_large_value_clamped() {
        // 655.36 kg = 65536 dg which overflows u16
        assert_eq!(parse_weight_kg("655.36"), None);
        // 655.35 kg = 65535 dg fits in u16
        assert_eq!(parse_weight_kg("655.35"), Some(Weight(65535)));
    }

    #[test]
    fn parse_distance_km_large_value_clamped() {
        // 655.36 km = 65536 dam which overflows u16
        assert_eq!(parse_distance_km("655.36"), None);
        // 655.35 km = 65535 dam fits in u16
        assert_eq!(parse_distance_km("655.35"), Some(Distance(65535)));
    }

    // ── User-created exercise (uses unified Exercise struct) ────────────────

    #[test]
    fn user_exercise_serialization_with_all_fields() {
        let exercise = Exercise {
            id: "custom_123".into(),
            name: "Test Exercise".into(),
            category: Category::Strength,
            force: Some(Force::Push),
            level: None,
            mechanic: None,
            equipment: Some(Equipment::Barbell),
            primary_muscles: vec![Muscle::Chest],
            secondary_muscles: vec![Muscle::Triceps, Muscle::Shoulders],
            instructions: vec!["Step 1".into(), "Step 2".into()],
            images: vec!["https://example.com/img.jpg".into()],
        };
        let json = serde_json::to_string(&exercise).unwrap();
        let deserialized: Exercise = serde_json::from_str(&json).unwrap();
        assert_eq!(exercise, deserialized);
        assert_eq!(deserialized.secondary_muscles.len(), 2);
        assert_eq!(deserialized.instructions.len(), 2);
        assert_eq!(deserialized.images.len(), 1);
    }

    #[test]
    fn exercise_backward_compat_missing_optional_fields() {
        // Old format without secondary_muscles, instructions, images, level
        let json = r#"{"id":"custom_1","name":"Old Exercise","category":"strength","force":"push","equipment":"barbell","primaryMuscles":["chest"]}"#;
        let exercise: Exercise = serde_json::from_str(json).unwrap();
        assert_eq!(exercise.secondary_muscles, Vec::<Muscle>::new());
        assert_eq!(exercise.instructions, Vec::<String>::new());
        assert_eq!(exercise.images, Vec::<String>::new());
        assert_eq!(exercise.level, None);
    }

    // ── WorkoutSession pending_exercise_ids ───────────────────────────────────

    #[test]
    fn workout_session_new_has_empty_pending_ids() {
        let session = WorkoutSession::new();
        assert!(session.pending_exercise_ids.is_empty());
    }

    #[test]
    fn workout_session_pending_ids_serialization_round_trip() {
        let session = WorkoutSession {
            id: "s1".into(),
            start_time: 1000,
            end_time: None,
            exercise_logs: vec![],
            version: DATA_VERSION,
            pending_exercise_ids: vec!["ex1".into(), "ex2".into()],
            rest_start_time: None,
        };
        let json = serde_json::to_string(&session).unwrap();
        let back: WorkoutSession = serde_json::from_str(&json).unwrap();
        assert_eq!(back.pending_exercise_ids, vec!["ex1", "ex2"]);
    }

    #[test]
    fn workout_session_backward_compat_missing_pending_ids() {
        // Old sessions without pending_exercise_ids should deserialize with empty vec
        let json =
            r#"{"id":"s1","start_time":1000,"end_time":null,"exercise_logs":[],"version":3}"#;
        let session: WorkoutSession = serde_json::from_str(json).unwrap();
        assert!(session.pending_exercise_ids.is_empty());
    }

    #[test]
    fn pending_ids_include_repeated_exercises() {
        // When an exercise is performed multiple times in a session, each
        // occurrence should appear in pending_exercise_ids so the repeated
        // session mirrors the original exactly.
        let session = WorkoutSession {
            id: "s1".into(),
            start_time: 1000,
            end_time: Some(2000),
            exercise_logs: vec![
                ExerciseLog {
                    exercise_id: "bench_press".into(),
                    exercise_name: "Bench Press".into(),
                    category: Category::Strength,
                    start_time: 1000,
                    end_time: Some(1100),
                    weight_dg: None,
                    reps: Some(10),
                    distance_dam: None,
                    force: None,
                },
                ExerciseLog {
                    exercise_id: "squat".into(),
                    exercise_name: "Squat".into(),
                    category: Category::Strength,
                    start_time: 1200,
                    end_time: Some(1300),
                    weight_dg: None,
                    reps: Some(8),
                    distance_dam: None,
                    force: None,
                },
                ExerciseLog {
                    exercise_id: "bench_press".into(),
                    exercise_name: "Bench Press".into(),
                    category: Category::Strength,
                    start_time: 1400,
                    end_time: Some(1500),
                    weight_dg: None,
                    reps: Some(8),
                    distance_dam: None,
                    force: None,
                },
            ],
            version: DATA_VERSION,
            pending_exercise_ids: vec![],
            rest_start_time: None,
        };

        // Build pending IDs the same way SessionCard does (all logs, not deduplicated)
        let pending: Vec<String> = session
            .exercise_logs
            .iter()
            .map(|log| log.exercise_id.clone())
            .collect();

        assert_eq!(
            pending,
            vec!["bench_press", "squat", "bench_press"],
            "repeated exercises must appear in pending_ids as many times as performed"
        );
    }

    #[test]
    fn remove_first_occurrence_from_pending_ids() {
        // Simulates the retain logic in active_session.rs: removing only the
        // first occurrence of an exercise ID so that subsequent repetitions
        // remain in the queue.
        let mut pending = vec![
            "bench_press".to_string(),
            "squat".to_string(),
            "bench_press".to_string(),
        ];
        let target = "bench_press";
        let mut removed = false;
        pending.retain(|x| {
            if !removed && x == target {
                removed = true;
                false
            } else {
                true
            }
        });

        assert_eq!(
            pending,
            vec!["squat", "bench_press"],
            "only the first occurrence should be removed"
        );
    }

    // ── Display impls full coverage ──────────────────────────────────────────

    #[test]
    fn category_display_all_variants() {
        assert_eq!(Category::Cardio.to_string(), "cardio");
        assert_eq!(
            Category::OlympicWeightlifting.to_string(),
            "olympic weightlifting"
        );
        assert_eq!(Category::Plyometrics.to_string(), "plyometrics");
        assert_eq!(Category::Powerlifting.to_string(), "powerlifting");
        assert_eq!(Category::Strength.to_string(), "strength");
        assert_eq!(Category::Stretching.to_string(), "stretching");
        assert_eq!(Category::Strongman.to_string(), "strongman");
    }

    #[test]
    fn force_display_all_variants() {
        assert_eq!(Force::Pull.to_string(), "pull");
        assert_eq!(Force::Push.to_string(), "push");
        assert_eq!(Force::Static.to_string(), "static");
    }

    #[test]
    fn level_display_all_variants() {
        assert_eq!(Level::Beginner.to_string(), "beginner");
        assert_eq!(Level::Intermediate.to_string(), "intermediate");
        assert_eq!(Level::Expert.to_string(), "expert");
    }

    #[test]
    fn mechanic_display_all_variants() {
        assert_eq!(Mechanic::Compound.to_string(), "compound");
        assert_eq!(Mechanic::Isolation.to_string(), "isolation");
    }

    #[test]
    fn equipment_display_all_variants() {
        assert_eq!(Equipment::Bands.to_string(), "bands");
        assert_eq!(Equipment::Barbell.to_string(), "barbell");
        assert_eq!(Equipment::BodyOnly.to_string(), "body only");
        assert_eq!(Equipment::Cable.to_string(), "cable");
        assert_eq!(Equipment::Dumbbell.to_string(), "dumbbell");
        assert_eq!(Equipment::EzCurlBar.to_string(), "e-z curl bar");
        assert_eq!(Equipment::ExerciseBall.to_string(), "exercise ball");
        assert_eq!(Equipment::FoamRoll.to_string(), "foam roll");
        assert_eq!(Equipment::Kettlebells.to_string(), "kettlebells");
        assert_eq!(Equipment::Machine.to_string(), "machine");
        assert_eq!(Equipment::MedicineBall.to_string(), "medicine ball");
        assert_eq!(Equipment::Other.to_string(), "other");
    }

    #[test]
    fn muscle_display_all_variants() {
        assert_eq!(Muscle::Abdominals.to_string(), "abdominals");
        assert_eq!(Muscle::Abductors.to_string(), "abductors");
        assert_eq!(Muscle::Adductors.to_string(), "adductors");
        assert_eq!(Muscle::Biceps.to_string(), "biceps");
        assert_eq!(Muscle::Calves.to_string(), "calves");
        assert_eq!(Muscle::Chest.to_string(), "chest");
        assert_eq!(Muscle::Forearms.to_string(), "forearms");
        assert_eq!(Muscle::Glutes.to_string(), "glutes");
        assert_eq!(Muscle::Hamstrings.to_string(), "hamstrings");
        assert_eq!(Muscle::Lats.to_string(), "lats");
        assert_eq!(Muscle::LowerBack.to_string(), "lower back");
        assert_eq!(Muscle::MiddleBack.to_string(), "middle back");
        assert_eq!(Muscle::Neck.to_string(), "neck");
        assert_eq!(Muscle::Quadriceps.to_string(), "quadriceps");
        assert_eq!(Muscle::Shoulders.to_string(), "shoulders");
        assert_eq!(Muscle::Traps.to_string(), "traps");
        assert_eq!(Muscle::Triceps.to_string(), "triceps");
    }

    // ── ALL constants ────────────────────────────────────────────────────────

    #[test]
    fn category_all_contains_every_variant() {
        assert_eq!(Category::ALL.len(), 7);
        assert!(Category::ALL.contains(&Category::Cardio));
        assert!(Category::ALL.contains(&Category::OlympicWeightlifting));
        assert!(Category::ALL.contains(&Category::Plyometrics));
        assert!(Category::ALL.contains(&Category::Powerlifting));
        assert!(Category::ALL.contains(&Category::Strength));
        assert!(Category::ALL.contains(&Category::Stretching));
        assert!(Category::ALL.contains(&Category::Strongman));
    }

    #[test]
    fn force_all_contains_every_variant() {
        assert_eq!(Force::ALL.len(), 3);
        assert!(Force::ALL.contains(&Force::Pull));
        assert!(Force::ALL.contains(&Force::Push));
        assert!(Force::ALL.contains(&Force::Static));
    }

    #[test]
    fn equipment_all_contains_every_variant() {
        assert_eq!(Equipment::ALL.len(), 12);
    }

    #[test]
    fn muscle_all_contains_every_variant() {
        assert_eq!(Muscle::ALL.len(), 17);
    }

    // ── Serde round-trip for every enum variant ──────────────────────────────

    #[test]
    fn all_categories_serde_round_trip() {
        for &cat in Category::ALL {
            let json = serde_json::to_string(&cat).unwrap();
            let back: Category = serde_json::from_str(&json).unwrap();
            assert_eq!(back, cat);
        }
    }

    #[test]
    fn all_forces_serde_round_trip() {
        for &f in Force::ALL {
            let json = serde_json::to_string(&f).unwrap();
            let back: Force = serde_json::from_str(&json).unwrap();
            assert_eq!(back, f);
        }
    }

    #[test]
    fn all_equipment_serde_round_trip() {
        for &eq in Equipment::ALL {
            let json = serde_json::to_string(&eq).unwrap();
            let back: Equipment = serde_json::from_str(&json).unwrap();
            assert_eq!(back, eq);
        }
    }

    #[test]
    fn all_muscles_serde_round_trip() {
        for &m in Muscle::ALL {
            let json = serde_json::to_string(&m).unwrap();
            let back: Muscle = serde_json::from_str(&json).unwrap();
            assert_eq!(back, m);
        }
    }

    #[test]
    fn level_serde_round_trip() {
        for level in [Level::Beginner, Level::Intermediate, Level::Expert] {
            let json = serde_json::to_string(&level).unwrap();
            let back: Level = serde_json::from_str(&json).unwrap();
            assert_eq!(back, level);
        }
    }

    #[test]
    fn mechanic_serde_round_trip() {
        for mech in [Mechanic::Compound, Mechanic::Isolation] {
            let json = serde_json::to_string(&mech).unwrap();
            let back: Mechanic = serde_json::from_str(&json).unwrap();
            assert_eq!(back, mech);
        }
    }

    // ── Exercise::get_image_url ──────────────────────────────────────────────

    #[test]
    fn exercise_get_image_url_by_index() {
        let ex = Exercise {
            id: "ex1".into(),
            name: "Squat".into(),
            force: None,
            level: Some(Level::Beginner),
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec![],
            category: Category::Strength,
            images: vec!["Squat/0.jpg".into(), "Squat/1.jpg".into()],
        };
        assert_eq!(
            ex.get_image_url(0),
            Some(
                "https://raw.githubusercontent.com/gfauredev/free-exercise-db/main/exercises/Squat/0.jpg"
                    .into()
            )
        );
        assert_eq!(
            ex.get_image_url(1),
            Some(
                "https://raw.githubusercontent.com/gfauredev/free-exercise-db/main/exercises/Squat/1.jpg"
                    .into()
            )
        );
        assert_eq!(ex.get_image_url(2), None);
    }

    #[test]
    fn exercise_get_image_url_full_url_passthrough() {
        let ex = Exercise {
            id: "ex1".into(),
            name: "Custom".into(),
            force: None,
            level: None,
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec![],
            category: Category::Strength,
            images: vec!["https://example.com/image.jpg".into()],
        };
        // Full URLs should be returned as-is (no prefix)
        assert_eq!(
            ex.get_image_url(0),
            Some("https://example.com/image.jpg".into())
        );
    }

    #[test]
    fn exercise_level_none_when_missing_from_json() {
        let json = r#"{"id":"ex1","name":"Test","category":"strength","primaryMuscles":[]}"#;
        let ex: Exercise = serde_json::from_str(json).unwrap();
        assert_eq!(ex.level, None);
    }

    #[test]
    fn exercise_level_some_when_present_in_json() {
        let json = r#"{"id":"ex1","name":"Test","level":"expert","category":"strength","primaryMuscles":[]}"#;
        let ex: Exercise = serde_json::from_str(json).unwrap();
        assert_eq!(ex.level, Some(Level::Expert));
    }

    // ── Exercise full deserialization ─────────────────────────────────────────

    #[test]
    fn exercise_full_json_deserialization() {
        let json = r#"{
            "id": "bench_press",
            "name": "Bench Press",
            "force": "push",
            "level": "intermediate",
            "mechanic": "compound",
            "equipment": "barbell",
            "primaryMuscles": ["chest"],
            "secondaryMuscles": ["triceps", "shoulders"],
            "instructions": ["Lie down", "Push up"],
            "category": "strength",
            "images": ["BenchPress/0.jpg"]
        }"#;
        let ex: Exercise = serde_json::from_str(json).unwrap();
        assert_eq!(ex.id, "bench_press");
        assert_eq!(ex.name, "Bench Press");
        assert_eq!(ex.force, Some(Force::Push));
        assert_eq!(ex.level, Some(Level::Intermediate));
        assert_eq!(ex.mechanic, Some(Mechanic::Compound));
        assert_eq!(ex.equipment, Some(Equipment::Barbell));
        assert_eq!(ex.primary_muscles, vec![Muscle::Chest]);
        assert_eq!(
            ex.secondary_muscles,
            vec![Muscle::Triceps, Muscle::Shoulders]
        );
        assert_eq!(ex.instructions.len(), 2);
        assert_eq!(ex.category, Category::Strength);
        assert_eq!(ex.images, vec!["BenchPress/0.jpg"]);
    }

    #[test]
    fn exercise_optional_fields_none() {
        let json = r#"{
            "id": "stretch1",
            "name": "Hamstring Stretch",
            "level": "beginner",
            "primaryMuscles": ["hamstrings"],
            "secondaryMuscles": [],
            "instructions": [],
            "category": "stretching",
            "images": []
        }"#;
        let ex: Exercise = serde_json::from_str(json).unwrap();
        assert_eq!(ex.force, None);
        assert_eq!(ex.mechanic, None);
        assert_eq!(ex.equipment, None);
        assert!(ex.images.is_empty());
    }

    // ── WorkoutSet / WorkoutExercise / Workout serialization ─────────────────

    #[test]
    fn workout_set_serde_round_trip() {
        let set = WorkoutSet {
            reps: 10,
            weight_dg: Some(Weight(1000)),
            duration: Some(60),
        };
        let json = serde_json::to_string(&set).unwrap();
        let back: WorkoutSet = serde_json::from_str(&json).unwrap();
        assert_eq!(back, set);
    }

    #[test]
    fn workout_set_without_optionals() {
        let set = WorkoutSet {
            reps: 5,
            weight_dg: None,
            duration: None,
        };
        let json = serde_json::to_string(&set).unwrap();
        let back: WorkoutSet = serde_json::from_str(&json).unwrap();
        assert_eq!(back, set);
    }

    #[test]
    fn workout_exercise_serde_round_trip() {
        let we = WorkoutExercise {
            exercise_id: "ex1".into(),
            exercise_name: "Squat".into(),
            sets: vec![WorkoutSet {
                reps: 5,
                weight_dg: Some(Weight(1000)),
                duration: None,
            }],
            notes: Some("Heavy day".into()),
        };
        let json = serde_json::to_string(&we).unwrap();
        let back: WorkoutExercise = serde_json::from_str(&json).unwrap();
        assert_eq!(back, we);
    }

    #[test]
    fn workout_serde_round_trip() {
        let workout = Workout {
            id: "w1".into(),
            date: "2025-01-01".into(),
            exercises: vec![],
            notes: None,
            version: DATA_VERSION,
        };
        let json = serde_json::to_string(&workout).unwrap();
        let back: Workout = serde_json::from_str(&json).unwrap();
        assert_eq!(back, workout);
    }

    // ── parse function edge cases ────────────────────────────────────────────

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

    // ── get_current_timestamp ────────────────────────────────────────────────

    #[test]
    fn get_current_timestamp_returns_reasonable_value() {
        let ts = get_current_timestamp();
        // Should be after 2020-01-01 (1577836800)
        assert!(ts > 1_577_836_800);
        // Should be before 2100-01-01 (4102444800)
        assert!(ts < 4_102_444_800);
    }

    // ── ExerciseLog with saturating subtraction ──────────────────────────────

    #[test]
    fn exercise_log_duration_saturates_on_underflow() {
        let log = ExerciseLog {
            exercise_id: "ex1".into(),
            exercise_name: "Bench".into(),
            category: Category::Strength,
            start_time: 2000,
            end_time: Some(1000), // end before start
            weight_dg: None,
            reps: None,
            distance_dam: None,
            force: None,
        };
        // saturating_sub should return 0 instead of wrapping
        assert_eq!(log.duration_seconds(), Some(0));
    }

    // ── ExerciseLog serialization round-trip ─────────────────────────────────

    #[test]
    fn exercise_log_serde_round_trip_with_all_fields() {
        let log = ExerciseLog {
            exercise_id: "ex1".into(),
            exercise_name: "Squat".into(),
            category: Category::Strength,
            start_time: 1000,
            end_time: Some(1120),
            weight_dg: Some(Weight(1000)),
            reps: Some(5),
            distance_dam: Some(Distance(50)),
            force: Some(Force::Push),
        };
        let json = serde_json::to_string(&log).unwrap();
        let back: ExerciseLog = serde_json::from_str(&json).unwrap();
        assert_eq!(back, log);
    }

    #[test]
    fn exercise_log_force_none_is_omitted_in_json() {
        let log = ExerciseLog {
            exercise_id: "ex1".into(),
            exercise_name: "Run".into(),
            category: Category::Cardio,
            start_time: 1000,
            end_time: Some(2000),
            weight_dg: None,
            reps: None,
            distance_dam: Some(Distance(500)),
            force: None,
        };
        let json = serde_json::to_string(&log).unwrap();
        assert!(!json.contains("force"));
    }

    // ── Exercise with all None optionals ─────────────────────────────────────

    #[test]
    fn exercise_minimal() {
        let ex = Exercise {
            id: "custom_1".into(),
            name: "My Exercise".into(),
            category: Category::Strength,
            force: None,
            level: None,
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec![],
            images: vec![],
        };
        let json = serde_json::to_string(&ex).unwrap();
        let back: Exercise = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ex);
    }

    // ── format_time edge cases ───────────────────────────────────────────────

    #[test]
    fn format_time_boundary_values() {
        assert_eq!(format_time(1), "00:01");
        assert_eq!(format_time(59), "00:59");
        assert_eq!(format_time(60), "01:00");
        assert_eq!(format_time(3599), "59:59");
        assert_eq!(format_time(3600), "01:00:00");
        assert_eq!(format_time(86399), "23:59:59");
    }

    // ── WorkoutSession serialization ─────────────────────────────────────────

    #[test]
    fn workout_session_with_exercise_logs_serde() {
        let session = WorkoutSession {
            id: "s1".into(),
            start_time: 1000,
            end_time: Some(2000),
            exercise_logs: vec![ExerciseLog {
                exercise_id: "ex1".into(),
                exercise_name: "Squat".into(),
                category: Category::Strength,
                start_time: 1000,
                end_time: Some(1120),
                weight_dg: Some(Weight(1000)),
                reps: Some(5),
                distance_dam: None,
                force: Some(Force::Push),
            }],
            version: DATA_VERSION,
            pending_exercise_ids: vec![],
            rest_start_time: None,
        };
        let json = serde_json::to_string(&session).unwrap();
        let back: WorkoutSession = serde_json::from_str(&json).unwrap();
        assert_eq!(back, session);
        assert_eq!(back.exercise_logs.len(), 1);
        assert_eq!(back.exercise_logs[0].exercise_name, "Squat");
    }
}
