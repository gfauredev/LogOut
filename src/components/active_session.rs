use crate::components::CompletedExerciseLog;
use crate::models::{
    get_current_timestamp, parse_distance_km, parse_weight_kg, Category, ExerciseLog,
    WorkoutSession,
};
use crate::services::{exercise_db, storage};
use crate::Route;
use dioxus::prelude::*;

use super::session_exercise_form::ExerciseFormPanel;
use super::session_timers::{RestTimerDisplay, SessionDurationDisplay};

/// Default rest duration in seconds
const DEFAULT_REST_DURATION: u64 = 30;
/// Snackbar auto-dismiss delay in milliseconds
const SNACKBAR_DISMISS_MS: u32 = 3_000;

// ── Helpers ────────────────────────────────────────────────────────────────

/// Prefill the weight / reps / distance inputs from the last recorded log for
/// `exercise_id`, or clear them if no prior log exists.
fn prefill_inputs_from_last_log(
    exercise_id: &str,
    mut weight_input: Signal<String>,
    mut reps_input: Signal<String>,
    mut distance_input: Signal<String>,
) {
    if let Some(last_log) = storage::get_last_exercise_log(exercise_id) {
        if let Some(w) = last_log.weight_hg {
            weight_input.set(format!("{:.1}", w.0 as f64 / 10.0));
        }
        if let Some(reps) = last_log.reps {
            reps_input.set(reps.to_string());
        }
        if let Some(d) = last_log.distance_m {
            distance_input.set(format!("{:.2}", d.0 as f64 / 1000.0));
        }
    } else {
        weight_input.set(String::new());
        reps_input.set(String::new());
        distance_input.set(String::new());
    }
}

