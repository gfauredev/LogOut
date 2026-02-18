use std::fmt;
use serde::{Deserialize, Serialize};

// Base URL for exercise images from the free-exercise-db repository
const EXERCISES_IMAGE_BASE_URL: &str = "https://raw.githubusercontent.com/yuhonas/free-exercise-db/main/exercises/";

// Version control for data structures to handle migrations
pub const DATA_VERSION: u32 = 3;

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

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cardio => write!(f, "cardio"),
            Self::OlympicWeightlifting => write!(f, "olympic weightlifting"),
            Self::Plyometrics => write!(f, "plyometrics"),
            Self::Powerlifting => write!(f, "powerlifting"),
            Self::Strength => write!(f, "strength"),
            Self::Stretching => write!(f, "stretching"),
            Self::Strongman => write!(f, "strongman"),
        }
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

impl fmt::Display for Force {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pull => write!(f, "pull"),
            Self::Push => write!(f, "push"),
            Self::Static => write!(f, "static"),
        }
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

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Beginner => write!(f, "beginner"),
            Self::Intermediate => write!(f, "intermediate"),
            Self::Expert => write!(f, "expert"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mechanic {
    #[serde(rename = "compound")]
    Compound,
    #[serde(rename = "isolation")]
    Isolation,
}

impl fmt::Display for Mechanic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compound => write!(f, "compound"),
            Self::Isolation => write!(f, "isolation"),
        }
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

impl fmt::Display for Equipment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bands => write!(f, "bands"),
            Self::Barbell => write!(f, "barbell"),
            Self::BodyOnly => write!(f, "body only"),
            Self::Cable => write!(f, "cable"),
            Self::Dumbbell => write!(f, "dumbbell"),
            Self::EzCurlBar => write!(f, "e-z curl bar"),
            Self::ExerciseBall => write!(f, "exercise ball"),
            Self::FoamRoll => write!(f, "foam roll"),
            Self::Kettlebells => write!(f, "kettlebells"),
            Self::Machine => write!(f, "machine"),
            Self::MedicineBall => write!(f, "medicine ball"),
            Self::Other => write!(f, "other"),
        }
    }
}

impl Equipment {
    pub const ALL: &'static [Equipment] = &[
        Self::Bands, Self::Barbell, Self::BodyOnly, Self::Cable,
        Self::Dumbbell, Self::EzCurlBar, Self::ExerciseBall, Self::FoamRoll,
        Self::Kettlebells, Self::Machine, Self::MedicineBall, Self::Other,
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

impl fmt::Display for Muscle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Abdominals => write!(f, "abdominals"),
            Self::Abductors => write!(f, "abductors"),
            Self::Adductors => write!(f, "adductors"),
            Self::Biceps => write!(f, "biceps"),
            Self::Calves => write!(f, "calves"),
            Self::Chest => write!(f, "chest"),
            Self::Forearms => write!(f, "forearms"),
            Self::Glutes => write!(f, "glutes"),
            Self::Hamstrings => write!(f, "hamstrings"),
            Self::Lats => write!(f, "lats"),
            Self::LowerBack => write!(f, "lower back"),
            Self::MiddleBack => write!(f, "middle back"),
            Self::Neck => write!(f, "neck"),
            Self::Quadriceps => write!(f, "quadriceps"),
            Self::Shoulders => write!(f, "shoulders"),
            Self::Traps => write!(f, "traps"),
            Self::Triceps => write!(f, "triceps"),
        }
    }
}

impl Muscle {
    pub const ALL: &'static [Muscle] = &[
        Self::Abdominals, Self::Abductors, Self::Adductors, Self::Biceps,
        Self::Calves, Self::Chest, Self::Forearms, Self::Glutes,
        Self::Hamstrings, Self::Lats, Self::LowerBack, Self::MiddleBack,
        Self::Neck, Self::Quadriceps, Self::Shoulders, Self::Traps, Self::Triceps,
    ];
}

// ── Weight and Distance value types ─────────────────────────────────────────

/// Weight stored as decagrams (10 g units). 1 kg = 100 dg.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Weight(pub u16);

