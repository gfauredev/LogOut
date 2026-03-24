use super::session_timers::ExerciseElapsedTimer;
use crate::models::{
    format_time, parse_distance_km, parse_weight_kg, Category, ExerciseLog, Force,
};
use crate::services::{exercise_db, storage};
use dioxus::prelude::*;
/// Shared exercise input form used both for performing a new set and for
/// editing a completed log entry.
///
/// Renders a CSS grid with five columns (metric · − · value · + · 🏆) and up to
/// four rows (⏱️ · ⚖️ · 📏 · 🔢).  A heading row at the top spans the first four
/// columns with the exercise name; the fifth column keeps the 🏆 header icon.
/// Rows that are not applicable for the current exercise (e.g. no-reps exercises
/// skip 🔢) are omitted.
#[component]
pub(super) fn ExerciseInputForm(
    /// ID of the exercise (used to look up personal records).
    exercise_id: String,
    /// Display name of the exercise shown in the grid heading.
    exercise_name: String,
    weight_input: Signal<String>,
    reps_input: Signal<String>,
    distance_input: Signal<String>,
    force: Option<Force>,
    category: Category,
    /// Duration of the most recent log for this exercise (previous set).
    last_duration: Option<u64>,
    on_complete: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut weight_input = weight_input;
    let mut reps_input = reps_input;
    let mut distance_input = distance_input;
    let show_reps = force.is_some_and(Force::has_reps);
    let is_cardio = category == Category::Cardio;
    let bests = storage::get_exercise_bests(&exercise_id);
    let weight = weight_input.read();
    let weight_invalid = !weight.is_empty() && parse_weight_kg(&weight).is_none();
    let reps = reps_input.read();
    let reps_invalid = !reps.is_empty() && reps.parse::<u32>().map(|r| r == 0).unwrap_or(true);
    let dist = distance_input.read();
    let distance_invalid = !dist.is_empty() && parse_distance_km(&dist).is_none();
    let weight_valid = weight.is_empty() || parse_weight_kg(&weight).is_some();
    let reps_valid = !show_reps || reps.parse::<u32>().map(|r| r > 0).unwrap_or(false);
    let distance_valid = !is_cardio || parse_distance_km(&dist).is_some();
    let complete_disabled = !weight_valid || !reps_valid || !distance_valid;
    let show_duration_row = last_duration.is_some() || bests.duration.is_some();
    rsx! {
        div { class: "exercise-edit",
            h3 { "{exercise_name}" }
            span { "🏆" }
            // ⏱️ Time display (or input in edit mode) and ATH
            if show_duration_row {
                div { class: "input-row", // duration-row",
                    span { "⏱️" }
                    span {} // TODO Only when editing, not performing
                    span {
                        if let Some(dur) = last_duration {
                            // TODO Increment while performing
                            time { "{format_time(dur)}" }
                        }
                    }
                    span {} // TODO Only when editing, not performing
                    span {
                        if let Some(best) = bests.duration {
                            if last_duration.is_none_or(|prev| best > prev) {
                                time { "{format_time(best)}" }
                            }
                        }
                    }
                }
            }
            // ⚖️ Weight input and ATH
            div { class: "input-row",
                span { "⚖️" }
                button {
                    class: "less",
                    r#type: "button",
                    tabindex: -1,
                    onclick: move |_| {
                        let cur: f64 = weight_input.read().parse().unwrap_or(0.0);
                        let next = (cur - 0.5).max(0.0);
                        weight_input.set(format!("{next:.1}"));
                    },
                    "−"
                }
                input {
                    r#type: "number",
                    inputmode: "decimal",
                    step: "0.1",
                    placeholder: "kg",
                    value: "{weight_input}",
                    oninput: move |evt| weight_input.set(evt.value()),
                    class: if weight_invalid { "invalid" } else { "" },
                }
                button {
                    class: "more",
                    r#type: "button",
                    tabindex: -1,
                    onclick: move |_| {
                        let cur: f64 = weight_input.read().parse().unwrap_or(0.0);
                        weight_input.set(format!("{:.1}", cur + 0.5));
                    },
                    "+"
                }
                if let Some(best) = bests.weight_hg {
                    span { "{best}" }
                } else {
                    span { "0" }
                }
            }
            // 📏 Distance input (cardio exercises only) and ATH
            if is_cardio {
                div { class: "input-row",
                    span { "📏" }
                    button {
                        class: "less",
                        r#type: "button",
                        tabindex: -1,
                        onclick: move |_| {
                            let cur: f64 = distance_input.read().parse().unwrap_or(0.0);
                            let next = (cur - 0.1).max(0.0);
                            distance_input.set(format!("{next:.2}"));
                        },
                        "−"
                    }
                    input {
                        r#type: "number",
                        inputmode: "decimal",
                        step: "0.1",
                        placeholder: "km",
                        value: "{distance_input}",
                        oninput: move |evt| distance_input.set(evt.value()),
                        class: if distance_invalid { "invalid" } else { "" },
                    }
                    button {
                        class: "more",
                        r#type: "button",
                        tabindex: -1,
                        onclick: move |_| {
                            let cur: f64 = distance_input.read().parse().unwrap_or(0.0);
                            distance_input.set(format!("{:.2}", cur + 0.1));
                        },
                        "+"
                    }
                    if let Some(best) = bests.distance_m {
                        span { "{best}" }
                    } else {
                        span { "0" }
                    }
                }
            }
            // 🔢 Repetitions input and ATH
            if show_reps {
                div { class: "input-row",
                    span { "🔢" }
                    button {
                        class: "less",
                        r#type: "button",
                        tabindex: -1,
                        onclick: move |_| {
                            let cur: u32 = reps_input.read().parse().unwrap_or(0);
                            if cur > 1 {
                                reps_input.set((cur - 1).to_string());
                            }
                        },
                        "−"
                    }
                    input {
                        r#type: "number",
                        inputmode: "numeric",
                        placeholder: "reps",
                        value: "{reps_input}",
                        oninput: move |evt| reps_input.set(evt.value()),
                        class: if reps_invalid { "invalid" } else { "" },
                    }
                    button {
                        class: "more",
                        r#type: "button",
                        tabindex: -1,
                        onclick: move |_| {
                            let cur: u32 = reps_input.read().parse().unwrap_or(0);
                            reps_input.set((cur + 1).to_string());
                        },
                        "+"
                    }
                    if let Some(best) = bests.reps {
                        span { class: "ath", "{best}" }
                    } else {
                        span { "0" }
                    }
                }
            }
        }
        footer {
            button {
                class: "save",
                onclick: move |_| on_complete.call(()),
                disabled: complete_disabled,
                title: "Complete Exercise",
                "💾"
            }
            button { class: "back", onclick: move |_| on_cancel.call(()), "❌" }
        }
    }
}
/// The active exercise input form.
///
/// Renders the exercise name (embedded in the grid heading), optional elapsed
/// timer and previous/ATH durations, then delegates the metric inputs to
/// [`ExerciseInputForm`].  All state mutation stays in the parent
/// [`super::active_session::SessionView`].
#[component]
pub(super) fn ExerciseFormPanel(
    /// ID of the exercise currently being performed.
    exercise_id: String,
    /// Reactive weight input (kg as a string).
    weight_input: Signal<String>,
    /// Reactive reps input.
    reps_input: Signal<String>,
    /// Reactive distance input (km as a string).
    distance_input: Signal<String>,
    /// Timestamp when the current exercise started.
    current_exercise_start: ReadSignal<Option<u64>>,
    /// Tracks whether the duration bell has fired for this exercise.
    duration_bell_rung: Signal<bool>,
    /// Timestamp when the session was paused; `None` when running.
    paused_at: Option<u64>,
    /// Called when the user clicks "✓ Complete Exercise".
    on_complete: EventHandler<()>,
    /// Called when the user clicks "Cancel".
    on_cancel: EventHandler<()>,
) -> Element {
    let all_exercises = exercise_db::use_exercises();
    let custom_exercises = storage::use_custom_exercises();
    let (exercise_name, category, force) = {
        let all = all_exercises.read();
        let custom = custom_exercises.read();
        if let Some(ex) = exercise_db::resolve_exercise(&all, &custom, &exercise_id) {
            (ex.name.clone(), ex.category, ex.force)
        } else {
            ("Unknown".to_string(), Category::Strength, None)
        }
    };
    let show_reps = force.is_some_and(Force::has_reps);
    let is_cardio = category == Category::Cardio;
    let last_log = storage::get_last_exercise_log(&exercise_id);
    let last_duration = last_log.as_ref().and_then(ExerciseLog::duration_seconds);
    let show_static_timer = !show_reps && !is_cardio;
    rsx! {
        article {
            if show_static_timer {
                ExerciseElapsedTimer {
                    exercise_start: *current_exercise_start.read(),
                    last_duration,
                    duration_bell_rung,
                    paused_at,
                }
            }
            ExerciseInputForm {
                exercise_id,
                exercise_name,
                weight_input,
                reps_input,
                distance_input,
                force,
                category,
                last_duration,
                on_complete,
                on_cancel,
            }
        }
    }
}
