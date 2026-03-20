use crate::models::{
    format_time, parse_distance_km, parse_weight_kg, Category, ExerciseLog, Force,
};
use crate::services::{exercise_db, storage};
use dioxus::prelude::*;

use super::session_timers::ExerciseElapsedTimer;

/// The active exercise input form.
///
/// Renders the weight / reps / distance inputs for the currently-performing
/// exercise and exposes "Complete Exercise" and "Cancel" actions via event
/// handlers so all state mutation stays in the parent [`super::active_session::SessionView`].
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
    current_exercise_start: Signal<Option<u64>>,
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

    // Secondary static timer: shown when exercise has no reps and no distance
    let show_static_timer = !show_reps && !is_cardio;

    rsx! {
        article {
            header {
                h3 { "{exercise_name}" }
                if let Some(dur) = last_duration {
                    div { label { "⏳" } time {"{format_time(dur)}"} }
                }
            }
            if show_static_timer {
                ExerciseElapsedTimer {
                    exercise_start: *current_exercise_start.read(),
                    last_duration,
                    duration_bell_rung,
                    paused_at,
                }
            }
            {
                let weight = weight_input.read();
                let weight_invalid = !weight.is_empty() && parse_weight_kg(&weight).is_none();
                rsx! {
                    div { class: "inputs",
                        label { "Weight" }
                        button { class: "no",
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
                        button { class: "yes",
                            r#type: "button",
                            tabindex: -1,
                            onclick: move |_| {
                                let cur: f64 = weight_input.read().parse().unwrap_or(0.0);
                                weight_input.set(format!("{:.1}", cur + 0.5));
                            },
                            "+"
                        }
                    }
                }
            }
            if is_cardio {
                {
                    let dist = distance_input.read();
                    let distance_invalid = !dist.is_empty() && parse_distance_km(&dist).is_none();
                    rsx! {
                        div { class: "inputs",
                            label { "Distance" }
                            // TODO decrement and increment buttons too
                            input {
                                r#type: "number",
                                inputmode: "decimal",
                                step: "0.1",
                                placeholder: "km",
                                value: "{distance_input}",
                                oninput: move |evt| distance_input.set(evt.value()),
                                class: if distance_invalid { "invalid" } else { "" },
                            }
                        }
                    }
                }
            }
            if show_reps {
                {
                    let reps = reps_input.read();
                    let reps_invalid = !reps.is_empty() && reps.parse::<u32>().map(|r| r == 0).unwrap_or(true);
                    rsx! {
                        div { class: "inputs",
                            label { "Repetitions" }
                            button { class: "no",
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
                            button { class: "yes",
                                r#type: "button",
                                tabindex: -1,
                                onclick: move |_| {
                                    let cur: u32 = reps_input.read().parse().unwrap_or(0);
                                    reps_input.set((cur + 1).to_string());
                                },
                                "+"
                            }
                        }
                    }
                }
            }
            {
                let weight = weight_input.read();
                let reps = reps_input.read();
                let dist = distance_input.read();
                let weight_valid = weight.is_empty() || parse_weight_kg(&weight).is_some();
                let reps_valid = !show_reps || reps.parse::<u32>().map(|r| r > 0).unwrap_or(false);
                let distance_valid = !is_cardio || parse_distance_km(&dist).is_some();
                let complete_disabled = !weight_valid || !reps_valid || !distance_valid;
                rsx! {
                    footer {
                        button { class: "yes",
                            onclick: move |_| on_complete.call(()),
                            disabled: complete_disabled,
                            title: "Complete Exercise", "💾"
                        }
                        button { class: "no",
                            onclick: move |_| on_cancel.call(()),
                            "❌"
                        }
                    }
                }
            }
        }
    }
}
