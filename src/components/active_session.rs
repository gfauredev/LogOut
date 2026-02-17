use dioxus::prelude::*;
use crate::models::{WorkoutSession, ExerciseLog, get_current_timestamp, format_time, format_weight, format_distance, parse_weight_kg, parse_distance_km};
use crate::services::{exercise_db, storage};
use crate::components::AnalyticsPanel;
use crate::Route;

#[component]
pub fn ActiveSessionPage() -> Element {
    let mut session = use_signal(|| {
        storage::get_active_session().unwrap_or_else(WorkoutSession::new)
    });
    
    let mut search_query = use_signal(|| String::new());
    let mut current_exercise_id = use_signal(|| None::<String>);
    let mut current_exercise_start = use_signal(|| None::<u64>);
    let mut weight_input = use_signal(|| String::new());
    let mut reps_input = use_signal(|| String::new());
    let mut distance_input = use_signal(|| String::new());
    let mut panel_open = use_signal(|| false);

    // Tick signal for live timer – updated every second by a coroutine
    let mut now_tick = use_signal(get_current_timestamp);

    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(1_000).await;
            now_tick.set(get_current_timestamp());
        }
    });

    // Calculate session duration (re-evaluated every tick)
    let session_duration = {
        let _tick = *now_tick.read(); // subscribe to tick
        let current_session = session.read();
        if current_session.is_active() {
            get_current_timestamp() - current_session.start_time
        } else {
            0
        }
    };

    let custom_exercises = storage::use_custom_exercises();
    
    let search_results = use_memo(move || {
        let query = search_query.read();
        if query.is_empty() {
            vec![]
        } else {
            let mut results = Vec::new();
            
            let db_exercises = exercise_db::search_exercises(&query);
            for ex in db_exercises.iter().take(10) {
                results.push((ex.id.clone(), ex.name.clone(), ex.category.clone()));
            }
            
            let custom = custom_exercises.read();
            for ex in custom.iter() {
                if ex.name.to_lowercase().contains(&query.to_lowercase()) {
                    results.push((ex.id.clone(), ex.name.clone(), ex.category.clone()));
                }
            }
            
            results
        }
    });
    
    let mut start_exercise = move |exercise_id: String, _exercise_name: String, _category: String| {
        if let Some(last_log) = storage::get_last_exercise_log(&exercise_id) {
            if let Some(w) = last_log.weight_dg {
                weight_input.set(format!("{:.1}", w as f64 / 100.0));
            }
            if let Some(reps) = last_log.reps {
                reps_input.set(reps.to_string());
            }
            if let Some(d) = last_log.distance_m {
                distance_input.set(format!("{:.2}", d as f64 / 1000.0));
            }
        } else {
            weight_input.set(String::new());
            reps_input.set(String::new());
            distance_input.set(String::new());
        }
        
        current_exercise_id.set(Some(exercise_id.clone()));
        current_exercise_start.set(Some(get_current_timestamp()));
        search_query.set(String::new());
    };
    
    let complete_exercise = move |_| {
        let exercise_id = match current_exercise_id.read().as_ref() {
            Some(id) => id.clone(),
            None => return,
        };
        
        let start_time = match current_exercise_start.read().as_ref() {
            Some(time) => *time,
            None => get_current_timestamp(),
        };
        
        let mut current_session = session.read().clone();
        
        let (exercise_name, category) = if let Some(ex) = exercise_db::get_exercise_by_id(&exercise_id) {
            (ex.name.clone(), ex.category.clone())
        } else {
            let custom = custom_exercises.read();
            if let Some(ex) = custom.iter().find(|e| e.id == exercise_id) {
                (ex.name.clone(), ex.category.clone())
            } else {
                return;
            }
        };
        
        let end_time = get_current_timestamp();
        let is_cardio = category.to_lowercase() == "cardio";
        
        let weight_dg = parse_weight_kg(&weight_input.read());
        let reps = if is_cardio { None } else { reps_input.read().parse().ok() };
        let distance_m = if is_cardio { parse_distance_km(&distance_input.read()) } else { None };
        
        let log = ExerciseLog {
            exercise_id: exercise_id.clone(),
            exercise_name,
            category,
            start_time,
            end_time: Some(end_time),
            weight_dg,
            reps,
            distance_m,
        };
        
        current_session.exercise_logs.push(log);
        storage::save_session(current_session.clone());
        session.set(current_session);
        
        current_exercise_id.set(None);
        current_exercise_start.set(None);
        weight_input.set(String::new());
        reps_input.set(String::new());
        distance_input.set(String::new());
    };
    
    let finish_session = move |_| {
        let mut current_session = session.read().clone();
        current_session.end_time = Some(get_current_timestamp());
        storage::save_session(current_session.clone());
        navigator().push(Route::HomePage {});
    };
    
    let panel_class = if *panel_open.read() { "slide-panel slide-panel--open" } else { "slide-panel slide-panel--closed" };

    rsx! {
        div {
            class: "session-container",
            
            // Sticky timer header
            div {
                class: "session-header",
                div {
                    h2 { class: "session-header__title", "⏱️ Active Session" }
                    p { class: "session-header__timer", "{format_time(session_duration)}" }
                }
                button {
                    onclick: finish_session,
                    class: "btn--finish",
                    "Finish Session"
                }
            }
            
            // Main content area with panel
            div {
                class: "session-body",
                
                div {
                    class: "session-main",
                
                // Exercise search and selection
                if current_exercise_id.read().is_none() {
                    div {
                        class: "form-group",
                        h3 { "Select Exercise" }
                        input {
                            r#type: "text",
                            placeholder: "Search for an exercise...",
                            value: "{search_query}",
                            oninput: move |evt| search_query.set(evt.value()),
                            class: "search-input",
                        }
                        
                        if !search_results().is_empty() {
                            div {
                                class: "search-results search-results--tall",
                                for (id, name, category) in search_results() {
                                    div {
                                        key: "{id}",
                                        onclick: move |_| start_exercise(id.clone(), name.clone(), category.clone()),
                                        class: "search-result-item search-result-item--flex",
                                        span { "{name}" }
                                        span { class: "tag tag--category", "{category}" }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Current exercise input form
                    if let Some(exercise_id) = current_exercise_id.read().as_ref() {
                        div {
                            class: "exercise-form",
                            
                            {
                                let (exercise_name, category) = if let Some(ex) = exercise_db::get_exercise_by_id(exercise_id) {
                                    (ex.name.clone(), ex.category.clone())
                                } else {
                                    let custom = custom_exercises.read();
                                    if let Some(ex) = custom.iter().find(|e| &e.id == exercise_id) {
                                        (ex.name.clone(), ex.category.clone())
                                    } else {
                                        ("Unknown".to_string(), "unknown".to_string())
                                    }
                                };
                                
                                let is_cardio = category.to_lowercase() == "cardio";
                                
                                rsx! {
                                    h3 { class: "exercise-form__title", "{exercise_name}" }
                                    
                                    div {
                                        class: "exercise-form__fields",
                                        
                                        div {
                                            label { class: "form-label", "Weight (kg)" }
                                            input {
                                                r#type: "number",
                                                step: "0.5",
                                                placeholder: "Optional",
                                                value: "{weight_input}",
                                                oninput: move |evt| weight_input.set(evt.value()),
                                                class: "form-input",
                                            }
                                        }
                                        
                                        if is_cardio {
                                            div {
                                                label { class: "form-label", "Distance (km)" }
                                                input {
                                                    r#type: "number",
                                                    step: "0.1",
                                                    placeholder: "Distance",
                                                    value: "{distance_input}",
                                                    oninput: move |evt| distance_input.set(evt.value()),
                                                    class: "form-input",
                                                }
                                            }
                                        } else {
                                            div {
                                                label { class: "form-label", "Repetitions" }
                                                input {
                                                    r#type: "number",
                                                    placeholder: "Reps",
                                                    value: "{reps_input}",
                                                    oninput: move |evt| reps_input.set(evt.value()),
                                                    class: "form-input",
                                                }
                                            }
                                        }
                                        
                                        div {
                                            class: "btn-row",
                                            button {
                                                onclick: complete_exercise,
                                                class: "btn--complete",
                                                "✓ Complete Exercise"
                                            }
                                            button {
                                                onclick: move |_| {
                                                    current_exercise_id.set(None);
                                                    current_exercise_start.set(None);
                                                    weight_input.set(String::new());
                                                    reps_input.set(String::new());
                                                    distance_input.set(String::new());
                                                },
                                                class: "btn--cancel",
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
                                class: "completed-log",
                                
                                h4 { class: "completed-log__title", "{log.exercise_name}" }
                                
                                div {
                                    class: "completed-log__details",
                                    if let Some(w) = log.weight_dg {
                                        div { "Weight: {format_weight(w)}" }
                                    }
                                    if let Some(reps) = log.reps {
                                        div { "Reps: {reps}" }
                                    }
                                    if let Some(d) = log.distance_m {
                                        div { "Distance: {format_distance(d)}" }
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
                    class: "center-text",
                    Link {
                        to: Route::AddCustomExercisePage {},
                        class: "add-custom-link",
                        "+ Add Custom Exercise"
                    }
                }
            }
        }
        
        // Analytics Panel
            div {
                class: "{panel_class}",
                AnalyticsPanel {}
            }
            
            // Bottom bar
            div {
                class: "bottom-bar",
                button {
                    onclick: move |_| {
                        let is_open = *panel_open.read();
                        panel_open.set(!is_open);
                    },
                    class: "bottom-bar__btn",
                    span { "📊 Analytics" }
                    span {
                        class: "bottom-bar__arrow",
                        if *panel_open.read() { "▼" } else { "▲" }
                    }
                }
            }
            
            // Overlay for mobile when panel is open
            if *panel_open.read() {
                div {
                    class: "overlay",
                    onclick: move |_| panel_open.set(false),
                }
            }
        }
    }
}
