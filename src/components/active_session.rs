use dioxus::prelude::*;
use crate::models::{WorkoutSession, ExerciseLog, CustomExercise};
use crate::services::{exercise_db, storage};
use crate::Route;
use std::time::{SystemTime, UNIX_EPOCH};

#[component]
pub fn ActiveSessionPage() -> Element {
    let mut session = use_signal(|| {
        storage::get_active_session().unwrap_or_else(WorkoutSession::new)
    });
    
    let mut search_query = use_signal(|| String::new());
    let mut current_exercise_id = use_signal(|| None::<String>);
    let mut weight_input = use_signal(|| String::new());
    let mut reps_input = use_signal(|| String::new());
    let mut distance_input = use_signal(|| String::new());
    let mut session_time = use_signal(|| 0u64);
    
    // Timer effect - updates every second
    use_effect(move || {
        let current_session = session.read().clone();
        if current_session.is_active() {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            session_time.set(now - current_session.start_time);
        }
    });
    
    let search_results = use_memo(move || {
        let query = search_query.read();
        if query.is_empty() {
            vec![]
        } else {
            let mut results = Vec::new();
            
            // Search database exercises
            let db_exercises = exercise_db::search_exercises(&query);
            for ex in db_exercises.iter().take(10) {
                results.push((ex.id.clone(), ex.name.clone(), ex.category.clone()));
            }
            
            // Search custom exercises
            let custom = storage::get_custom_exercises();
            for ex in custom.iter() {
                if ex.name.to_lowercase().contains(&query.to_lowercase()) {
                    results.push((ex.id.clone(), ex.name.clone(), ex.category.clone()));
                }
            }
            
            results
        }
    });
    
    let mut start_exercise = move |exercise_id: String, exercise_name: String, _category: String| {
        // Look up last values for prefilling
        if let Some(last_log) = storage::get_last_exercise_log(&exercise_id) {
            if let Some(weight) = last_log.weight {
                weight_input.set(weight.to_string());
            }
            if let Some(reps) = last_log.reps {
                reps_input.set(reps.to_string());
            }
            if let Some(distance) = last_log.distance {
                distance_input.set(distance.to_string());
            }
        } else {
            // Clear inputs if no previous data
            weight_input.set(String::new());
            reps_input.set(String::new());
            distance_input.set(String::new());
        }
        
        current_exercise_id.set(Some(exercise_id.clone()));
        search_query.set(String::new());
    };
    
    let complete_exercise = move |_| {
        let exercise_id = match current_exercise_id.read().as_ref() {
            Some(id) => id.clone(),
            None => return,
        };
        
        let mut current_session = session.read().clone();
        
        // Find the exercise info
        let (exercise_name, category) = if let Some(ex) = exercise_db::get_exercise_by_id(&exercise_id) {
            (ex.name.clone(), ex.category.clone())
        } else {
            // Check custom exercises
            let custom = storage::get_custom_exercises();
            if let Some(ex) = custom.iter().find(|e| e.id == exercise_id) {
                (ex.name.clone(), ex.category.clone())
            } else {
                return; // Exercise not found
            }
        };
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let weight = weight_input.read().parse().ok();
        let reps = if category.to_lowercase() == "cardio" {
            None
        } else {
            reps_input.read().parse().ok()
        };
        let distance = if category.to_lowercase() == "cardio" {
            distance_input.read().parse().ok()
        } else {
            None
        };
        
        let log = ExerciseLog {
            exercise_id: exercise_id.clone(),
            exercise_name,
            category,
            start_time: now,  // For simplicity, using current time as both start and end
            end_time: Some(now),
            weight,
            reps,
            distance,
        };
        
        current_session.exercise_logs.push(log);
        storage::save_session(current_session.clone());
        session.set(current_session);
        
        current_exercise_id.set(None);
        weight_input.set(String::new());
        reps_input.set(String::new());
        distance_input.set(String::new());
    };
    
    let finish_session = move |_| {
        let mut current_session = session.read().clone();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        current_session.end_time = Some(now);
        storage::save_session(current_session.clone());
        
        // Navigate back to home
        navigator().push(Route::HomePage {});
    };
    
    let format_time = |seconds: u64| -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;
        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, minutes, secs)
        } else {
            format!("{:02}:{:02}", minutes, secs)
        }
    };

    rsx! {
        div {
            class: "container",
            style: "font-family: system-ui, -apple-system, sans-serif;",
            
            // Sticky timer header
            div {
                style: "
                    position: sticky;
                    top: 0;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    color: white;
                    padding: 15px 20px;
                    box-shadow: 0 2px 10px rgba(0,0,0,0.1);
                    z-index: 100;
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                ",
                div {
                    h2 { 
                        style: "margin: 0; font-size: 1.5em;",
                        "⏱️ Active Session"
                    }
                    p {
                        style: "margin: 5px 0 0 0; font-size: 1.8em; font-weight: bold;",
                        "{format_time(session_time())}"
                    }
                }
                button {
                    onclick: finish_session,
                    style: "
                        padding: 12px 24px;
                        background: white;
                        color: #667eea;
                        border: none;
                        border-radius: 8px;
                        font-size: 1.1em;
                        font-weight: bold;
                        cursor: pointer;
                        box-shadow: 0 2px 4px rgba(0,0,0,0.1);
                    ",
                    "Finish Session"
                }
            }
            
            div {
                style: "padding: 20px; max-width: 800px; margin: 0 auto;",
                
                // Exercise search and selection
                if current_exercise_id.read().is_none() {
                    div {
                        style: "margin-bottom: 20px;",
                        h3 { "Select Exercise" }
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
                                    max-height: 300px;
                                    overflow-y: auto;
                                ",
                                for (id, name, category) in search_results() {
                                    div {
                                        key: "{id}",
                                        onclick: move |_| start_exercise(id.clone(), name.clone(), category.clone()),
                                        style: "
                                            padding: 12px;
                                            cursor: pointer;
                                            border-bottom: 1px solid #f0f0f0;
                                            display: flex;
                                            justify-content: space-between;
                                            align-items: center;
                                        ",
                                        span { "{name}" }
                                        span {
                                            style: "
                                                padding: 4px 10px;
                                                background: #667eea;
                                                color: white;
                                                border-radius: 12px;
                                                font-size: 0.85em;
                                            ",
                                            "{category}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Current exercise input form
                    if let Some(exercise_id) = current_exercise_id.read().as_ref() {
                        div {
                            style: "
                                padding: 20px;
                                border: 2px solid #667eea;
                                border-radius: 12px;
                                background: #f8f9ff;
                                margin-bottom: 20px;
                            ",
                            
                            // Get exercise details
                            {
                                let (exercise_name, category) = if let Some(ex) = exercise_db::get_exercise_by_id(exercise_id) {
                                    (ex.name.clone(), ex.category.clone())
                                } else {
                                    let custom = storage::get_custom_exercises();
                                    if let Some(ex) = custom.iter().find(|e| &e.id == exercise_id) {
                                        (ex.name.clone(), ex.category.clone())
                                    } else {
                                        ("Unknown".to_string(), "unknown".to_string())
                                    }
                                };
                                
                                let is_cardio = category.to_lowercase() == "cardio";
                                
                                rsx! {
                                    h3 { 
                                        style: "margin-top: 0;",
                                        "{exercise_name}"
                                    }
                                    
                                    div {
                                        style: "display: flex; flex-direction: column; gap: 12px;",
                                        
                                        div {
                                            label {
                                                style: "display: block; margin-bottom: 5px; font-weight: bold;",
                                                "Weight (lbs)"
                                            }
                                            input {
                                                r#type: "number",
                                                step: "0.5",
                                                placeholder: "Optional",
                                                value: "{weight_input}",
                                                oninput: move |evt| weight_input.set(evt.value()),
                                                style: "
                                                    width: 100%;
                                                    padding: 10px;
                                                    border: 1px solid #e0e0e0;
                                                    border-radius: 6px;
                                                    font-size: 16px;
                                                    box-sizing: border-box;
                                                ",
                                            }
                                        }
                                        
                                        if is_cardio {
                                            div {
                                                label {
                                                    style: "display: block; margin-bottom: 5px; font-weight: bold;",
                                                    "Distance (miles)"
                                                }
                                                input {
                                                    r#type: "number",
                                                    step: "0.1",
                                                    placeholder: "Distance",
                                                    value: "{distance_input}",
                                                    oninput: move |evt| distance_input.set(evt.value()),
                                                    style: "
                                                        width: 100%;
                                                        padding: 10px;
                                                        border: 1px solid #e0e0e0;
                                                        border-radius: 6px;
                                                        font-size: 16px;
                                                        box-sizing: border-box;
                                                    ",
                                                }
                                            }
                                        } else {
                                            div {
                                                label {
                                                    style: "display: block; margin-bottom: 5px; font-weight: bold;",
                                                    "Repetitions"
                                                }
                                                input {
                                                    r#type: "number",
                                                    placeholder: "Reps",
                                                    value: "{reps_input}",
                                                    oninput: move |evt| reps_input.set(evt.value()),
                                                    style: "
                                                        width: 100%;
                                                        padding: 10px;
                                                        border: 1px solid #e0e0e0;
                                                        border-radius: 6px;
                                                        font-size: 16px;
                                                        box-sizing: border-box;
                                                    ",
                                                }
                                            }
                                        }
                                        
                                        div {
                                            style: "display: flex; gap: 10px;",
                                            button {
                                                onclick: complete_exercise,
                                                style: "
                                                    flex: 1;
                                                    padding: 15px;
                                                    background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%);
                                                    color: white;
                                                    border: none;
                                                    border-radius: 8px;
                                                    font-size: 1.1em;
                                                    font-weight: bold;
                                                    cursor: pointer;
                                                ",
                                                "✓ Complete Exercise"
                                            }
                                            button {
                                                onclick: move |_| {
                                                    current_exercise_id.set(None);
                                                    weight_input.set(String::new());
                                                    reps_input.set(String::new());
                                                    distance_input.set(String::new());
                                                },
                                                style: "
                                                    padding: 15px 25px;
                                                    background: #e0e0e0;
                                                    color: #666;
                                                    border: none;
                                                    border-radius: 8px;
                                                    font-size: 1.1em;
                                                    font-weight: bold;
                                                    cursor: pointer;
                                                ",
                                                "Cancel"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Completed exercises list
                if !session.read().exercise_logs.is_empty() {
                    div {
                        style: "margin-top: 30px;",
                        h3 { "Completed Exercises" }
                        
                        for (idx, log) in session.read().exercise_logs.iter().enumerate() {
                            div {
                                key: "{idx}",
                                style: "
                                    padding: 15px;
                                    margin-bottom: 10px;
                                    border: 1px solid #e0e0e0;
                                    border-radius: 8px;
                                    background: white;
                                ",
                                
                                h4 { 
                                    style: "margin: 0 0 8px 0;",
                                    "{log.exercise_name}"
                                }
                                
                                div {
                                    style: "color: #666; font-size: 0.9em;",
                                    if let Some(weight) = log.weight {
                                        div { "Weight: {weight} lbs" }
                                    }
                                    if let Some(reps) = log.reps {
                                        div { "Reps: {reps}" }
                                    }
                                    if let Some(distance) = log.distance {
                                        div { "Distance: {distance} miles" }
                                    }
                                    if let Some(duration) = log.duration_seconds() {
                                        div { "Duration: {format_time(duration)}" }
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Link to add custom exercise
                div {
                    style: "margin-top: 30px; text-align: center;",
                    Link {
                        to: Route::AddCustomExercisePage {},
                        style: "
                            display: inline-block;
                            padding: 12px 24px;
                            background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
                            color: white;
                            text-decoration: none;
                            border-radius: 8px;
                            font-weight: bold;
                        ",
                        "+ Add Custom Exercise"
                    }
                }
            }
        }
    }
}
