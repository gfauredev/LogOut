use crate::components::CompletedExerciseLog;
use crate::models::{
    get_current_timestamp, parse_distance_km, parse_weight_kg, Category, ExerciseLog, Force,
    WorkoutSession,
};
use crate::services::{exercise_db, storage};
use crate::Route;
use dioxus::prelude::*;

use super::session_exercise_form::ExerciseFormPanel;
use super::session_timers::{RestTimerDisplay, SessionDurationDisplay};

/// Default rest duration in seconds
const DEFAULT_REST_DURATION: u64 = 30;

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
            weight_input.set(format!("{:.1}", f64::from(w.0) / 10.0));
        }
        if let Some(reps) = last_log.reps {
            reps_input.set(reps.to_string());
        }
        if let Some(d) = last_log.distance_m {
            distance_input.set(format!("{:.2}", f64::from(d.0) / 1000.0));
        }
    } else {
        weight_input.set(String::new());
        reps_input.set(String::new());
        distance_input.set(String::new());
    }
}

/// Sticky session header showing the elapsed timer and the cancel/finish button.
#[component]
fn SessionHeader(
    session_start_time: u64,
    session_is_active: bool,
    exercise_count: usize,
    on_click_timer: EventHandler<()>,
    on_finish: EventHandler<()>,
) -> Element {
    rsx! {
        header { class: "session",
            h2 { tabindex: 0, "⏱️ Active Session" }
            time {
                onclick: move |_| on_click_timer.call(()),
                title: "Click to set rest duration",
                SessionDurationDisplay {
                    session_start_time,
                    session_is_active,
                }
            }
            if exercise_count == 0 {
                button { class: "no",
                    onclick: move |_| on_finish.call(()),
                    title: "Cancel Session", "❌"
                }
            } else {
                button { class: "yes",
                    onclick: move |_| on_finish.call(()),
                    title: "Finish Session", "✅"
                }
            }
        }
    }
}

