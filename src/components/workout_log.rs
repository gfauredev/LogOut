use dioxus::prelude::*;
use crate::models::{Workout, WorkoutExercise, WorkoutSet};
use crate::services::{exercise_db, storage};
use crate::Route;

#[component]
pub fn WorkoutLogPage() -> Element {
    let mut workout_exercises = use_signal(|| Vec::<WorkoutExercise>::new());
    let mut search_query = use_signal(|| String::new());
    let mut selected_exercise = use_signal(|| None::<String>);
    let mut reps_input = use_signal(|| String::from("10"));
    let mut weight_input = use_signal(|| String::from("0"));
    
    let search_results = use_memo(move || {
        let query = search_query.read();
        if query.is_empty() {
            vec![]
        } else {
            exercise_db::search_exercises(&query)
                .into_iter()
                .take(10)
                .cloned()
                .collect::<Vec<_>>()
        }
    });

    let mut add_exercise = move |exercise_id: String, exercise_name: String| {
        let mut exercises = workout_exercises.write();
        if !exercises.iter().any(|e| e.exercise_id == exercise_id) {
            exercises.push(WorkoutExercise {
                exercise_id: exercise_id.clone(),
                exercise_name,
                sets: vec![],
                notes: None,
            });
        }
        selected_exercise.set(Some(exercise_id));
        search_query.set(String::new());
    };

    let mut add_set_to_exercise = move |exercise_id: String| {
        let reps: u32 = reps_input.read().parse().unwrap_or(10);
        let weight: f32 = weight_input.read().parse().unwrap_or(0.0);
        
        let mut exercises = workout_exercises.write();
        if let Some(exercise) = exercises.iter_mut().find(|e| e.exercise_id == exercise_id) {
            exercise.sets.push(WorkoutSet {
                reps,
                weight: if weight > 0.0 { Some(weight) } else { None },
                duration: None,
            });
        }
    };

    let save_workout = move |_| {
        let exercises = workout_exercises.read().clone();
        if !exercises.is_empty() {
            use std::time::{SystemTime, UNIX_EPOCH};
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let workout = Workout {
                id: format!("workout_{}", timestamp),
                date: format!("{}", timestamp),
                exercises,
                notes: None,
            };
            storage::add_workout(workout);
            workout_exercises.set(vec![]);
        }
    };

    rsx! {
        div {
            class: "container",
            style: "padding: 20px; font-family: system-ui, -apple-system, sans-serif; max-width: 800px; margin: 0 auto;",
            
            header {
                style: "margin-bottom: 20px;",
                Link {
                    to: Route::HomePage {},
                    style: "text-decoration: none; color: #667eea; font-size: 1.1em;",
                    "← Back"
                }
                h1 { 
                    style: "margin: 15px 0;",
                    "Log Your Workout" 
                }
            }
            
            // Exercise Search
            div {
                style: "margin-bottom: 20px;",
                h3 { "Add Exercise" }
                input {
                    r#type: "text",
                    placeholder: "Search for an exercise...",
                    value: "{search_query}",
                    oninput: move |evt| search_query.set(evt.value()),
                    style: "
                        width: 100%;
                        padding: 12px;
                        font-size: 16px;
                        border: 2px solid #e0e0e0;
                        border-radius: 8px;
                        box-sizing: border-box;
                    ",
                }
                
                if !search_results().is_empty() {
                    div {
                        style: "
                            margin-top: 10px;
                            border: 1px solid #e0e0e0;
                            border-radius: 8px;
                            background: white;
                            max-height: 200px;
                            overflow-y: auto;
                        ",
                        for exercise in search_results() {
                            div {
                                key: "{exercise.id}",
                                onclick: move |_| add_exercise(exercise.id.clone(), exercise.name.clone()),
                                style: "
                                    padding: 10px;
                                    cursor: pointer;
                                    border-bottom: 1px solid #f0f0f0;
                                ",
                                "{exercise.name}"
                            }
                        }
                    }
                }
            }
            
            // Current Workout
            if !workout_exercises.read().is_empty() {
                div {
                    style: "margin-bottom: 20px;",
                    h3 { "Current Workout" }
                    
                    for exercise in workout_exercises.read().iter() {
                        div {
                            key: "{exercise.exercise_id}",
                            style: "
                                padding: 15px;
                                margin-bottom: 15px;
                                border: 1px solid #e0e0e0;
                                border-radius: 8px;
                                background: white;
                            ",
                            
                            h4 { 
                                style: "margin: 0 0 10px 0;",
                                "{exercise.exercise_name}" 
                            }
                            
                            if !exercise.sets.is_empty() {
                                div {
                                    style: "margin-bottom: 10px;",
                                    for (idx, set) in exercise.sets.iter().enumerate() {
                                        div {
                                            key: "{idx}",
                                            style: "padding: 5px 0; color: #666;",
                                            "Set {idx + 1}: {set.reps} reps"
                                            if let Some(weight) = set.weight {
                                                " @ {weight} lbs"
                                            }
                                        }
                                    }
                                }
                            }
                            
                            div {
                                style: "display: flex; gap: 10px; align-items: center;",
                                input {
                                    r#type: "number",
                                    placeholder: "Reps",
                                    value: "{reps_input}",
                                    oninput: move |evt| reps_input.set(evt.value()),
                                    style: "
                                        padding: 8px;
                                        border: 1px solid #e0e0e0;
                                        border-radius: 4px;
                                        width: 80px;
                                    ",
                                }
                                input {
                                    r#type: "number",
                                    placeholder: "Weight (lbs)",
                                    value: "{weight_input}",
                                    oninput: move |evt| weight_input.set(evt.value()),
                                    style: "
                                        padding: 8px;
                                        border: 1px solid #e0e0e0;
                                        border-radius: 4px;
                                        width: 100px;
                                    ",
                                }
                                button {
                                    onclick: {
                                        let exercise_id = exercise.exercise_id.clone();
                                        move |_| add_set_to_exercise(exercise_id.clone())
                                    },
                                    style: "
                                        padding: 8px 16px;
                                        background: #4facfe;
                                        color: white;
                                        border: none;
                                        border-radius: 4px;
                                        cursor: pointer;
                                        font-weight: bold;
                                    ",
                                    "Add Set"
                                }
                            }
                        }
                    }
                    
                    button {
                        onclick: save_workout,
                        style: "
                            width: 100%;
                            padding: 15px;
                            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                            color: white;
                            border: none;
                            border-radius: 8px;
                            font-size: 1.2em;
                            font-weight: bold;
                            cursor: pointer;
                            margin-top: 10px;
                        ",
                        "💾 Save Workout"
                    }
                }
            }
        }
    }
}
