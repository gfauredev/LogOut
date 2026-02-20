use crate::models::{format_time, parse_distance_km, parse_weight_kg, Category};
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
    /// Called when the user clicks "✓ Complete Exercise".
    on_complete: EventHandler<()>,
    /// Called when the user clicks "Cancel".
    on_cancel: EventHandler<()>,
) -> Element {
    let all_exercises = exercise_db::use_exercises();
    let custom_exercises = storage::use_custom_exercises();

    let (exercise_name, category, force) = {
        let all = all_exercises.read();
        if let Some(ex) = exercise_db::get_exercise_by_id(&all, &exercise_id) {
            (ex.name.clone(), ex.category, ex.force)
        } else {
            let custom = custom_exercises.read();
            if let Some(ex) = custom.iter().find(|e| e.id == exercise_id) {
                (ex.name.clone(), ex.category, ex.force)
            } else {
                ("Unknown".to_string(), Category::Strength, None)
            }
        }
    };

    let show_reps = force.is_some_and(|f| f.has_reps());
    let is_cardio = category == Category::Cardio;
    let last_log = storage::get_last_exercise_log(&exercise_id);
    let last_duration = last_log.as_ref().and_then(|log| log.duration_seconds());

    // Secondary static timer: shown when exercise has no reps and no distance
    let show_static_timer = !show_reps && !is_cardio;

    rsx! {
        article {
            class: "exercise-form",

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

                {
                    let weight = weight_input.read();
                    let weight_invalid = !weight.is_empty() && parse_weight_kg(&weight).is_none();
                    rsx! {
                        div {
                            label { class: "form-label", "Weight (kg)" }
                            input {
                                r#type: "number",
                                step: "0.1",
                                placeholder: "Optional",
                                value: "{weight_input}",
                                oninput: move |evt| weight_input.set(evt.value()),
                                class: if weight_invalid { "form-input form-input--invalid" } else { "form-input" },
                            }
                        }
                    }
                }

                if is_cardio {
                    {
                        let dist = distance_input.read();
                        let distance_invalid = !dist.is_empty() && parse_distance_km(&dist).is_none();
                        rsx! {
                            div {
                                label { class: "form-label", "Distance (km)" }
                                input {
                                    r#type: "number",
                                    step: "0.1",
                                    placeholder: "Distance",
                                    value: "{distance_input}",
                                    oninput: move |evt| distance_input.set(evt.value()),
                                    class: if distance_invalid { "form-input form-input--invalid" } else { "form-input" },
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
                            div {
                                label { class: "form-label", "Repetitions" }
                                input {
                                    r#type: "number",
                                    placeholder: "Reps",
                                    value: "{reps_input}",
                                    oninput: move |evt| reps_input.set(evt.value()),
                                    class: if reps_invalid { "form-input form-input--invalid" } else { "form-input" },
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
                        div {
                            class: "btn-row",
                            button {
                                onclick: move |_| on_complete.call(()),
                                disabled: complete_disabled,
                                class: "btn--complete",
                                "✓ Complete Exercise"
                            }
                            button {
                                onclick: move |_| on_cancel.call(()),
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
