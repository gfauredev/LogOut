use crate::components::CompletedExerciseLog;
use crate::models::{
    format_time, get_current_timestamp, parse_distance_km, parse_weight_kg, Category, ExerciseLog,
    WorkoutSession,
};
use crate::services::{exercise_db, storage};
use crate::Route;
use dioxus::prelude::*;

/// Default rest duration in seconds
const DEFAULT_REST_DURATION: u64 = 30;
/// Timer tick interval in milliseconds
#[cfg(target_arch = "wasm32")]
const TIMER_TICK_MS: u32 = 1_000;
/// Snackbar auto-dismiss delay in milliseconds
#[cfg(target_arch = "wasm32")]
const SNACKBAR_DISMISS_MS: u32 = 3_000;

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

        let weight_dg = parse_weight_kg(&weight_input.read());
        let reps = if force.is_some_and(|f| f.has_reps()) {
            reps_input.read().parse().ok()
        } else {
            None
        };
        let distance_dam = if category == Category::Cardio {
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
            weight_dg,
            reps,
            distance_dam,
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
                                                    // Prefill from last log
                                                    if let Some(last_log) = storage::get_last_exercise_log(&id) {
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

                                let show_reps = force.is_some_and(|f| f.has_reps());
                                let is_cardio = category == Category::Cardio;
                                let last_log = storage::get_last_exercise_log(exercise_id);
                                let last_duration = last_log.as_ref()
                                    .and_then(|log| log.duration_seconds());

                                // Secondary static timer: shown when exercise has no reps and no distance
                                let show_static_timer = !show_reps && !is_cardio;

                                rsx! {
                                    header { class: "exercise-form__header",
                                    h3 { class: "exercise-form__title", "{exercise_name}" }
                                    if let Some(dur) = last_duration {
                                        span {
                                            class: "exercise-form__last-duration",
                                            "Last duration: {format_time(dur)}"
                                        }
                                    }
                                    }

                                    if show_static_timer {
                                        ExerciseElapsedTimer {
                                            exercise_start: *current_exercise_start.read(),
                                            last_duration,
                                            duration_bell_rung,
                                        }
                                    }

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
                                                    // Persist cleared performing state
                                                    let mut current_session = session.read().clone();
                                                    current_session.current_exercise_id = None;
                                                    current_session.current_exercise_start = None;
                                                    session.set(current_session.clone());
                                                    storage::save_session(current_session);
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

/// Send a notification using the Web Notifications API.
/// The system decides whether to play audio or vibrate.
/// `is_duration_bell` selects a different message to distinguish from rest alerts.
#[cfg(target_arch = "wasm32")]
fn send_notification(is_duration_bell: bool) {
    use web_sys::{Notification, NotificationOptions, NotificationPermission};

    let (title, body) = if is_duration_bell {
        ("Duration reached", "Target exercise duration reached!")
    } else {
        ("Rest over", "Time to start your next set!")
    };

    let send = |t: &str, b: &str| {
        let opts = NotificationOptions::new();
        opts.set_body(b);
        let _ = Notification::new_with_options(t, &opts);
    };

    match Notification::permission() {
        NotificationPermission::Granted => send(title, body),
        NotificationPermission::Default => {
            let title = title.to_string();
            let body = body.to_string();
            if let Ok(promise) = Notification::request_permission() {
                wasm_bindgen_futures::spawn_local(async move {
                    if wasm_bindgen_futures::JsFuture::from(promise).await.is_ok()
                        && Notification::permission() == NotificationPermission::Granted
                    {
                        let opts = NotificationOptions::new();
                        opts.set_body(&body);
                        let _ = Notification::new_with_options(&title, &opts);
                    }
                });
            }
        }
        _ => {}
    }
}

// ── Isolated timer components ──────────────────────────────────────────────
// Each component owns its own tick coroutine so that only the timer display
// re-renders every second, preventing unnecessary re-renders of the main
// session form (input fields, exercise list, etc.).

/// Renders the session elapsed time, updating every second.
#[component]
fn SessionDurationDisplay(session_start_time: u64, session_is_active: bool) -> Element {
    let mut now_tick = use_signal(get_current_timestamp);
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        loop {
            #[cfg(target_arch = "wasm32")]
            gloo_timers::future::TimeoutFuture::new(TIMER_TICK_MS).await;
            #[cfg(not(target_arch = "wasm32"))]
            std::future::pending::<()>().await;
            now_tick.set(get_current_timestamp());
        }
    });
    let tick = *now_tick.read();
    let duration = if session_is_active {
        tick.saturating_sub(session_start_time)
    } else {
        0
    };
    rsx! { "{format_time(duration)}" }
}

/// Renders the rest timer and fires a notification when the rest period ends.
#[component]
fn RestTimerDisplay(
    rest_start_time: Signal<Option<u64>>,
    rest_duration: Signal<u64>,
    mut rest_bell_count: Signal<u64>,
) -> Element {
    let mut now_tick = use_signal(get_current_timestamp);
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        loop {
            #[cfg(target_arch = "wasm32")]
            gloo_timers::future::TimeoutFuture::new(TIMER_TICK_MS).await;
            #[cfg(not(target_arch = "wasm32"))]
            std::future::pending::<()>().await;
            now_tick.set(get_current_timestamp());
        }
    });

    let tick = *now_tick.read();
    let Some(start) = *rest_start_time.read() else {
        return rsx! {};
    };
    let elapsed = tick.saturating_sub(start);
    let rd = *rest_duration.read();

    // Fire bell at each completed rest interval
    if rd > 0 && elapsed > 0 {
        let intervals = elapsed / rd;
        let prev_count = *rest_bell_count.read();
        if intervals > prev_count {
            rest_bell_count.set(intervals);
            #[cfg(target_arch = "wasm32")]
            send_notification(false);
        }
    }

    let exceeded = rd > 0 && elapsed >= rd;
    rsx! {
        div {
            class: if exceeded { "rest-timer rest-timer--exceeded" } else { "rest-timer" },
            "🛋️ Rest: {format_time(elapsed)}"
        }
    }
}

/// Renders the exercise elapsed timer and fires a notification when the
/// target duration from the last log is reached.
#[component]
fn ExerciseElapsedTimer(
    exercise_start: Option<u64>,
    last_duration: Option<u64>,
    mut duration_bell_rung: Signal<bool>,
) -> Element {
    let mut now_tick = use_signal(get_current_timestamp);
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        loop {
            #[cfg(target_arch = "wasm32")]
            gloo_timers::future::TimeoutFuture::new(TIMER_TICK_MS).await;
            #[cfg(not(target_arch = "wasm32"))]
            std::future::pending::<()>().await;
            now_tick.set(get_current_timestamp());
        }
    });

    let tick = *now_tick.read();
    let elapsed = if let Some(start) = exercise_start {
        tick.saturating_sub(start)
    } else {
        0
    };

    // Fire duration bell once when the previous exercise duration is reached
    if !*duration_bell_rung.read() {
        if let Some(dur) = last_duration {
            if dur > 0 && elapsed >= dur {
                duration_bell_rung.set(true);
                #[cfg(target_arch = "wasm32")]
                send_notification(true);
            }
        }
    }

    let timer_reached = last_duration.is_some_and(|d| d > 0 && elapsed >= d);
    rsx! {
        div {
            class: if timer_reached { "exercise-static-timer exercise-static-timer--reached" } else { "exercise-static-timer" },
            "⏱ {format_time(elapsed)}"
        }
    }
}
