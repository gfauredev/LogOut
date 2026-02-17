use crate::models::{Workout, WorkoutSession, ExerciseLog, CustomExercise};
use std::sync::Mutex;

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
                // Load workouts
                if let Ok(Some(data)) = storage.get_item(WORKOUTS_KEY) {
                    if let Ok(workouts) = serde_json::from_str::<Vec<Workout>>(&data) {
                        *WORKOUTS.lock().unwrap_or_else(|e| e.into_inner()) = workouts;
                    }
                }
                // Load sessions
                if let Ok(Some(data)) = storage.get_item(SESSIONS_KEY) {
                    if let Ok(sessions) = serde_json::from_str::<Vec<WorkoutSession>>(&data) {
                        *SESSIONS.lock().unwrap_or_else(|e| e.into_inner()) = sessions;
                    }
                }
                // Load custom exercises
                if let Ok(Some(data)) = storage.get_item(CUSTOM_EXERCISES_KEY) {
                    if let Ok(exercises) = serde_json::from_str::<Vec<CustomExercise>>(&data) {
                        *CUSTOM_EXERCISES.lock().unwrap_or_else(|e| e.into_inner()) = exercises;
                    }
                }
            }
        }
    }
}

fn save_workouts(workouts: &[Workout]) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(data) = serde_json::to_string(workouts) {
                    let _ = storage.set_item(WORKOUTS_KEY, &data);
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
                if let Ok(data) = serde_json::to_string(sessions) {
                    let _ = storage.set_item(SESSIONS_KEY, &data);
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
                if let Ok(data) = serde_json::to_string(exercises) {
                    let _ = storage.set_item(CUSTOM_EXERCISES_KEY, &data);
                }
            }
        }
    }
}

#[allow(dead_code)]
pub fn get_all_workouts() -> Vec<Workout> {
    WORKOUTS.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

pub fn add_workout(workout: Workout) {
    let mut workouts = WORKOUTS.lock().unwrap_or_else(|e| e.into_inner());
    workouts.push(workout.clone());
    save_workouts(&workouts);
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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
