use serde::{Deserialize, Serialize};

// Base URL for exercise images from the free-exercise-db repository
const EXERCISES_IMAGE_BASE_URL: &str = "https://raw.githubusercontent.com/yuhonas/free-exercise-db/main/exercises/";

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
    /// Get full URLs for exercise images
    pub fn get_image_urls(&self) -> Vec<String> {
        self.images
            .iter()
            .map(|img| format!("{}{}", EXERCISES_IMAGE_BASE_URL, img))
            .collect()
    }

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
    pub weight: Option<f32>,
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
}

// New models for active session tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExerciseLog {
    pub exercise_id: String,
    pub exercise_name: String,
    pub category: String,
    pub start_time: u64,  // Unix timestamp in seconds
    pub end_time: Option<u64>,  // Unix timestamp in seconds
    pub weight: Option<f32>,
    pub reps: Option<u32>,  // For strength exercises
    pub distance: Option<f32>,  // For cardio exercises (in km or miles)
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
    pub start_time: u64,  // Unix timestamp in seconds
    pub end_time: Option<u64>,  // Unix timestamp in seconds
    pub exercise_logs: Vec<ExerciseLog>,
}

impl WorkoutSession {
    /// Create a new workout session
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            id: format!("session_{}", timestamp),
            start_time: timestamp,
            end_time: None,
            exercise_logs: Vec::new(),
        }
    }

    /// Get total duration in seconds
    pub fn duration_seconds(&self) -> Option<u64> {
        self.end_time.map(|end| end.saturating_sub(self.start_time))
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