impl fmt::Display for Weight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 % 100 == 0 {
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
            if self.0 % 100 == 0 {
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
    if !val.is_finite() || val <= 0.0 { return None; }
    let dg = (val * 100.0).round();
    if dg < 0.0 || dg > u16::MAX as f64 { return None; }
    Some(Weight(dg as u16))
}

/// Parse a user-entered km string into a Distance (decameters).
pub fn parse_distance_km(input: &str) -> Option<Distance> {
    let val: f64 = input.parse().ok()?;
    if !val.is_finite() || val <= 0.0 { return None; }
    let dam = (val * 100.0).round();
    if dam < 0.0 || dam > u16::MAX as f64 { return None; }
    Some(Distance(dam as u16))
}

// ── Data structures ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Exercise {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force: Option<Force>,
    pub level: Level,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mechanic: Option<Mechanic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equipment: Option<Equipment>,
    #[serde(rename = "primaryMuscles")]
    pub primary_muscles: Vec<Muscle>,
    #[serde(rename = "secondaryMuscles")]
    pub secondary_muscles: Vec<Muscle>,
    pub instructions: Vec<String>,
    pub category: Category,
    pub images: Vec<String>,
}

impl Exercise {
    /// Get the URL for a specific image by index
    pub fn get_image_url(&self, index: usize) -> Option<String> {
        self.images
            .get(index)
            .map(|img| format!("{}{}", EXERCISES_IMAGE_BASE_URL, img))
    }

    /// Get the first image URL if available
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
    pub version: u32,
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
    pub category: Category,
    pub force: Option<Force>,
    pub equipment: Option<Equipment>,
    pub primary_muscles: Vec<Muscle>,
    #[serde(default)]
    pub secondary_muscles: Vec<Muscle>,
    #[serde(default)]
    pub instructions: Vec<String>,
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

    #[test]
    fn find_active_session_returns_first_without_end_time() {
        let sessions = vec![
            WorkoutSession {
                id: "s1".into(),
                start_time: 1000,
                end_time: Some(2000),
                exercise_logs: vec![],
                version: DATA_VERSION,
            },
            WorkoutSession {
                id: "s2".into(),
                start_time: 3000,
                end_time: None,
                exercise_logs: vec![],
                version: DATA_VERSION,
            },
        ];
        let active = sessions.iter().find(|s| s.is_active()).cloned();
        assert_eq!(active.unwrap().id, "s2");
    }

    #[test]
    fn find_active_session_returns_none_when_all_finished() {
        let sessions = vec![
            WorkoutSession {
                id: "s1".into(),
                start_time: 1000,
                end_time: Some(2000),
                exercise_logs: vec![],
                version: DATA_VERSION,
            },
        ];
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
            level: Level::Beginner,
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
            Some("https://raw.githubusercontent.com/yuhonas/free-exercise-db/main/exercises/Squat/0.jpg".into())
        );
    }

    #[test]
    fn exercise_get_first_image_url_none() {
        let ex = Exercise {
            id: "ex1".into(),
            name: "Squat".into(),
            force: None,
            level: Level::Beginner,
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

    // ── CustomExercise ────────────────────────────────────────────────────────

    #[test]
    fn custom_exercise_serialization_with_new_fields() {
        let exercise = CustomExercise {
            id: "custom_123".into(),
            name: "Test Exercise".into(),
            category: Category::Strength,
            force: Some(Force::Push),
            equipment: Some(Equipment::Barbell),
            primary_muscles: vec![Muscle::Chest],
            secondary_muscles: vec![Muscle::Triceps, Muscle::Shoulders],
            instructions: vec!["Step 1".into(), "Step 2".into()],
        };
        let json = serde_json::to_string(&exercise).unwrap();
        let deserialized: CustomExercise = serde_json::from_str(&json).unwrap();
        assert_eq!(exercise, deserialized);
        assert_eq!(deserialized.secondary_muscles.len(), 2);
        assert_eq!(deserialized.instructions.len(), 2);
    }

    #[test]
    fn custom_exercise_backward_compat_missing_new_fields() {
        // Old format without secondary_muscles and instructions
        let json = r#"{"id":"custom_1","name":"Old Exercise","category":"strength","force":"push","equipment":"barbell","primary_muscles":["chest"]}"#;
        let exercise: CustomExercise = serde_json::from_str(json).unwrap();
        assert_eq!(exercise.secondary_muscles, Vec::<Muscle>::new());
        assert_eq!(exercise.instructions, Vec::<String>::new());
    }
}
