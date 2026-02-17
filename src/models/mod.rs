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
