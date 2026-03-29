use super::session_exercise_form::ExerciseInputForm;
use crate::models::{
    format_time, parse_distance_km, parse_duration_seconds, parse_weight_kg, Category, ExerciseLog,
    Force, WorkoutSession, HG_PER_KG, M_PER_KM,
};
use crate::services::{exercise_db, storage};
use dioxus::prelude::*;
use dioxus_i18n::prelude::i18n;
use dioxus_i18n::t;
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
    let mut edit_time_input = use_signal(String::new);
    let start_edit = {
        let log = log.clone();
        move |_| {
            edit_weight_input.set(if log.weight_hg.0 == 0 {
                String::new()
            } else {
                format!("{:.1}", f64::from(log.weight_hg.0) / HG_PER_KG)
            });
            edit_reps_input.set(log.reps.map(|r| r.to_string()).unwrap_or_default());
            edit_distance_input.set(
                log.distance_m
                    .map(|d| format!("{:.2}", f64::from(d.0) / M_PER_KM))
                    .unwrap_or_default(),
            );
            edit_time_input.set(log.duration_seconds().map(format_time).unwrap_or_default());
            is_editing.set(true);
        }
    };
    let all_exercises = exercise_db::use_exercises();
    let custom_exercises = storage::use_custom_exercises();
    let lang_str = use_memo(move || i18n().language().to_string());
    let exercise_id_for_name = log.exercise_id.clone();
    let fallback_name = log.exercise_name.clone();
    let display_name = use_memo(move || {
        let all = all_exercises.read();
        let custom = custom_exercises.read();
        let lang = lang_str.read();
        exercise_db::resolve_exercise(&all, &custom, &exercise_id_for_name).map_or_else(
            || fallback_name.clone(),
            |ex| ex.name_for_lang(&lang).to_owned(),
        )
    });
    let force = log.force;
    let category = log.category;
    let exercise_id = log.exercise_id.clone();
    let last_duration = log.duration_seconds();
    rsx! {
        article {
            header {
                h4 { "{display_name}" }
                div { class: "inputs",
                    if show_replay {
                        button {
                            class: "edit",
                            title: t!("log-replay-title"),
                            onclick: move |_| on_replay.call(()),
                            "🔁"
                        }
                    }
                    button {
                        class: "edit",
                        onclick: start_edit,
                        title: t!("log-edit-title"),
                        "✏️"
                    }
                    button {
                        class: "del",
                        title: t!("log-delete-title"),
                        onclick: move |_| {
                            let mut current_session = session.read().clone();
                            current_session.exercise_logs.remove(idx);
                            storage::save_session(current_session);
                        },
                        "🗑️"
                    }
                }
            }
            if *is_editing.read() {
                ExerciseInputForm {
                    exercise_id,
                    exercise_name: log.exercise_name.clone(),
                    weight_input: edit_weight_input,
                    reps_input: edit_reps_input,
                    distance_input: edit_distance_input,
                    force,
                    category,
                    last_duration,
                    time_input: Some(edit_time_input),
                    on_complete: move |()| {
                        let mut current_session = session.read().clone();
                        if let Some(log) = current_session.exercise_logs.get_mut(idx) {
                            log.weight_hg = parse_weight_kg(&edit_weight_input.read())
                                .unwrap_or_default();
                            log.reps = if force.is_some_and(Force::has_reps) {
                                edit_reps_input.read().parse().ok()
                            } else {
                                None
                            };
                            if log.category == Category::Cardio {
                                log.distance_m = parse_distance_km(&edit_distance_input.read());
                            }
                            let time_str = edit_time_input.read();
                            if !time_str.is_empty() {
                                if let Some(dur) = parse_duration_seconds(&time_str) {
                                    log.end_time = Some(log.start_time + dur);
                                }
                            }
                        }
                        storage::save_session(current_session);
                        is_editing.set(false);
                        edit_weight_input.set(String::new());
                        edit_reps_input.set(String::new());
                        edit_distance_input.set(String::new());
                        edit_time_input.set(String::new());
                    },
                    on_cancel: move |()| is_editing.set(false),
                }
            } else {
                ul {
                    if log.weight_hg.0 > 0 {
                        li { "{log.weight_hg}" }
                    }
                    if let Some(reps) = log.reps {
                        li { "{reps} reps" }
                    }
                    if let Some(d) = log.distance_m {
                        li { "{d}" }
                    }
                    if let Some(duration) = log.duration_seconds() {
                        li { "{crate::models::format_time(duration)}" }
                    }
                }
            }
        }
    }
}
