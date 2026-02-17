use crate::models::Workout;
use std::sync::Mutex;

static WORKOUTS: Mutex<Vec<Workout>> = Mutex::new(Vec::new());

pub fn init_storage() {
    // Storage is already initialized with an empty vec
    // In a production app, this would load from persistent storage
}

fn save_workouts(_workouts: &[Workout]) {
    // In a real app, this would save to local storage
    // For now, just keep in memory
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
