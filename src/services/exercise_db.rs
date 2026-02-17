use crate::models::Exercise;
use std::sync::OnceLock;

static EXERCISES: OnceLock<Vec<Exercise>> = OnceLock::new();

pub fn get_exercises() -> &'static Vec<Exercise> {
    EXERCISES.get_or_init(|| {
        let exercises_json = include_str!("../../assets/exercises.json");
        serde_json::from_str(exercises_json).unwrap_or_else(|e| {
            eprintln!("Failed to parse exercises: {}", e);
            vec![]
        })
    })
}

pub fn search_exercises(query: &str) -> Vec<&Exercise> {
    let query_lower = query.to_lowercase();
    get_exercises()
        .iter()
        .filter(|exercise| {
            exercise.name.to_lowercase().contains(&query_lower)
                || exercise
                    .primary_muscles
                    .iter()
                    .any(|m| m.to_lowercase().contains(&query_lower))
                || exercise.category.to_lowercase().contains(&query_lower)
        })
        .collect()
}

pub fn get_exercise_by_id(id: &str) -> Option<&Exercise> {
    get_exercises().iter().find(|e| e.id == id)
}

pub fn get_categories() -> Vec<String> {
    let mut categories: Vec<String> = get_exercises()
        .iter()
        .map(|e| e.category.clone())
        .collect();
    categories.sort();
    categories.dedup();
    categories
}

pub fn get_equipment_types() -> Vec<String> {
    let mut equipment: Vec<String> = get_exercises()
        .iter()
        .filter_map(|e| e.equipment.clone())
        .collect();
    equipment.sort();
    equipment.dedup();
    equipment
}

pub fn get_muscle_groups() -> Vec<String> {
    let mut muscles: Vec<String> = get_exercises()
        .iter()
        .flat_map(|e| e.primary_muscles.clone())
        .collect();
    muscles.sort();
    muscles.dedup();
    muscles
}
