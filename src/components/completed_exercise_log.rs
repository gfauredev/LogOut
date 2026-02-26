use crate::models::{
    format_time, parse_distance_km, parse_weight_kg, Category, ExerciseLog, WorkoutSession,
};
use crate::services::storage;
use dioxus::prelude::*;

/// A single completed exercise log entry with inline edit support.
#[component]
pub fn CompletedExerciseLog(
    idx: usize,
    log: ExerciseLog,
    session: Signal<WorkoutSession>,
    on_replay: EventHandler<()>,
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
                    .map(|w| format!("{:.1}", w.0 as f64 / 10.0))
                    .unwrap_or_default(),
            );
            edit_reps_input.set(log.reps.map(|r| r.to_string()).unwrap_or_default());
            edit_distance_input.set(
                log.distance_m
                    .map(|d| format!("{:.2}", d.0 as f64 / 1000.0))
                    .unwrap_or_default(),
            );
            is_editing.set(true);
        }
    };

    let save_edit = move |_| {
        let mut current_session = session.read().clone();
        if let Some(log) = current_session.exercise_logs.get_mut(idx) {
            log.weight_hg = parse_weight_kg(&edit_weight_input.read());
            let force = log.force;
            log.reps = if force.is_some_and(|f| f.has_reps()) {
                edit_reps_input.read().parse().ok()
            } else {
                None
            };
            if log.category == Category::Cardio {
                log.distance_m = parse_distance_km(&edit_distance_input.read());
            }
        }
        storage::save_session(current_session.clone());
        session.set(current_session);
        is_editing.set(false);
        edit_weight_input.set(String::new());
        edit_reps_input.set(String::new());
        edit_distance_input.set(String::new());
    };

    let force = log.force;
    let show_reps = force.is_some_and(|f| f.has_reps());
    let is_cardio = log.category == Category::Cardio;

    rsx! {
        article {
            class: "completed-log",

            div {
                class: "completed-log__header",
                h4 { class: "completed-log__title", "{log.exercise_name}" }
                div { class: "completed-log__actions",
                    button {
                        class: "btn--replay-log",
                        title: "Do another set",
                        onclick: move |_| on_replay.call(()),
                        "▶"
                    }
                    button {
                        class: "btn--edit-log",
                        onclick: start_edit,
                        "✏️"
                    }
                    button {
                        class: "btn--delete-log",
                        title: "Delete this exercise",
                        onclick: move |_| {
                            let mut current_session = session.read().clone();
                            current_session.exercise_logs.remove(idx);
                            storage::save_session(current_session.clone());
                            session.set(current_session);
                        },
                        "🗑️"
                    }
                }
            }

            if *is_editing.read() {
                div {
                    class: "completed-log__edit-form",
                    div {
                        label { class: "form-label", "Weight (kg)" }
                        input {
                            r#type: "number",
                            step: "0.5",
                            placeholder: "Optional",
                            value: "{edit_weight_input}",
                            oninput: move |evt| edit_weight_input.set(evt.value()),
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
                                value: "{edit_distance_input}",
                                oninput: move |evt| edit_distance_input.set(evt.value()),
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
                                value: "{edit_reps_input}",
                                oninput: move |evt| edit_reps_input.set(evt.value()),
                                class: "form-input",
                            }
                        }
                    }
                    div {
                        class: "btn-row",
                        button {
                            onclick: save_edit,
                            class: "btn--complete",
                            "✓ Save"
                        }
                        button {
                            onclick: move |_| is_editing.set(false),
                            class: "btn--cancel",
                            "Cancel"
                        }
                    }
                }
            } else {
                div {
                    class: "completed-log__details",
                    if let Some(w) = log.weight_hg {
                        div { "Weight: {w}" }
                    }
                    if let Some(reps) = log.reps {
                        div { "Reps: {reps}" }
                    }
                    if let Some(d) = log.distance_m {
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