/// Collapsible form for configuring the rest duration between sets.
#[component]
fn RestDurationInput(
    mut show_rest_input: Signal<bool>,
    mut rest_input_value: Signal<String>,
    mut rest_duration: Signal<u64>,
) -> Element {
    rsx! {
        form { class: "inputs",
            aria_label: "Set rest duration",
            onsubmit: move |evt| {
                evt.prevent_default();
                if let Ok(val) = rest_input_value.read().parse::<u64>() {
                    rest_duration.set(val);
                }
                show_rest_input.set(false);
            },
            label { r#for: "rest-duration-field", "Rest duration" }
            input {
                id: "rest-duration-field",
                r#type: "number",
                inputmode: "numeric",
                value: "{rest_input_value}",
                oninput: move |evt| rest_input_value.set(evt.value()),
            }
            button { class: "yes", r#type: "submit", "✅" }
        }
    }
}

/// List of exercises pre-added to the session that haven't been started yet.
/// Fires `on_start` with the exercise ID when the user taps ▶ Start.
#[component]
fn PendingExercisesSection(pending_ids: Vec<String>, on_start: EventHandler<String>) -> Element {
    let all_exercises = exercise_db::use_exercises();
    let custom_exercises = storage::use_custom_exercises();
    rsx! {
        section { class: "exercises",
            // h3 { "Pre-added Exercises" }
            for exercise_id in pending_ids {
                {
                    let (name, category) = {
                        let all = all_exercises.read();
                        let custom = custom_exercises.read();
                        if let Some(ex) = exercise_db::resolve_exercise(&all, &custom, &exercise_id) {
                            (ex.name.clone(), ex.category)
                        } else {
                            ("Unknown".to_string(), Category::Strength)
                        }
                    };
                    rsx! {
                        article { header {
                            h4 { "{name}" }
                            ul { li { "{category}" } }
                            button { class: "edit",
                                onclick: {
                                    let id = exercise_id.clone();
                                    move |_| on_start.call(id.clone())
                                },
                                "🔄"
                            }
                        }}
                    }
                }
            }
        }
    }
}

/// Antichronological list of completed exercise logs with replay and edit actions.
/// Fires `on_replay` with the exercise ID when the user taps 🔁.
///
/// When no exercise is active and the last completed exercise was also done
/// earlier in the session, a quick-action button is shown at the top suggesting
/// the exercise that followed that earlier set.
#[component]
fn CompletedExercisesSection(
    session: Memo<WorkoutSession>,
    no_exercise_active: bool,
    on_replay: EventHandler<String>,
) -> Element {
    let all_exercises = exercise_db::use_exercises();
    let custom_exercises = storage::use_custom_exercises();
    // Determine whether we can suggest a "next" exercise.
    // Rule: find the last log entry, then search the history backwards for a
    // previous occurrence of the same exercise; if found and a log entry exists
    // immediately after it, that entry's exercise is the suggestion.
    let suggested_next = use_memo(move || {
        let logs = &session.read().exercise_logs;
        let last = logs.last()?;
        let last_id = &last.exercise_id;

        // Index of the last log entry
        let last_idx = logs.len() - 1;

        // Search backwards (excluding the final entry) for a prior occurrence
        let prior_idx = logs[..last_idx]
            .iter()
            .rposition(|l| l.exercise_id == *last_id)?;

        // The exercise that followed the prior occurrence
        let next_log = logs.get(prior_idx + 1)?;
        // If the next entry is the same as the last, there is nothing new to suggest
        if next_log.exercise_id == *last_id {
            return None;
        }
        Some((next_log.exercise_id.clone(), next_log.exercise_name.clone()))
    });

    // Resolve a human-readable name for the suggestion (prefer DB/custom name
    // over the stored snapshot in case the exercise was renamed).
    let suggestion_label = use_memo(move || {
        suggested_next().map(|(id, fallback_name)| {
            let all = all_exercises.read();
            let custom = custom_exercises.read();
            let name = exercise_db::resolve_exercise(&all, &custom, &id)
                .map_or(fallback_name, |ex| ex.name.clone());
            (id, name)
        })
    });

    rsx! {
        section { class: "exercises",
            h3 { "Completed Exercises" }
            // Quick-repeat suggestion: shown when no exercise is active and a
            // prior sequence implies what the next exercise should be.
            if no_exercise_active {
                if let Some((next_id, next_name)) = suggestion_label() {
                    button { class: "label",
                        onclick: {
                            let id = next_id.clone();
                            move |_| on_replay.call(id.clone())
                        },
                        "⏩ {next_name}"
                    }
                }
            }
            {
                rsx! {
                    for (idx, log) in session.read().exercise_logs.iter().enumerate().rev() {
                        CompletedExerciseLog {
                            key: "{idx}",
                            idx,
                            log: log.clone(),
                            session,
                            show_replay: no_exercise_active,
                            on_replay: {
                                let id = log.exercise_id.clone();
                                #[allow(clippy::ignored_unit_patterns)]
                                move |_| on_replay.call(id.clone())
                            },
                        }
                    }
                }
            }
        }
    }
}

#[component]
#[allow(clippy::ignored_unit_patterns)]
pub fn SessionView() -> Element {
    let sessions = storage::use_sessions();
    let session = use_memo(move || {
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

    // Rest duration setting (configurable by clicking the timer header)
    let rest_duration = use_signal(|| DEFAULT_REST_DURATION);
    // show_rest_input is driven by the global ShowRestInputSignal so that
    // the header (rendered in the layout) can toggle it from any page.
    let show_rest_input = use_context::<crate::ShowRestInputSignal>().0;
    let mut rest_input_value = use_signal(|| DEFAULT_REST_DURATION.to_string());

    // Pre-fill the rest duration input with the current value each time the
    // rest-input panel is opened (so the field always reflects the latest setting).
    use_effect(move || {
        if *show_rest_input.read() {
            rest_input_value.set(rest_duration.read().to_string());
        }
    });

    // Rest timer state: tracks when the last exercise was completed
    let mut rest_start_time = use_signal(move || {
        sessions
            .read()
            .iter()
            .find(|s| s.is_active())
            .and_then(|s| s.rest_start_time)
    });

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
            for ex in db_results.into_iter().take(10) {
                if seen_ids.insert(ex.id.clone()) {
                    results.push((ex.id.clone(), ex.name.clone(), ex.category));
                }
            }

            results
        }
    });

    let mut start_exercise = move |exercise_id: String| {
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
        storage::save_session(current_session);
    };

    #[allow(clippy::ignored_unit_patterns)]
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
            let custom = custom_exercises.read();
            if let Some(ex) = exercise_db::resolve_exercise(&all, &custom, &exercise_id) {
                (ex.name.clone(), ex.category, ex.force)
            } else {
                return;
            }
        };

        let end_time = get_current_timestamp();

        let weight_hg = parse_weight_kg(&weight_input.read());
        let reps = if force.is_some_and(Force::has_reps) {
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

    #[allow(clippy::ignored_unit_patterns)]
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
        storage::save_session(current_session);
    };

    rsx! {
        // NOTE: SessionHeader is rendered by GlobalSessionHeader in the layout.
        if *show_rest_input.read() {
            RestDurationInput {
                show_rest_input,
                rest_input_value,
                rest_duration,
            }
        }
        if current_exercise_id.read().is_none() {
            RestTimerDisplay {
                start_time: rest_start_time,
                duration: rest_duration,
                bell_count: rest_bell_count,
            }
        }
        main { class: "session",
            if current_exercise_id.read().is_none() && !pending_ids().is_empty() {
                PendingExercisesSection {
                    pending_ids: pending_ids(),
                    on_start: move |exercise_id: String| {
                        prefill_inputs_from_last_log(
                            &exercise_id,
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
                            if !removed && x == &exercise_id {
                                removed = true;
                                false
                            } else {
                                true
                            }
                        });
                        let pending_start = get_current_timestamp();
                        current_session.rest_start_time = None;
                        current_session.current_exercise_id = Some(exercise_id.clone());
                        current_session.current_exercise_start = Some(pending_start);
                        storage::save_session(current_session);
                        // Start the exercise
                        current_exercise_id.set(Some(exercise_id.clone()));
                        current_exercise_start.set(Some(pending_start));
                        search_query.set(String::new());
                        rest_start_time.set(None);
                        rest_bell_count.set(0);
                        duration_bell_rung.set(false);
                    },
                }
            }
            if current_exercise_id.read().is_none() {
                div { class: "inputs",
                    input { r#type: "text",
                        placeholder: "Search for an exercise...",
                        value: "{search_query}",
                        oninput: move |evt| search_query.set(evt.value()),
                    }
                    Link { class: "add",
                        to: Route::AddExercise {},
                        title: "Add Custom Exercise",
                        "+"
                    }
                }
                if !search_results().is_empty() {
                    ul { class: "results",
                        for (id, name, category) in search_results() {
                            li {
                                key: "{id}",
                                onclick: move |_| start_exercise(id.clone()),
                                span { "{name}" }
                                span { class: "category", "{category}" }
                            }
                        }
                    }
                }
            } else if let Some(exercise_id) = current_exercise_id.read().as_ref().cloned() {
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
            if !session.read().exercise_logs.is_empty() {
                CompletedExercisesSection {
                    session,
                    no_exercise_active: current_exercise_id.read().is_none(),
                    on_replay: move |exercise_id: String| start_exercise(exercise_id),
                }
            }
        }
    }
}

