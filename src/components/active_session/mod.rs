use super::session_exercise_form::ExerciseFormPanel;
use crate::models::{
    get_current_timestamp, parse_distance_km, parse_weight_kg, Category, ExerciseLog, Force,
    Weight, WorkoutSession, HG_PER_KG, M_PER_KM,
};
use crate::services::exercise_db::{
    detect_filter_suggestions, exercise_matches_filters, SearchFilter,
};
use crate::services::{exercise_db, storage};
use crate::{RestDurationSignal, Route};
use dioxus::prelude::*;
use dioxus_i18n::prelude::i18n;
use dioxus_i18n::t;
use futures_channel::mpsc::UnboundedReceiver;
#[cfg(target_arch = "wasm32")]
use gloo_timers::future::TimeoutFuture;
use std::sync::Arc;

mod completed_exercises;
mod header;
mod pending_exercises;
mod rest_input;

pub use completed_exercises::CompletedExercisesSection;
pub use header::SessionHeader;
pub use pending_exercises::PendingExercisesSection;
pub use rest_input::RestDurationInput;

/// Maximum number of simultaneously active hard filters in the session search.
const MAX_FILTERS: usize = 4;
/// Debounce delay in milliseconds before re-running the expensive exercise filter.
const SEARCH_DEBOUNCE_MS: u32 = 200;
/// Maximum exercises shown when only attribute filters are active and there is no text query.
const MAX_FILTER_ONLY_RESULTS: usize = 20;
/// Maximum exercises shown from the full database when a text search query is active.
const MAX_TEXT_SEARCH_RESULTS: usize = 10;
/// Default rest time in seconds offered to the user in the rest input form.
const DEFAULT_REST_SECONDS: u64 = 30;