#[component]
pub fn SessionView() -> Element {
    // use_sessions() must be called at the top level of the component, not inside
    // use_signal's initializer. Calling use_context (via use_sessions) inside another
    // use_hook's initializer causes a double-borrow of the hooks RefCell → panic.
    let sessions = storage::use_sessions();
    let mut session = use_signal(move || {
        sessions
            .read()
            .iter()
            .find(|s| s.is_active())
            .cloned()
            .unwrap_or_else(WorkoutSession::new)
    });

    let mut search_query = use_signal(String::new);
    let mut current_exercise_id = use_signal(move || {
        sessions
            .read()
            .iter()
            .find(|s| s.is_active())
            .and_then(|s| s.current_exercise_id.clone())
    });
    let mut current_exercise_start = use_signal(move || {
        sessions
            .read()
            .iter()
            .find(|s| s.is_active())
            .and_then(|s| s.current_exercise_start)
    });
    let mut weight_input = use_signal(String::new);
    let mut reps_input = use_signal(String::new);
    let mut distance_input = use_signal(String::new);

    // Rest duration setting (configurable by clicking the timer)
    let mut rest_duration = use_signal(|| DEFAULT_REST_DURATION);
    let mut show_rest_input = use_signal(|| false);
    let mut rest_input_value = use_signal(|| DEFAULT_REST_DURATION.to_string());

    // Rest timer state: tracks when the last exercise was completed
    let mut rest_start_time = use_signal(move || {
        sessions
            .read()
            .iter()
            .find(|s| s.is_active())
            .and_then(|s| s.rest_start_time)
    });

    // Congratulations toast (global context, survives session unmount)
    let mut congratulations = use_context::<crate::CongratulationsSignal>().0;

    // Bell rung tracker: how many times the rest bell has rung this rest period
    let mut rest_bell_count = use_signal(|| 0u64);
    // Duration bell tracker: whether the duration bell has been rung for this exercise
    let mut duration_bell_rung = use_signal(|| false);

    let custom_exercises = storage::use_custom_exercises();
    let all_exercises = exercise_db::use_exercises();

    // Reactive snapshot of pending exercise IDs – avoids multiple session.read() calls in the template
    let pending_ids = use_memo(move || session.read().pending_exercise_ids.clone());

    let search_results = use_memo(move || {
        let query = search_query.read();
        if query.is_empty() {
            vec![]
        } else {
            let mut results: Vec<(String, String, Category)> = Vec::new();
            let mut seen_ids = std::collections::HashSet::new();

            // Add custom exercises first (they have priority over DB exercises).
            // Use unified search_exercises so muscle/category/etc. are all searchable.
            let custom = custom_exercises.read();
            let custom_results = exercise_db::search_exercises(&custom, &query);
            for ex in custom_results {
                if seen_ids.insert(ex.id.clone()) {
                    results.push((ex.id.clone(), ex.name.clone(), ex.category));
                }
            }

            // Add DB exercises, skipping any IDs already added from custom exercises
            let all = all_exercises.read();
            let db_results = exercise_db::search_exercises(&all, &query);
            for ex in db_results.iter().take(10) {
                if seen_ids.insert(ex.id.clone()) {
                    results.push((ex.id.clone(), ex.name.clone(), ex.category));
                }
            }

            results
        }
    });

    let mut start_exercise =
        move |exercise_id: String, _exercise_name: String, _category: Category| {
            prefill_inputs_from_last_log(&exercise_id, weight_input, reps_input, distance_input);
            current_exercise_id.set(Some(exercise_id.clone()));
            let exercise_start = get_current_timestamp();
            current_exercise_start.set(Some(exercise_start));
            search_query.set(String::new());
            // Clear rest timer when starting a new exercise
            rest_start_time.set(None);
            rest_bell_count.set(0);
            duration_bell_rung.set(false);
            // Persist exercise start and cleared rest timer in session
            let mut current_session = session.read().clone();
            current_session.rest_start_time = None;
            current_session.current_exercise_id = Some(exercise_id.clone());
            current_session.current_exercise_start = Some(exercise_start);
            session.set(current_session.clone());
            storage::save_session(current_session);
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

        let weight_hg = parse_weight_kg(&weight_input.read());
        let reps = if force.is_some_and(|f| f.has_reps()) {
            reps_input.read().parse().ok()
        } else {
            None
        };
        let distance_m = if category == Category::Cardio {
            parse_distance_km(&distance_input.read())
        } else {
            None
        };

        let log = ExerciseLog {
            exercise_id: exercise_id.clone(),
            exercise_name,
            category,
            start_time,
            end_time: Some(end_time),
            weight_hg,
            reps,
            distance_m,
            force,
        };

        current_session.exercise_logs.push(log);
        // Save rest timer start time in the session for persistence across tab switches
        let rest_start = get_current_timestamp();
        current_session.rest_start_time = Some(rest_start);
        // Clear performing exercise from session
        current_session.current_exercise_id = None;
        current_session.current_exercise_start = None;
        session.set(current_session.clone());
        storage::save_session(current_session);

        current_exercise_id.set(None);
        current_exercise_start.set(None);
        weight_input.set(String::new());
        reps_input.set(String::new());
        distance_input.set(String::new());
        // Start rest timer
        rest_start_time.set(Some(rest_start));
        rest_bell_count.set(0);
        duration_bell_rung.set(false);
    };

    let cancel_exercise = move |_| {
        current_exercise_id.set(None);
        current_exercise_start.set(None);
        weight_input.set(String::new());
        reps_input.set(String::new());
        distance_input.set(String::new());
        // Persist cleared performing state
        let mut current_session = session.read().clone();
        current_session.current_exercise_id = None;
        current_session.current_exercise_start = None;
        session.set(current_session.clone());
        storage::save_session(current_session);
    };

    let finish_session = move |_| {
        let mut current_session = session.read().clone();
        if current_session.is_cancelled() {
            // No exercises logged: discard the session entirely
            storage::delete_session(&current_session.id);
            return;
        }
        current_session.end_time = Some(get_current_timestamp());
        storage::save_session(current_session.clone());
        // Show congratulatory toast (via global context so it survives unmount)
        congratulations.set(true);
        #[cfg(target_arch = "wasm32")]
        {
            spawn(async move {
                gloo_timers::future::TimeoutFuture::new(SNACKBAR_DISMISS_MS).await;
                congratulations.set(false);
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(SNACKBAR_DISMISS_MS as u64))
                    .await;
                congratulations.set(false);
            });
        }
    };

    let exercise_count = session.read().exercise_logs.len();

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
                        SessionDurationDisplay {
                            session_start_time: session.read().start_time,
                            session_is_active: session.read().is_active(),
                        }
                    }
                }
                div { class: "session-header__actions",
                    if exercise_count == 0 {
                        button {
                            onclick: finish_session,
                            class: "btn--cancel-session",
                            "Cancel Session"
                        }
                    } else {
                        button {
                            onclick: finish_session,
                            class: "btn--finish",
                            "Finish Session"
                        }
                    }
                }
            }

            // Rest duration input (shown when clicking timer)
            if *show_rest_input.read() {
                form {
                    class: "rest-duration-input",
                    aria_label: "Set rest duration",
                    onsubmit: move |evt| {
                        evt.prevent_default();
                        if let Ok(val) = rest_input_value.read().parse::<u64>() {
                            rest_duration.set(val);
                        }
                        show_rest_input.set(false);
                    },
                    label { r#for: "rest-duration-field", "Rest duration (seconds):" }
                    input {
                        id: "rest-duration-field",
                        r#type: "number",
                        value: "{rest_input_value}",
                        oninput: move |evt| rest_input_value.set(evt.value()),
                        class: "form-input form-input--rest",
                    }
                    button {
                        r#type: "submit",
                        class: "btn btn--accent",
                        "Set"
                    }
                }
            }

            // Rest timer (shown when no exercise is active and rest is ongoing)
            if current_exercise_id.read().is_none() {
                RestTimerDisplay {
                    rest_start_time,
                    rest_duration,
                    rest_bell_count,
                }
            }

            // Main content area
            section {
                class: "session-main",

                // Pending exercises (pre-added from a previous session)
                if current_exercise_id.read().is_none() && !pending_ids().is_empty() {
                    section { class: "pending-exercises",
                        h3 { "Pre-added Exercises" }
                        for exercise_id in pending_ids() {
                            {
                                let (name, category) = {
                                    let all = all_exercises.read();
                                    if let Some(ex) = exercise_db::get_exercise_by_id(&all, &exercise_id) {
                                        (ex.name.clone(), ex.category)
                                    } else {
                                        let custom = custom_exercises.read();
                                        if let Some(ex) = custom.iter().find(|e| e.id == exercise_id) {
                                            (ex.name.clone(), ex.category)
                                        } else {
                                            ("Unknown".to_string(), Category::Strength)
                                        }
                                    }
                                };
                                rsx! {
                                    article { class: "pending-exercise-item",
                                        span { class: "pending-exercise-item__name", "{name}" }
                                        span { class: "tag tag--category", "{category}" }
                                        button {
                                            class: "btn--start",
                                            onclick: {
                                                let id = exercise_id.clone();
                                                move |_| {
                                                    prefill_inputs_from_last_log(
                                                        &id,
                                                        weight_input,
                                                        reps_input,
                                                        distance_input,
                                                    );
                                                    // Remove from pending and save
                                                    let mut current_session = session.read().clone();
                                                    // Remove only the first occurrence so that repeated
                                                    // exercises are consumed one at a time.
                                                    let mut removed = false;
                                                    current_session.pending_exercise_ids.retain(|x| {
                                                        if !removed && x == &id {
                                                            removed = true;
                                                            false
                                                        } else {
                                                            true
                                                        }
                                                    });
                                                    let pending_start = get_current_timestamp();
                                                    current_session.rest_start_time = None;
                                                    current_session.current_exercise_id = Some(id.clone());
                                                    current_session.current_exercise_start = Some(pending_start);
                                                    session.set(current_session.clone());
                                                    storage::save_session(current_session);
                                                    // Start the exercise
                                                    current_exercise_id.set(Some(id.clone()));
                                                    current_exercise_start.set(Some(pending_start));
                                                    search_query.set(String::new());
                                                    rest_start_time.set(None);
                                                    rest_bell_count.set(0);
                                                    duration_bell_rung.set(false);
                                                }
                                            },
                                            "▶ Start"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Exercise search and selection
                if current_exercise_id.read().is_none() {
                    div {
                        class: "form-group",
                        h3 { "Select Exercise" }
                        div { class: "search-with-add",
                            input {
                                r#type: "text",
                                placeholder: "Search for an exercise...",
                                value: "{search_query}",
                                oninput: move |evt| search_query.set(evt.value()),
                                class: "search-input",
                            }
                            Link {
                                to: Route::AddCustomExercisePage {},
                                class: "add-exercise-btn",
                                title: "Add Custom Exercise",
                                "+"
                            }
                        }

                        if !search_results().is_empty() {
                            div {
                                class: "search-results search-results--tall",
                                for (id, name, category) in search_results() {
                                    div {
                                        key: "{id}",
                                        onclick: move |_| start_exercise(id.clone(), name.clone(), category),
                                        class: "search-result-item search-result-item--flex",
                                        span { "{name}" }
                                        span { class: "tag tag--category", "{category}" }
                                    }
                                }
                            }
                        }
                    }
                } else if let Some(exercise_id) = current_exercise_id.read().as_ref().cloned() {
                    // Current exercise input form
                    ExerciseFormPanel {
                        exercise_id,
                        weight_input,
                        reps_input,
                        distance_input,
                        current_exercise_start,
                        duration_bell_rung,
                        on_complete: complete_exercise,
                        on_cancel: cancel_exercise,
                    }
                }

                // Completed exercises list (antichronological order)
                if !session.read().exercise_logs.is_empty() {
                    section {
                        class: "completed-exercises-section",
                        h3 { "Completed Exercises" }

                        for (idx, log) in session.read().exercise_logs.iter().enumerate().rev() {
                            CompletedExerciseLog {
                                key: "{idx}",
                                idx,
                                log: log.clone(),
                                session,
                            }
                        }
                    }
                }
            }
        }
    }
}