/// Sticky session header rendered in the app-level layout so it remains
/// visible on every page while a workout session is active.
///
/// Clicking the timer toggles the rest-duration input form in [`SessionView`]
/// via the global [`crate::ShowRestInputSignal`].
/// The finish/cancel button ends or discards the session from any page.
#[component]
#[allow(clippy::ignored_unit_patterns)]
pub fn GlobalSessionHeader() -> Element {
    let sessions = storage::use_sessions();
    let session = use_memo(move || sessions.read().iter().find(|s| s.is_active()).cloned());

    let mut show_rest = use_context::<crate::ShowRestInputSignal>().0;
    let mut congratulations = use_context::<crate::CongratulationsSignal>().0;

    let Some(sess) = session() else {
        return rsx! {};
    };

    let exercise_count = sess.exercise_logs.len();
    let session_start_time = sess.start_time;
    let session_is_active = sess.is_active();

    #[allow(clippy::ignored_unit_patterns)]
    let on_finish = move |_| {
        let Some(s) = session() else { return };
        if s.is_cancelled() {
            storage::delete_session(&s.id);
        } else {
            let mut s = s.clone();
            s.end_time = Some(get_current_timestamp());
            storage::save_session(s);
            congratulations.set(true);
        }
    };

    rsx! {
        SessionHeader {
            session_start_time,
            session_is_active,
            exercise_count,
            on_click_timer: #[allow(clippy::ignored_unit_patterns)]
            move |_| {
                let current = *show_rest.peek();
                show_rest.set(!current);
            },
            on_finish,
        }
    }
}
