use super::session_exercise_form::ExerciseFormPanel;
use super::session_timers::{RestTimerDisplay, SessionDurationDisplay};
use crate::components::CompletedExerciseLog;
use crate::models::{
    get_current_timestamp, parse_distance_km, parse_weight_kg, Category, ExerciseLog, Force,
    WorkoutSession,
};
use crate::services::exercise_db::{
    detect_filter_suggestions, exercise_matches_filters, SearchFilter,
};
use crate::services::{exercise_db, storage};
use crate::{RestDurationSignal, Route};
use dioxus::prelude::*;
use futures_channel::mpsc::UnboundedReceiver;
use std::sync::Arc;
/// Maximum number of simultaneously active hard filters in the session search.
const MAX_FILTERS: usize = 4;
/// Debounce delay in milliseconds before re-running the expensive exercise filter.
const SEARCH_DEBOUNCE_MS: u32 = 200;
/// Maximum exercises shown when only attribute filters are active and there is no text query.
const MAX_FILTER_ONLY_RESULTS: usize = 20;
/// Maximum exercises shown from the full database when a text search query is active.
const MAX_TEXT_SEARCH_RESULTS: usize = 10;
/// Prefill the weight / reps / distance inputs from the last recorded log for
/// `exercise_id`, or clear them if no prior log exists.
///
/// Checks both the active session (in-memory signal) and completed sessions
/// (via the `BestsCache` `last_*` fields) and uses the most-recently completed
/// log regardless of which session it belongs to.
fn prefill_inputs_from_last_log(
    exercise_id: &str,
    mut weight_input: Signal<String>,
    mut reps_input: Signal<String>,
    mut distance_input: Signal<String>,
) {
    // Most-recent log from the active session (same session).
    let active_log = storage::get_last_exercise_log(exercise_id);
    // Most-recent log info from completed sessions (via cache).
    let bests = storage::get_exercise_bests(exercise_id);

    // Pick whichever source has the more recent end_time.
    let use_active = match (
        active_log.as_ref().and_then(|l| l.end_time),
        bests.last_log_end_time,
    ) {
        (Some(a), Some(b)) => a >= b,
        (Some(_), None) => true,
        (None, Some(_)) => false,
        (None, None) => true, // both empty; fall through to clearing
    };

    if use_active {
        if let Some(last_log) = active_log {
            if let Some(w) = last_log.weight_hg {
                weight_input.set(format!("{:.1}", f64::from(w.0) / 10.0));
            } else {
                weight_input.set(String::new());
            }
            if let Some(reps) = last_log.reps {
                reps_input.set(reps.to_string());
            } else {
                reps_input.set(String::new());
            }
            if let Some(d) = last_log.distance_m {
                distance_input.set(format!("{:.2}", f64::from(d.0) / 1000.0));
            } else {
                distance_input.set(String::new());
            }
        } else if bests.last_log_end_time.is_none() {
            // No log anywhere – clear inputs.
            weight_input.set(String::new());
            reps_input.set(String::new());
            distance_input.set(String::new());
        }
    } else {
        // Use values from the most-recently completed cross-session log.
        if let Some(w) = bests.last_weight_hg {
            weight_input.set(format!("{:.1}", f64::from(w.0) / 10.0));
        } else {
            weight_input.set(String::new());
        }
        if let Some(reps) = bests.last_reps {
            reps_input.set(reps.to_string());
        } else {
            reps_input.set(String::new());
        }
        if let Some(d) = bests.last_distance_m {
            distance_input.set(format!("{:.2}", f64::from(d.0) / 1000.0));
        } else {
            distance_input.set(String::new());
        }
    }
}
/// Sticky session header showing the elapsed timer, rest timer, and session controls.
#[component]
fn SessionHeader(
    session_start_time: u64,
    session_is_active: bool,
    paused_at: Option<u64>,
    /// Total cumulative seconds spent paused (not counting the current pause).
    total_paused_duration: u64,
    exercise_count: usize,
    /// Timestamp when the current rest period began, or `None` when not resting.
    rest_start_time: Option<u64>,
    /// Configured rest duration (seconds).
    rest_duration: u64,
    on_click_timer: EventHandler<()>,
    on_pause: EventHandler<()>,
    on_finish: EventHandler<()>,
) -> Element {
    let is_paused = paused_at.is_some();
    rsx! {
        header { class: "session",
            h2 { tabindex: 0, "⏱️ Active Session" }
            div {
                class: "session-timers",
                onclick: move |_| on_click_timer.call(()),
                title: "Click to set rest duration",
                time {
                    SessionDurationDisplay {
                        session_start_time,
                        session_is_active,
                        paused_at,
                        total_paused_duration,
                    }
                }
                RestTimerDisplay {
                    start_time: rest_start_time,
                    rest_duration,
                    paused_at,
                }
            }
            button {
                class: "edit",
                onclick: move |_| on_pause.call(()),
                title: if is_paused { "Resume Session" } else { "Pause Session" },
                if is_paused {
                    "▶️"
                } else {
                    "⏸️"
                }
            }
            if exercise_count == 0 {
                button {
                    class: "back",
                    onclick: move |_| on_finish.call(()),
                    title: "Cancel Session",
                    "❌"
                }
            } else {
                button {
                    class: "save",
                    onclick: move |_| on_finish.call(()),
                    title: "Finish Session",
                    "💾"
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
        form {
            class: "inputs",
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
            button { class: "yes", r#type: "submit", "💾" }
        }
    }
}
/// List of exercises pre-added to the session that haven't been started yet.
/// The first (oldest) exercise is always visible and directly clickable.
/// Any additional exercises are hidden inside a folded `<details>` dropdown.
/// Fires `on_start` with the exercise ID when the user taps 🔁.
#[component]
fn PendingExercisesSection(pending_ids: Vec<String>, on_start: EventHandler<String>) -> Element {
    let all_exercises = exercise_db::use_exercises();
    let custom_exercises = storage::use_custom_exercises();
    let resolved: Vec<(String, String, Category)> = {
        let all = all_exercises.read();
        let custom = custom_exercises.read();
        pending_ids
            .iter()
            .map(|id| {
                if let Some(ex) = exercise_db::resolve_exercise(&all, &custom, id) {
                    (id.clone(), ex.name.clone(), ex.category)
                } else {
                    (id.clone(), "Unknown".to_string(), Category::Strength)
                }
            })
            .collect()
    };
    rsx! {
        section { class: "exercises",
            if let Some((first_id, first_name, first_cat)) = resolved.first() {
                {
                    let id = first_id.clone();
                    let name = first_name.clone();
                    let cat = *first_cat;
                    rsx! {
                        article {
                            header {
                                h4 { "{name}" }
                                ul {
                                    li { "{cat}" }
                                }
                                button { class: "edit", onclick: move |_| on_start.call(id.clone()), "🔁" }
                            }
                        }
                    }
                }
            }
            if resolved.len() > 1 {
                details {
                    summary { "More pre-added ({resolved.len() - 1})" }
                    for (id , name , category) in resolved.iter().skip(1).cloned() {
                        {
                            let id2 = id.clone();
                            rsx! {
                                article { key: "{id}",
                                    header {
                                        h4 { "{name}" }
                                        ul {
                                            li { "{category}" }
                                        }
                                        button { class: "edit", onclick: move |_| on_start.call(id2.clone()), "🔁" }
                                    }
                                }
                            }
                        }
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
    let suggested_next = use_memo(move || {
        let logs = &session.read().exercise_logs;
        let last = logs.last()?;
        let last_id = &last.exercise_id;
        let last_idx = logs.len() - 1;
        let prior_idx = logs[..last_idx]
            .iter()
            .rposition(|l| l.exercise_id == *last_id)?;
        let next_log = logs.get(prior_idx + 1)?;
        if next_log.exercise_id == *last_id {
            return None;
        }
        Some((next_log.exercise_id.clone(), next_log.exercise_name.clone()))
    });
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
        section { // class: "exercises",


            h3 { "Completed Exercises" }
            if no_exercise_active {
                if let Some((next_id, next_name)) = suggestion_label() {
                    button {
                        class: "label",
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
                    for (idx , log) in session.read().exercise_logs.iter().enumerate().rev() {
                        CompletedExerciseLog {
                            key: "{idx}",
                            idx,
                            log: log.clone(),
                            session,
                            show_replay: no_exercise_active,
                            on_replay: {
                                let id = log.exercise_id.clone();
                                move |()| on_replay.call(id.clone())
                            },
                        }
                    }
                }
            }
        }
    }
}
#[component]
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
    let mut debounced_query = use_signal(String::new);
    let mut active_filters: Signal<Vec<SearchFilter>> = use_signal(Vec::new);
    let current_exercise_id = use_memo(move || session.read().current_exercise_id.clone());
    let current_exercise_start = use_memo(move || session.read().current_exercise_start);
    let mut weight_input = use_signal(String::new);
    let mut reps_input = use_signal(String::new);
    let mut distance_input = use_signal(String::new);
    let mut duration_bell_rung = use_signal(|| false);
    let custom_exercises = storage::use_custom_exercises();
    let all_exercises = exercise_db::use_exercises();
    let pending_ids = use_memo(move || session.read().pending_exercise_ids.clone());
    // Debounce coroutine: drains any already-queued keystrokes, sleeps for the
    // debounce window, drains again to pick up late arrivals, then commits.
    // Uses the cross-platform `sleep_ms` helper so no `#[cfg]` is needed here.
    let debounce_handle = use_coroutine(move |mut rx: UnboundedReceiver<String>| async move {
        use futures_util::StreamExt as _;
        while let Some(q) = rx.next().await {
            let mut latest = q;
            while let Ok(q) = rx.try_recv() {
                latest = q;
            }
            crate::utils::sleep_ms(SEARCH_DEBOUNCE_MS).await;
            while let Ok(q) = rx.try_recv() {
                latest = q;
            }
            debounced_query.set(latest);
        }
    });
    use_effect(move || {
        debounce_handle.send(search_query.read().clone());
    });
    let filter_suggestions = use_memo(move || {
        let query = search_query.read();
        if query.is_empty() {
            return Vec::new();
        }
        let current = active_filters.read();
        detect_filter_suggestions(&query)
            .into_iter()
            .filter(|s| !current.contains(s))
            .collect::<Vec<_>>()
    });
    // Step 1: filter both exercise pools by active chips (re-runs only when chips change).
    let filter_pool = use_memo(move || {
        let custom = custom_exercises.read();
        let all = all_exercises.read();
        let filters = active_filters.read();
        if filters.is_empty() {
            return (custom.clone(), all.clone());
        }
        let filtered_custom: Vec<_> = custom
            .iter()
            .filter(|e| exercise_matches_filters(e.as_ref(), &filters))
            .cloned()
            .collect();
        let filtered_all: Vec<_> = all
            .iter()
            .filter(|e| exercise_matches_filters(e.as_ref(), &filters))
            .cloned()
            .collect();
        (filtered_custom, filtered_all)
    });
    // Step 2: text-search within the pre-filtered pool (re-runs on debounced keystrokes).
    let search_results = use_memo(move || {
        let query = debounced_query.read();
        let has_query = !query.is_empty();
        let has_filters = !active_filters.read().is_empty();
        if !has_query && !has_filters {
            return vec![];
        }
        let (custom_pool, all_pool) = filter_pool();
        let mut results: Vec<Arc<crate::models::Exercise>> = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();
        if has_query {
            let custom_results = exercise_db::search_exercises(&custom_pool, &query);
            for ex in custom_results {
                if seen_ids.insert(ex.id.clone()) {
                    results.push(Arc::clone(ex));
                }
            }
            let db_results = exercise_db::search_exercises(&all_pool, &query);
            for ex in db_results.into_iter().take(MAX_TEXT_SEARCH_RESULTS) {
                if seen_ids.insert(ex.id.clone()) {
                    results.push(Arc::clone(ex));
                }
            }
        } else {
            // Filters only, no text query – show all matching exercises (capped for performance).
            for ex in &custom_pool {
                if seen_ids.insert(ex.id.clone()) {
                    results.push(Arc::clone(ex));
                }
            }
            for ex in all_pool.iter().take(MAX_FILTER_ONLY_RESULTS) {
                if seen_ids.insert(ex.id.clone()) {
                    results.push(Arc::clone(ex));
                }
            }
        }
        results
    });
    let mut start_exercise = move |exercise_id: String| {
        prefill_inputs_from_last_log(&exercise_id, weight_input, reps_input, distance_input);
        let exercise_start = get_current_timestamp();
        search_query.set(String::new());
        debounced_query.set(String::new());
        active_filters.write().clear();
        duration_bell_rung.set(false);
        storage::begin_exercise_in_session(exercise_id, exercise_start);
    };
    let complete_exercise = move |()| {
        let Some(exercise_id) = current_exercise_id() else {
            return;
        };
        let start_time = current_exercise_start().unwrap_or_else(get_current_timestamp);
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
        storage::append_exercise_log(log);
        weight_input.set(String::new());
        reps_input.set(String::new());
        distance_input.set(String::new());
        duration_bell_rung.set(false);
    };
    let cancel_exercise = move |()| {
        weight_input.set(String::new());
        reps_input.set(String::new());
        distance_input.set(String::new());
        storage::cancel_exercise_in_session();
    };
    rsx! {
        Stylesheet { href: asset!("/assets/session.scss") }
        main { class: "session",
            if current_exercise_id().is_none() && !pending_ids().is_empty() {
                PendingExercisesSection {
                    pending_ids: pending_ids(),
                    on_start: move |exercise_id: String| {
                        prefill_inputs_from_last_log(
                            &exercise_id,
                            weight_input,
                            reps_input,
                            distance_input,
                        );
                        let pending_start = get_current_timestamp();
                        search_query.set(String::new());
                        debounced_query.set(String::new());
                        active_filters.write().clear();
                        duration_bell_rung.set(false);
                        storage::start_pending_exercise_in_session(exercise_id, pending_start);
                    },
                }
            }
            if current_exercise_id().is_none() {
                div { class: "inputs",
                    input {
                        r#type: "text",
                        placeholder: "Search for an exercise...",
                        value: "{search_query}",
                        oninput: move |evt| search_query.set(evt.value()),
                    }
                    Link {
                        class: "more",
                        to: Route::AddExercise {},
                        title: "Add Custom Exercise",
                        "+"
                    }
                }
                if !active_filters.read().is_empty() {
                    div { class: "filter-chips",
                        for (i , filter) in active_filters.read().iter().enumerate() {
                            button {
                                class: "filter-chip active",
                                title: "Remove filter",
                                onclick: move |_| {
                                    let mut filters = active_filters.write();
                                    if i < filters.len() {
                                        filters.remove(i);
                                    }
                                },
                                "{filter.label()} ✕"
                            }
                        }
                    }
                }
                if !filter_suggestions.read().is_empty() {
                    div { class: "filter-chips",
                        for suggestion in filter_suggestions.read().iter() {
                            if active_filters.read().len() < MAX_FILTERS {
                                button {
                                    class: "filter-chip suggestion",
                                    title: "Add filter",
                                    onclick: {
                                        let suggestion = suggestion.clone();
                                        move |_| {
                                            active_filters.write().push(suggestion.clone());
                                            search_query.set(String::new());
                                            debounced_query.set(String::new());
                                        }
                                    },
                                    "🔍 {suggestion.label()}"
                                }
                            }
                        }
                    }
                }
                if !search_results().is_empty() {
                    ul { class: "results",
                        for ex in search_results() {
                            li {
                                key: "{ex.id}",
                                onclick: move |_| start_exercise(ex.id.clone()),
                                span { "{ex.name}" }
                                span { class: "category", "{ex.category}" }
                            }
                        }
                    }
                }
            } else if let Some(exercise_id) = current_exercise_id() {
                ExerciseFormPanel {
                    exercise_id,
                    weight_input,
                    reps_input,
                    distance_input,
                    current_exercise_start,
                    duration_bell_rung,
                    paused_at: session.read().paused_at,
                    on_complete: complete_exercise,
                    on_cancel: cancel_exercise,
                }
            }
            if !session.read().exercise_logs.is_empty() {
                CompletedExercisesSection {
                    session,
                    no_exercise_active: current_exercise_id().is_none(),
                    on_replay: move |exercise_id: String| start_exercise(exercise_id),
                }
            }
        }
    }
}
/// Sticky session header rendered in the app-level layout so it remains
/// visible on every page while a workout session is active.
///
/// Clicking the timer block toggles the rest-duration input form via the
/// global [`crate::ShowRestInputSignal`].
/// The finish/cancel button ends or discards the session from any page.
#[component]
pub fn GlobalSessionHeader() -> Element {
    let sessions = storage::use_sessions();
    let session = use_memo(move || sessions.read().iter().find(|s| s.is_active()).cloned());
    let mut show_rest = use_context::<crate::ShowRestInputSignal>().0;
    let rest_duration = use_context::<RestDurationSignal>().0;
    let mut rest_input_value = use_signal(|| 30u64.to_string());
    let mut congratulations = use_context::<crate::CongratulationsSignal>().0;
    use_effect(move || {
        if *show_rest.read() {
            rest_input_value.set(rest_duration.read().to_string());
        }
    });
    let Some(sess) = session() else {
        return rsx! {};
    };
    let exercise_count = sess.exercise_logs.len();
    let session_start_time = sess.start_time;
    let session_is_active = sess.is_active();
    let paused_at = sess.paused_at;
    let total_paused_duration = sess.total_paused_duration;
    let rest_start_time = sess.rest_start_time;
    let on_pause = move |()| {
        let Some(mut s) = session() else { return };
        if s.is_paused() {
            s.resume();
        } else {
            s.pause();
        }
        storage::save_session(s);
    };
    let on_finish = move |()| {
        let Some(s) = session() else { return };
        if s.is_cancelled() {
            storage::delete_session(&s.id);
        } else {
            let mut s = s.clone();
            if s.is_paused() {
                s.resume();
            }
            s.end_time = Some(get_current_timestamp());
            storage::save_session(s);
            congratulations.set(true);
        }
    };
    rsx! {
        SessionHeader {
            session_start_time,
            session_is_active,
            paused_at,
            total_paused_duration,
            exercise_count,
            rest_start_time,
            rest_duration: *rest_duration.read(),
            on_click_timer: move |()| {
                let current = *show_rest.peek();
                show_rest.set(!current);
            },
            on_pause,
            on_finish,
        }
        if *show_rest.read() {
            RestDurationInput {
                show_rest_input: show_rest,
                rest_input_value,
                rest_duration,
            }
        }
    }
}
