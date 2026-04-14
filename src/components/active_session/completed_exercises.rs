use crate::components::CompletedExerciseLog;
use crate::models::WorkoutSession;
use crate::services::{exercise_db, storage};
use dioxus::prelude::*;
use dioxus_i18n::prelude::i18n;
use dioxus_i18n::t;

/// Antichronological list of completed exercise logs with replay and edit actions.
/// Fires `on_replay` with the exercise ID when the user taps 🔁.
///
/// When no exercise is active and the last completed exercise was also done
/// earlier in the session, a quick-action button is shown at the top suggesting
/// the exercise that followed that earlier set.
#[component]
pub fn CompletedExercisesSection(
    session: Memo<WorkoutSession>,
    no_exercise_active: bool,
    on_replay: EventHandler<String>,
) -> Element {
    let all_exercises = exercise_db::use_exercises();
    let custom_exercises = storage::use_custom_exercises();
    let lang_str = use_memo(move || i18n().language().to_string());
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
            let lang = lang_str.read();
            let name = exercise_db::resolve_exercise(&all, &custom, &id)
                .map_or(fallback_name, |ex| ex.name_for_lang(&lang).to_owned());
            (id, name)
        })
    });
    rsx! {
        section { // class: "exercises",


            h3 { {t!("completed-exercises-title")} }
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
                    for (idx, log) in session.read().exercise_logs.iter().enumerate().rev() {
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
