use dioxus::prelude::*;
use crate::models::{Workout, WorkoutExercise, WorkoutSet, DATA_VERSION, get_current_timestamp, format_weight, parse_weight_kg};
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
        let weight_dg = parse_weight_kg(&weight_input.read());
        
        let mut exercises = workout_exercises.write();
        if let Some(exercise) = exercises.iter_mut().find(|e| e.exercise_id == exercise_id) {
            exercise.sets.push(WorkoutSet {
                reps,
                weight_dg,
                duration: None,
            });
        }
    };

    let save_workout = move |_| {
        let exercises = workout_exercises.read().clone();
        if !exercises.is_empty() {
            let timestamp = get_current_timestamp();
            let workout = Workout {
                id: format!("workout_{}", timestamp),
                date: format!("{}", timestamp),
                exercises,
                notes: None,
                version: DATA_VERSION,
            };
            storage::add_workout(workout);
            workout_exercises.set(vec![]);
        }
    };

    rsx! {
        div {
            class: "container container--narrow",
            
            header {
                class: "page-header",
                Link {
                    to: Route::HomePage {},
                    class: "back-link",
                    "← Back"
                }
                h1 { class: "page-title", "Log Your Workout" }
            }
            
            // Exercise Search
            div {
                class: "form-group",
                h3 { "Add Exercise" }
                input {
                    r#type: "text",
                    placeholder: "Search for an exercise...",
                    value: "{search_query}",
                    oninput: move |evt| search_query.set(evt.value()),
                    class: "search-input",
                }
                
                if !search_results().is_empty() {
                    div {
                        class: "search-results",
                        for exercise in search_results() {
                            div {
                                key: "{exercise.id}",
                                onclick: move |_| add_exercise(exercise.id.clone(), exercise.name.clone()),
                                class: "search-result-item",
                                "{exercise.name}"
                            }
                        }
                    }
                }
            }
            
            // Current Workout
            if !workout_exercises.read().is_empty() {
                div {
                    class: "form-group",
                    h3 { "Current Workout" }
                    
                    for exercise in workout_exercises.read().iter() {
                        div {
                            key: "{exercise.exercise_id}",
                            class: "workout-exercise-card",
                            
                            h4 { class: "workout-exercise-card__title", "{exercise.exercise_name}" }
                            
                            if !exercise.sets.is_empty() {
                                div {
                                    for (idx, set) in exercise.sets.iter().enumerate() {
                                        div {
                                            key: "{idx}",
                                            class: "set-line",
                                            "Set {idx + 1}: {set.reps} reps"
                                            if let Some(w) = set.weight_dg {
                                                " @ {format_weight(w)}"
                                            }
                                        }
                                    }
                                }
                            }
                            
                            div {
                                class: "set-inputs",
                                input {
                                    r#type: "number",
                                    placeholder: "Reps",
                                    value: "{reps_input}",
                                    oninput: move |evt| reps_input.set(evt.value()),
                                    class: "input-small",
                                }
                                input {
                                    r#type: "number",
                                    placeholder: "Weight (kg)",
                                    value: "{weight_input}",
                                    oninput: move |evt| weight_input.set(evt.value()),
                                    class: "input-medium",
                                }
                                button {
                                    onclick: {
                                        let exercise_id = exercise.exercise_id.clone();
                                        move |_| add_set_to_exercise(exercise_id.clone())
                                    },
                                    class: "btn btn--accent",
                                    "Add Set"
                                }
                            }
                        }
                    }
                    
                    button {
                        onclick: save_workout,
                        class: "btn btn--primary",
                        "💾 Save Workout"
                    }
                }
            }
        }
    }
}
