use crate::models::Category;
use crate::services::{exercise_db, storage};
use dioxus::prelude::*;
use dioxus_i18n::t;

/// List of exercises pre-added to the session that haven't been started yet.
/// The first (oldest) exercise is always visible and directly clickable.
/// Any additional exercises are hidden inside a folded `<details>` dropdown.
/// Fires `on_start` with the exercise ID when the user taps 🔁.
#[component]
pub fn PendingExercisesSection(
    pending_ids: Vec<String>,
    on_start: EventHandler<String>,
) -> Element {
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
                    summary { {t!("pending-more", count : (resolved.len() - 1).to_string())} }
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
