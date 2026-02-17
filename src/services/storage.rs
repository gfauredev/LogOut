use crate::models::Workout;
use std::sync::Mutex;

static WORKOUTS: Mutex<Option<Vec<Workout>>> = Mutex::new(None);

pub fn init_storage() {
    let mut workouts = WORKOUTS.lock().unwrap();
    if workouts.is_none() {
        *workouts = Some(load_workouts());
    }
}

fn load_workouts() -> Vec<Workout> {
    // In a real app, this would load from local storage
    // For now, return an empty vec
    vec![]
}

fn save_workouts(workouts: &[Workout]) {
    // In a real app, this would save to local storage
    // For now, just keep in memory
    let _ = workouts;
}

pub fn get_all_workouts() -> Vec<Workout> {
    let workouts = WORKOUTS.lock().unwrap();
    workouts.as_ref().unwrap_or(&vec![]).clone()
}

pub fn add_workout(workout: Workout) {
    let mut workouts = WORKOUTS.lock().unwrap();
    if let Some(ref mut w) = *workouts {
        w.push(workout.clone());
        save_workouts(w);
    }
}

pub fn delete_workout(id: &str) {
    let mut workouts = WORKOUTS.lock().unwrap();
    if let Some(ref mut w) = *workouts {
        w.retain(|workout| workout.id != id);
        save_workouts(w);
    }
}
