use dioxus::prelude::*;
use crate::models::{WorkoutSession, ExerciseLog, get_current_timestamp, format_time, parse_weight_kg, parse_distance_km, Category};
use crate::services::{exercise_db, storage};
use crate::Route;

/// Default rest duration in seconds
const DEFAULT_REST_DURATION: u64 = 30;

#[component]
pub fn SessionView() -> Element {
    // use_sessions() must be called at the top level of the component, not inside
    // use_signal's initializer. Calling use_context (via use_sessions) inside another
    // use_hook's initializer causes a double-borrow of the hooks RefCell → panic.
    let sessions = storage::use_sessions();
    let mut session = use_signal(move || {
        sessions.read().iter().find(|s| s.is_active()).cloned()
            .unwrap_or_else(WorkoutSession::new)
    });
    
    let mut search_query = use_signal(|| String::new());
    let mut current_exercise_id = use_signal(|| None::<String>);
    let mut current_exercise_start = use_signal(|| None::<u64>);
    let mut weight_input = use_signal(|| String::new());
    let mut reps_input = use_signal(|| String::new());
    let mut distance_input = use_signal(|| String::new());

    // Rest duration setting (configurable by clicking the timer)
    let mut rest_duration = use_signal(|| DEFAULT_REST_DURATION);
    let mut show_rest_input = use_signal(|| false);
    let mut rest_input_value = use_signal(|| DEFAULT_REST_DURATION.to_string());

    // Rest timer state: tracks when the last exercise was completed
    let mut rest_start_time = use_signal(|| None::<u64>);

    // Snackbar state for congratulatory message
    let mut show_snackbar = use_signal(|| false);

    // Bell rung tracker: how many times the rest bell has rung this rest period
    let mut rest_bell_count = use_signal(|| 0u64);
    // Duration bell tracker: whether the duration bell has been rung for this exercise
    let mut duration_bell_rung = use_signal(|| false);

    // Tick signal for live timer – updated every second by a coroutine
    let mut now_tick = use_signal(|| get_current_timestamp());

    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        loop {
            #[cfg(target_arch = "wasm32")]
            gloo_timers::future::TimeoutFuture::new(1_000).await;
            // On non-wasm targets (e.g. native test builds) this coroutine is
            // never executed – components only run in the Dioxus web runtime.
            // The pending() call is only here so the code compiles on all targets.
            #[cfg(not(target_arch = "wasm32"))]
            std::future::pending::<()>().await;
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

    // Calculate rest elapsed time and handle rest bell
    let rest_elapsed = {
        let _tick = *now_tick.read();
        if let Some(start) = *rest_start_time.read() {
            let elapsed = get_current_timestamp() - start;
            let rd = *rest_duration.read();
            if rd > 0 && elapsed > 0 {
                let intervals = elapsed / rd;
                let prev_count = *rest_bell_count.read();
                if intervals > prev_count {
                    rest_bell_count.set(intervals);
                    #[cfg(target_arch = "wasm32")]
                    ring_bell(false);
                }
            }
            Some(elapsed)
        } else {
            None
        }
    };

    let rest_exceeded = rest_elapsed
        .map(|e| e >= *rest_duration.read())
        .unwrap_or(false);

    // Handle duration bell for exercises without reps or distance
    {
        let _tick = *now_tick.read();
        if let (Some(exercise_id), Some(start)) = (
            current_exercise_id.read().as_ref(),
            *current_exercise_start.read(),
        ) {
            if !*duration_bell_rung.read() {
                if let Some(last_log) = storage::get_last_exercise_log(exercise_id) {
                    let has_reps = last_log.reps.is_some();
                    let has_distance = last_log.distance_dam.is_some();
                    if !has_reps && !has_distance {
                        if let Some(last_dur) = last_log.duration_seconds() {
                            let current_dur = get_current_timestamp().saturating_sub(start);
                            if current_dur >= last_dur && last_dur > 0 {
                                duration_bell_rung.set(true);
                                #[cfg(target_arch = "wasm32")]
                                ring_bell(true);
                            }
                        }
                    }
                }
            }
        }
    }

    let custom_exercises = storage::use_custom_exercises();
    let all_exercises = exercise_db::use_exercises();
    
    let search_results = use_memo(move || {
        let query = search_query.read();
        if query.is_empty() {
            vec![]
        } else {
            let mut results: Vec<(String, String, Category)> = Vec::new();
            
            let all = all_exercises.read();
            let db_results = exercise_db::search_exercises(&all, &query);
            for ex in db_results.iter().take(10) {
                results.push((ex.id.clone(), ex.name.clone(), ex.category));
            }
            
            let custom = custom_exercises.read();
            for ex in custom.iter() {
                if ex.name.to_lowercase().contains(&query.to_lowercase()) {
                    results.push((ex.id.clone(), ex.name.clone(), ex.category));
                }
            }
            
            results
        }
    });
    
    let mut start_exercise = move |exercise_id: String, _exercise_name: String, _category: Category| {
        if let Some(last_log) = storage::get_last_exercise_log(&exercise_id) {
            if let Some(w) = last_log.weight_dg {
                weight_input.set(format!("{:.1}", w.0 as f64 / 100.0));
            }
            if let Some(reps) = last_log.reps {
                reps_input.set(reps.to_string());
            }
            if let Some(d) = last_log.distance_dam {
                distance_input.set(format!("{:.2}", d.0 as f64 / 100.0));
            }
        } else {
            weight_input.set(String::new());
            reps_input.set(String::new());
            distance_input.set(String::new());
        }
        
        current_exercise_id.set(Some(exercise_id.clone()));
        current_exercise_start.set(Some(get_current_timestamp()));
        search_query.set(String::new());
        // Clear rest timer when starting a new exercise
        rest_start_time.set(None);
        rest_bell_count.set(0);
        duration_bell_rung.set(false);
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
        
        let (exercise_name, category, force) = {
            let all = all_exercises.read();
            if let Some(ex) = exercise_db::get_exercise_by_id(&all, &exercise_id) {
                (ex.name.clone(), ex.category, ex.force)
            } else {
                let custom = custom_exercises.read();
                if let Some(ex) = custom.iter().find(|e| e.id == exercise_id) {
                    (ex.name.clone(), ex.category, ex.force)
                } else {
                    return;
                }
            }
        };
        
        let end_time = get_current_timestamp();
        
        let weight_dg = parse_weight_kg(&weight_input.read());
        let reps = if force.map_or(false, |f| f.has_reps()) { reps_input.read().parse().ok() } else { None };
        let distance_dam = if category == Category::Cardio { parse_distance_km(&distance_input.read()) } else { None };
        
        let log = ExerciseLog {
            exercise_id: exercise_id.clone(),
            exercise_name,
            category,
            start_time,
            end_time: Some(end_time),
            weight_dg,
            reps,
            distance_dam,
            force,
        };
        
        current_session.exercise_logs.push(log);
        storage::save_session(current_session.clone());
        session.set(current_session);
        
        current_exercise_id.set(None);
        current_exercise_start.set(None);
        weight_input.set(String::new());
        reps_input.set(String::new());
        distance_input.set(String::new());
        // Start rest timer
        rest_start_time.set(Some(get_current_timestamp()));
        rest_bell_count.set(0);
        duration_bell_rung.set(false);
    };
    
    let finish_session = move |_| {
        let mut current_session = session.read().clone();
        current_session.end_time = Some(get_current_timestamp());
        storage::save_session(current_session.clone());
        // Show congratulatory snackbar if exercises were completed
        if !current_session.exercise_logs.is_empty() {
            show_snackbar.set(true);
            #[cfg(target_arch = "wasm32")]
            {
                spawn(async move {
                    gloo_timers::future::TimeoutFuture::new(3_000).await;
                    show_snackbar.set(false);
                });
            }
        }
    };

    let exercise_count = session.read().exercise_logs.len();
    let finish_label = if exercise_count == 0 { "Cancel Session" } else { "Finish Session" };
    

    rsx! {
        section {
            class: "session-container",
            
            // Sticky timer header
            header {
                class: "session-header",
                div {
                    h2 { class: "session-header__title", "⏱️ Active Session" }
                    p {
                        class: "session-header__timer",
                        onclick: move |_| {
                            rest_input_value.set(rest_duration.read().to_string());
                            let current = *show_rest_input.read();
                            show_rest_input.set(!current);
                        },
                        title: "Click to set rest duration",
                        "{format_time(session_duration)}"
                    }
                }
                button {
                    onclick: finish_session,
                    class: if exercise_count == 0 { "btn--cancel-session" } else { "btn--finish" },
                    "{finish_label}"
                }
            }

            // Rest duration input (shown when clicking timer)
            if *show_rest_input.read() {
                div {
                    class: "rest-duration-input",
                    label { "Rest duration (seconds):" }
                    input {
                        r#type: "number",
                        value: "{rest_input_value}",
                        oninput: move |evt| rest_input_value.set(evt.value()),
                        class: "form-input",
                        style: "width: 80px; text-align: center;",
                    }
                    button {
                        onclick: move |_| {
                            if let Ok(val) = rest_input_value.read().parse::<u64>() {
                                rest_duration.set(val);
                            }
                            show_rest_input.set(false);
                        },
                        class: "btn btn--accent",
                        "Set"
                    }
                }
            }

            // Rest timer (shown when no exercise is active and rest is ongoing)
            if current_exercise_id.read().is_none() {
                if let Some(elapsed) = rest_elapsed {
                    div {
                        class: if rest_exceeded { "rest-timer rest-timer--exceeded" } else { "rest-timer" },
                        "🛋️ Rest: {format_time(elapsed)}"
                    }
                }
            }
            
            // Main content area
            div {
                class: "session-body",
                
                main {
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
                        article {
                            class: "exercise-form",
                            
                            {
                                let (exercise_name, category, force) = {
                                    let all = all_exercises.read();
                                    if let Some(ex) = exercise_db::get_exercise_by_id(&all, exercise_id) {
                                        (ex.name.clone(), ex.category, ex.force)
                                    } else {
                                        let custom = custom_exercises.read();
                                        if let Some(ex) = custom.iter().find(|e| &e.id == exercise_id) {
                                            (ex.name.clone(), ex.category, ex.force)
                                        } else {
                                            ("Unknown".to_string(), Category::Strength, None)
                                        }
                                    }
                                };
                                
                                let show_reps = force.map_or(false, |f| f.has_reps());
                                let is_cardio = category == Category::Cardio;
                                let last_duration = storage::get_last_exercise_log(exercise_id)
                                    .and_then(|log| log.duration_seconds());
                                
                                rsx! {
                                    if let Some(dur) = last_duration {
                                        div {
                                            class: "exercise-form__last-duration",
                                            "Last: {format_time(dur)}"
                                        }
                                    }
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
                                        }
                                        
                                        if show_reps {
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
                    section {
                        style: "margin-top: 30px;",
                        h3 { "Completed Exercises" }
                        
                        for (idx, log) in session.read().exercise_logs.iter().enumerate() {
                            article {
                                key: "{idx}",
                                class: "completed-log",
                                
                                h4 { class: "completed-log__title", "{log.exercise_name}" }
                                
                                div {
                                    class: "completed-log__details",
                                    if let Some(w) = log.weight_dg {
                                        div { "Weight: {w}" }
                                    }
                                    if let Some(reps) = log.reps {
                                        div { "Reps: {reps}" }
                                    }
                                    if let Some(d) = log.distance_dam {
                                        div { "Distance: {d}" }
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

            // Congratulatory snackbar
            if *show_snackbar.read() {
                div {
                    class: "snackbar",
                    "🎉 Great workout! Session complete!"
                }
            }
        }
    }
}

/// Ring a bell sound using the Web Audio API.
/// `is_duration_bell` uses a different tone to distinguish from rest bell.
#[cfg(target_arch = "wasm32")]
fn ring_bell(is_duration_bell: bool) {
    let freq = if is_duration_bell { "880" } else { "440" };
    let duration = if is_duration_bell { "0.3" } else { "0.2" };
    let js_code = format!(
        "try{{var c=new(window.AudioContext||window.webkitAudioContext)();var o=c.createOscillator();o.type='sine';o.frequency.value={};o.connect(c.destination);o.start();o.stop(c.currentTime+{});}}catch(e){{}}",
        freq, duration
    );
    let _ = js_sys::eval(&js_code);
}
