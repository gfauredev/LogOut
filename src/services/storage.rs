use crate::models::{Workout, WorkoutSession, ExerciseLog, CustomExercise, DATA_VERSION};
use std::sync::Mutex;

#[cfg(target_arch = "wasm32")]
use log::{error, warn, info};

static WORKOUTS: Mutex<Vec<Workout>> = Mutex::new(Vec::new());
static SESSIONS: Mutex<Vec<WorkoutSession>> = Mutex::new(Vec::new());
static CUSTOM_EXERCISES: Mutex<Vec<CustomExercise>> = Mutex::new(Vec::new());

const WORKOUTS_KEY: &str = "logout_workouts";
const SESSIONS_KEY: &str = "logout_sessions";
const CUSTOM_EXERCISES_KEY: &str = "logout_custom_exercises";

pub fn init_storage() {
    // Load from localStorage on web
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                info!("Initializing storage from localStorage");
                
                // Load workouts with validation
                if let Ok(Some(data)) = storage.get_item(WORKOUTS_KEY) {
                    match serde_json::from_str::<Vec<Workout>>(&data) {
                        Ok(mut workouts) => {
                            // Migrate old data if needed
                            let migrated = migrate_workouts(&mut workouts);
                            if migrated {
                                info!("Migrated {} workouts to current version", workouts.len());
                            }
                            
                            // Validate exercise references
                            validate_workout_exercises(&mut workouts);
                            
                            let count = workouts.len();
                            *WORKOUTS.lock().unwrap_or_else(|e| e.into_inner()) = workouts;
                            info!("Loaded {} workouts from storage", count);
                        }
                        Err(e) => {
                            error!("Failed to parse workouts from localStorage: {}. Data may be corrupted.", e);
                        }
                    }
                }
                
                // Load sessions with validation
                if let Ok(Some(data)) = storage.get_item(SESSIONS_KEY) {
                    match serde_json::from_str::<Vec<WorkoutSession>>(&data) {
                        Ok(mut sessions) => {
                            // Migrate old data if needed
                            let migrated = migrate_sessions(&mut sessions);
                            if migrated {
                                info!("Migrated {} sessions to current version", sessions.len());
                            }
                            
                            let count = sessions.len();
                            *SESSIONS.lock().unwrap_or_else(|e| e.into_inner()) = sessions;
                            info!("Loaded {} sessions from storage", count);
                        }
                        Err(e) => {
                            error!("Failed to parse sessions from localStorage: {}. Data may be corrupted.", e);
                        }
                    }
                }
                
                // Load custom exercises
                if let Ok(Some(data)) = storage.get_item(CUSTOM_EXERCISES_KEY) {
                    match serde_json::from_str::<Vec<CustomExercise>>(&data) {
                        Ok(exercises) => {
                            let count = exercises.len();
                            *CUSTOM_EXERCISES.lock().unwrap_or_else(|e| e.into_inner()) = exercises;
                            info!("Loaded {} custom exercises from storage", count);
                        }
                        Err(e) => {
                            error!("Failed to parse custom exercises from localStorage: {}. Data may be corrupted.", e);
                        }
                    }
                }
            } else {
                warn!("localStorage is not available");
            }
        }
    }
}

fn save_workouts(workouts: &[Workout]) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                match serde_json::to_string(workouts) {
                    Ok(data) => {
                        if let Err(e) = storage.set_item(WORKOUTS_KEY, &data) {
                            error!("Failed to save workouts to localStorage: {:?}", e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to serialize workouts: {}", e);
                    }
                }
            }
        }
    }
}

fn save_sessions(sessions: &[WorkoutSession]) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                match serde_json::to_string(sessions) {
                    Ok(data) => {
                        if let Err(e) = storage.set_item(SESSIONS_KEY, &data) {
                            error!("Failed to save sessions to localStorage: {:?}", e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to serialize sessions: {}", e);
                    }
                }
            }
        }
    }
}

fn save_custom_exercises(exercises: &[CustomExercise]) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                match serde_json::to_string(exercises) {
                    Ok(data) => {
                        if let Err(e) = storage.set_item(CUSTOM_EXERCISES_KEY, &data) {
                            error!("Failed to save custom exercises to localStorage: {:?}", e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to serialize custom exercises: {}", e);
                    }
                }
            }
        }
    }
}

