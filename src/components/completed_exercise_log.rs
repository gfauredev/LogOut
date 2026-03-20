use crate::models::{Category, ExerciseLog, Force, WorkoutSession};
use crate::services::storage;
use dioxus::prelude::*;

use super::session_exercise_form::ExerciseInputForm;

/// A single completed exercise log entry with inline edit support.
#[component]
pub fn CompletedExerciseLog(
    idx: usize,
    log: ExerciseLog,
    session: Memo<WorkoutSession>,
    /// Called when the user clicks the replay button to start another set.
    #[props(default)]
    on_replay: EventHandler<()>,
    /// Whether to show the replay button (only in an active session with no exercise in progress).
    #[props(default)]
    show_replay: bool,
) -> Element {
    let mut is_editing = use_signal(|| false);
    let mut edit_weight_input = use_signal(String::new);
    let mut edit_reps_input = use_signal(String::new);
    let mut edit_distance_input = use_signal(String::new);

    let start_edit = {
        let log = log.clone();
        move |_| {
            edit_weight_input.set(
                log.weight_hg
                    .map(|w| format!("{:.1}", f64::from(w.0) / 10.0))
                    .unwrap_or_default(),
            );
            edit_reps_input.set(log.reps.map(|r| r.to_string()).unwrap_or_default());
            edit_distance_input.set(
                log.distance_m
                    .map(|d| format!("{:.2}", f64::from(d.0) / 1000.0))
                    .unwrap_or_default(),
            );
            is_editing.set(true);
        }
    };

    let force = log.force;
    let category = log.category;
    let exercise_id = log.exercise_id.clone();
    let last_duration = log.duration_seconds();

    rsx! {
        article {
            header {
                h4 { "{log.exercise_name}" }
                // Action buttons stacked on the right
                div {class: "inputs",
                    if show_replay {
                        button { class: "edit", title: "Do another set",
                            onclick: move |_| on_replay.call(()), "🔁"
                        }
                    }
                    button { class: "edit", onclick: start_edit,
                        title: "Edit this exercise", "✏️"
                    }
                    button { class: "del", title: "Delete this exercise",
                        onclick: move |_| {
                            let mut current_session = session.read().clone();
                            current_session.exercise_logs.remove(idx);
                            storage::save_session(current_session.clone());
                        }, "🗑️"
                    }
                }
            }
            if *is_editing.read() {
                ExerciseInputForm {
                    exercise_id,
                    weight_input: edit_weight_input,
                    reps_input: edit_reps_input,
                    distance_input: edit_distance_input,
                    force,
                    category,
                    last_duration,
                    on_complete: move |()| {
                        let mut current_session = session.read().clone();
                        if let Some(log) = current_session.exercise_logs.get_mut(idx) {
                            use crate::models::{parse_distance_km, parse_weight_kg};
                            log.weight_hg = parse_weight_kg(&edit_weight_input.read());
                            log.reps = if force.is_some_and(Force::has_reps) {
                                edit_reps_input.read().parse().ok()
                            } else {
                                None
                            };
                            if log.category == Category::Cardio {
                                log.distance_m = parse_distance_km(&edit_distance_input.read());
                            }
                        }
                        storage::save_session(current_session);
                        is_editing.set(false);
                        edit_weight_input.set(String::new());
                        edit_reps_input.set(String::new());
                        edit_distance_input.set(String::new());
                    },
                    on_cancel: move |()| is_editing.set(false),
                }
            } else {
                ul {
                    if let Some(w) = log.weight_hg { li { "{w}" } }
                    if let Some(reps) = log.reps { li { "{reps} reps" } }
                    if let Some(d) = log.distance_m { li { "{d}" } }
                    if let Some(duration) = log.duration_seconds() {
                        li { "{crate::models::format_time(duration)}" }
                    }
                }
            }
        }
    }
}