/// Prefill the weight / reps / distance inputs from the last recorded log for
/// `exercise_id`, or clear them if no prior log exists.
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
        (Some(_) | None, None) => true,
        (None, Some(_)) => false,
    };

    if use_active {
        if let Some(last_log) = active_log {
            if last_log.weight_hg.0 > 0 {
                weight_input.set(format!(
                    "{:.1}",
                    f64::from(last_log.weight_hg.0) / HG_PER_KG
                ));
            } else {
                weight_input.set(String::new());
            }
            if let Some(reps) = last_log.reps {
                reps_input.set(reps.to_string());
            } else {
                reps_input.set(String::new());
            }
            if let Some(d) = last_log.distance_m {
                distance_input.set(format!("{:.2}", f64::from(d.0) / M_PER_KM));
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
            weight_input.set(format!("{:.1}", f64::from(w.0) / HG_PER_KG));
        } else {
            weight_input.set(String::new());
        }
        if let Some(reps) = bests.last_reps {
            reps_input.set(reps.to_string());
        } else {
            reps_input.set(String::new());
        }
        if let Some(d) = bests.last_distance_m {
            distance_input.set(format!("{:.2}", f64::from(d.0) / M_PER_KM));
        } else {
            distance_input.set(String::new());
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
    let lang_str = use_memo(move || i18n().language().to_string());
    let mut notes_input = use_signal(|| session.read().notes.clone());
    // Track the session ID so we can distinguish between:
    //   (a) the debounce saving the user's own input for the *same* session
    //       → do NOT touch the DOM (would reset cursor on Android)
    //   (b) a *different* session being loaded
    //       → update both the signal and the DOM
    let mut last_synced_session_id = use_signal(|| session.read().id.clone());
    use_effect(move || {
        let s = session.read();
        let new_id = s.id.clone();
        let new_notes = s.notes.clone();
        if new_id != *last_synced_session_id.peek() {
            // Different session loaded – update signal and DOM.
            last_synced_session_id.set(new_id);
            notes_input.set(new_notes.clone());
            spawn(async move {
                let val_js = serde_json::to_string(&new_notes).unwrap_or_default();
                document::eval(&format!(
                    "var el=document.getElementById('session-notes-input');if(el)el.value={val_js};"
                ));
            });
        }
        // Same session: notes changed because the debounce saved the user's
        // own input.  Leave the DOM alone to avoid resetting the cursor.
    });
    let notes_debounce = use_coroutine(move |mut rx: UnboundedReceiver<String>| async move {
        use futures_util::StreamExt as _;
        while let Some(text) = rx.next().await {
            let mut latest = text;
            while let Ok(t) = rx.try_recv() {
                latest = t;
            }
            crate::utils::sleep_ms(400).await;
            while let Ok(t) = rx.try_recv() {
                latest = t;
            }
            // Retrieve the current session, update notes, and persist.
            let sessions_w = storage::use_sessions();
            let active = sessions_w.read().iter().find(|s| s.is_active()).cloned();
            let _ = sessions_w;
            if let Some(mut s) = active {
                s.notes = latest;
                storage::save_session(s);
            }
        }
    });

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

    let search_results = use_memo(move || {
        let query = debounced_query.read();
        let has_query = !query.is_empty();
        let has_filters = !active_filters.read().is_empty();
        if !has_query && !has_filters {
            return vec![];
        }
        let (custom_pool, all_pool) = filter_pool();
        let lang = lang_str.read();
        let mut results: Vec<Arc<crate::models::Exercise>> = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();
        if has_query {
            let custom_results = exercise_db::search_exercises(&custom_pool, &query, &lang);
            for ex in custom_results {
                if seen_ids.insert(ex.id.clone()) {
                    results.push(Arc::clone(ex));
                }
            }
            let db_results = exercise_db::search_exercises(&all_pool, &query, &lang);
            for ex in db_results.into_iter().take(MAX_TEXT_SEARCH_RESULTS) {
                if seen_ids.insert(ex.id.clone()) {
                    results.push(Arc::clone(ex));
                }
            }
        } else {
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
        let weight_hg = if category == Category::Stretching {
            Weight::default()
        } else {
            parse_weight_kg(&weight_input.read()).unwrap_or_default()
        };
        let reps = if category != Category::Cardio && force.is_some_and(Force::has_reps) {
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
                        placeholder: t!("session-search-placeholder"),
                        value: "{search_query}",
                        oninput: move |evt| search_query.set(evt.value()),
                    }
                    Link {
                        class: "more",
                        to: Route::AddExercise {},
                        title: t!("session-add-exercise-title"),
                        "+"
                    }
                }
                if !active_filters.read().is_empty() {
                    div { class: "filter-chips",
                        for (i, filter) in active_filters.read().iter().enumerate() {
                            button {
                                class: "filter-chip active",
                                title: t!("session-filter-remove"),
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
                                    title: t!("session-filter-add"),
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
                                span { "{ex.name_for_lang(&lang_str.read())}" }
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
            textarea {
                id: "session-notes-input",
                placeholder: t!("session-notes-placeholder"),
                // Do NOT bind `value` here.  A controlled textarea causes
                // Dioxus to overwrite the DOM value on every re-render, which
                // resets the cursor position to the end on Android's WebView
                // (especially visible when typing fast with the IME).
                // Instead we set the initial DOM value via `onmounted` (fires
                // once the element is in the DOM) and keep `notes_input` in
                // sync via `oninput` so the effect can detect session changes.
                onmounted: move |_| {
                    let notes = notes_input.peek().clone();
                    if !notes.is_empty() {
                        let val_js = serde_json::to_string(&notes).unwrap_or_default();
                        document::eval(
                            &format!(
                                "var el=document.getElementById('session-notes-input');if(el)el.value={val_js};",
                            ),
                        );
                    }
                },
                oninput: move |evt| {
                    let text = evt.value();
                    notes_input.set(text.clone());
                    notes_debounce.send(text);
                },
            }
        }
    }
}

#[component]
pub fn GlobalSessionHeader() -> Element {
    let sessions = storage::use_sessions();
    let session = use_memo(move || sessions.read().iter().find(|s| s.is_active()).cloned());
    let mut show_rest = use_context::<crate::ShowRestInputSignal>().0;
    let rest_duration = use_context::<RestDurationSignal>().0;
    let mut rest_input_value = use_signal(|| DEFAULT_REST_SECONDS.to_string());
    let mut congratulations = use_context::<crate::CongratulationsSignal>().0;

    // A memo that captures the (rest_start_time, rest_duration) pair so the
    // notification effect only re-fires when the rest period actually changes.
    let rest_key = use_memo(move || {
        let rd = *rest_duration.read();
        session()
            .and_then(|s| s.rest_start_time)
            .map(|start| (start, rd))
    });

    // Track how many rest-exceeded intervals have fired for the current rest
    // period.  Reset to 0 each time a new rest period begins.
    let mut rest_bell_count = use_signal(|| 0u64);

    // Pre-localise the notification strings in the reactive context so they
    // can be moved into async closures without requiring i18n context access.
    let rest_notif_title = use_memo(move || t!("notif-rest-title").to_string());
    let rest_notif_body = use_memo(move || t!("notif-rest-body").to_string());

    // Schedule a precise one-shot rest-over notification whenever a new rest
    // period begins.  Fires ~250 ms early to compensate for jitter.
    use_effect(move || {
        let Some((start, duration)) = rest_key() else {
            return;
        };
        if duration == 0 {
            return;
        }
        // Reset the exceeded-interval counter for the new rest period.
        rest_bell_count.set(0);

        let title = rest_notif_title.peek().clone();
        let body = rest_notif_body.peek().clone();
        let fire_at_secs = start + duration;

        #[cfg(not(target_arch = "wasm32"))]
        {
            let now = crate::models::get_current_timestamp();
            if fire_at_secs > now {
                let delay_ms = ((fire_at_secs - now) * 1_000)
                    .saturating_sub(crate::components::session_timers::NOTIF_EARLY_MS);
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                    crate::services::notifications::send_notification(&title, &body, "logout-rest");
                });
            } else {
                crate::services::notifications::send_notification(&title, &body, "logout-rest");
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let now = crate::models::get_current_timestamp();
            let delay_ms = if fire_at_secs > now {
                ((fire_at_secs - now) * 1_000)
                    .saturating_sub(crate::components::session_timers::NOTIF_EARLY_MS)
                    .min(u32::MAX as u64) as u32
            } else {
                0
            };
            wasm_bindgen_futures::spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(delay_ms).await;
                crate::services::notifications::send_notification(&title, &body, "logout-rest");
            });
        }
    });

    // Tick-based coroutine: fires a notification for every completed exceeded
    // interval (2nd, 3rd, … ring) so the user keeps being reminded.
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        loop {
            crate::utils::sleep_ms(1_000).await;
            let Some((start, duration)) = *rest_key.peek() else {
                continue;
            };
            if duration == 0 {
                continue;
            }
            let now = crate::models::get_current_timestamp();
            let elapsed = now.saturating_sub(start);
            let intervals = elapsed / duration;
            let prev = *rest_bell_count.peek();
            if intervals > prev {
                rest_bell_count.set(intervals);
                crate::services::notifications::send_notification(
                    &rest_notif_title.peek(),
                    &rest_notif_body.peek(),
                    "logout-rest",
                );
            }
        }
    });

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