pub fn get_all_workouts() -> Vec<Workout> {
    WORKOUTS.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

pub fn add_workout(workout: Workout) {
    let mut workouts = WORKOUTS.lock().unwrap_or_else(|e| e.into_inner());
    workouts.push(workout.clone());
    save_workouts(&workouts);
}

pub fn delete_workout(id: &str) {
    let mut workouts = WORKOUTS.lock().unwrap_or_else(|e| e.into_inner());
    workouts.retain(|workout| workout.id != id);
    save_workouts(&workouts);
}

// Session management
pub fn get_all_sessions() -> Vec<WorkoutSession> {
    SESSIONS.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

pub fn get_active_session() -> Option<WorkoutSession> {
    SESSIONS.lock().unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|s| s.is_active())
        .cloned()
}

pub fn save_session(session: WorkoutSession) {
    let mut sessions = SESSIONS.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(pos) = sessions.iter().position(|s| s.id == session.id) {
        sessions[pos] = session;
    } else {
        sessions.push(session);
    }
    save_sessions(&sessions);
}

pub fn delete_session(id: &str) {
    let mut sessions = SESSIONS.lock().unwrap_or_else(|e| e.into_inner());
    sessions.retain(|s| s.id != id);
    save_sessions(&sessions);
}

// Custom exercises management
pub fn get_custom_exercises() -> Vec<CustomExercise> {
    CUSTOM_EXERCISES.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

pub fn add_custom_exercise(exercise: CustomExercise) {
    let mut exercises = CUSTOM_EXERCISES.lock().unwrap_or_else(|e| e.into_inner());
    exercises.push(exercise);
    save_custom_exercises(&exercises);
}

// Helper to get last values for an exercise (for prefilling)
pub fn get_last_exercise_log(exercise_id: &str) -> Option<ExerciseLog> {
    let sessions = SESSIONS.lock().unwrap_or_else(|e| e.into_inner());
    
    // Search through sessions in reverse order (most recent first)
    for session in sessions.iter().rev() {
        for log in session.exercise_logs.iter().rev() {
            if log.exercise_id == exercise_id && log.is_complete() {
                return Some(log.clone());
            }
        }
    }
    None
}

/// Migrate workouts to current data version
/// Returns true if any migrations were performed
#[cfg(target_arch = "wasm32")]
fn migrate_workouts(workouts: &mut Vec<Workout>) -> bool {
    let mut migrated = false;
    
    for workout in workouts.iter_mut() {
        if workout.version == 0 {
            // Migration from version 0 to 1
            workout.version = DATA_VERSION;
            migrated = true;
        }
        // Future migrations can be added here
        // if workout.version == 1 { ... }
    }
    
    migrated
}

/// Migrate workout sessions to current data version
/// Returns true if any migrations were performed
#[cfg(target_arch = "wasm32")]
fn migrate_sessions(sessions: &mut Vec<WorkoutSession>) -> bool {
    let mut migrated = false;
    
    for session in sessions.iter_mut() {
        if session.version == 0 {
            // Migration from version 0 to 1
            session.version = DATA_VERSION;
            migrated = true;
        }
        // Future migrations can be added here
    }
    
    migrated
}

/// Validate that all exercise references in workouts exist in the exercise database
/// or in custom exercises. Log warnings for orphaned references.
#[cfg(target_arch = "wasm32")]
fn validate_workout_exercises(workouts: &mut Vec<Workout>) {
    use crate::services::exercise_db;
    
    let custom_exercises = CUSTOM_EXERCISES.lock().unwrap_or_else(|e| e.into_inner());
    let mut orphaned_count = 0;
    
    for workout in workouts.iter() {
        for exercise in workout.exercises.iter() {
            let exists_in_db = exercise_db::get_exercise_by_id(&exercise.exercise_id).is_some();
            let exists_in_custom = custom_exercises.iter().any(|ce| ce.id == exercise.exercise_id);
            
            if !exists_in_db && !exists_in_custom {
                warn!(
                    "Workout '{}' references non-existent exercise '{}' (ID: {}). \
                    This may happen if an exercise was removed from the database after you logged it. \
                    Your workout data is safe and the exercise name '{}' is preserved.",
                    workout.id, exercise.exercise_name, exercise.exercise_id, exercise.exercise_name
                );
                orphaned_count += 1;
            }
        }
    }
    
    if orphaned_count > 0 {
        warn!(
            "Found {} orphaned exercise reference(s) in workouts. \
            These exercises may have been removed or renamed in the exercise database. \
            Your workout history is preserved with the original exercise names.",
            orphaned_count
        );
    }
}
