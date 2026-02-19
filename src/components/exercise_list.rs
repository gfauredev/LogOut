use crate::components::{ActiveTab, BottomNav, ExerciseCard};
use crate::services::{exercise_db, storage};
use dioxus::prelude::*;

#[component]
pub fn ExerciseListPage() -> Element {
    let all_exercises = exercise_db::use_exercises();
    let custom_exercises = storage::use_custom_exercises();
    let mut search_query = use_signal(String::new);

    // Merge DB exercises and user-created exercises into a unified list
    let exercises = use_memo(move || {
        let query = search_query.read();
        let all = all_exercises.read();
        let custom = custom_exercises.read();
        let query_lower = query.to_lowercase();

        let mut results = Vec::new();

        // Add user-created exercises first
        for ex in custom.iter() {
            let matches = query_lower.is_empty() || ex.name.to_lowercase().contains(&query_lower);
            if matches {
                results.push(ex.clone());
            }
        }

        // Add DB exercises
        if query_lower.is_empty() {
            results.extend(all.iter().take(50).cloned());
        } else {
            results.extend(
                exercise_db::search_exercises(&all, &query)
                    .into_iter()
                    .take(50),
            );
        }

        results
    });

    let total = all_exercises.read().len() + custom_exercises.read().len();

    rsx! {
        main { class: "page-content container container--narrow",

            header {
                class: "page-header",
                h1 { class: "page-title", "Exercise Database" }
                p { class: "page-subtitle",
                    "Browse {total} exercises"
                }
            }

            div {
                class: "search-wrapper",
                input {
                    r#type: "text",
                    placeholder: "Search exercises, muscles, or categories...",
                    value: "{search_query}",
                    oninput: move |evt| search_query.set(evt.value()),
                    class: "search-input",
                }
            }

            section {
                class: "exercise-list",
                for exercise in exercises() {
                    ExerciseCard { key: "{exercise.id}", exercise }
                }
            }
        }
        BottomNav { active_tab: ActiveTab::Exercises }
    }
}
