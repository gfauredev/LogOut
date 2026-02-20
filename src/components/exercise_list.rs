use crate::components::{ActiveTab, BottomNav, ExerciseCard};
use crate::services::{exercise_db, storage};
use dioxus::prelude::*;

#[component]
pub fn ExerciseListPage() -> Element {
    let all_exercises = exercise_db::use_exercises();
    let custom_exercises = storage::use_custom_exercises();
    let sessions = storage::use_sessions();
    let mut search_query = use_signal(String::new);

    // Collect exercise IDs from the active session (if any)
    let active_session_ids = use_memo(move || {
        let mut ids = std::collections::HashSet::new();
        if let Some(session) = sessions.read().iter().find(|s| s.is_active()) {
            for log in &session.exercise_logs {
                ids.insert(log.exercise_id.clone());
            }
        }
        ids
    });

    // Merge DB exercises and user-created exercises into a unified list
    let exercises = use_memo(move || {
        let query = search_query.read();
        let all = all_exercises.read();
        let custom = custom_exercises.read();
        let query_lower = query.to_lowercase();
        let active_ids = active_session_ids();

        let mut results = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        // Add user-created exercises first (they have priority)
        for ex in custom.iter() {
            let matches = query_lower.is_empty() || ex.name.to_lowercase().contains(&query_lower);
            if matches && seen_ids.insert(ex.id.clone()) {
                results.push((ex.clone(), true));
            }
        }

        // Add DB exercises, skipping duplicates
        if query_lower.is_empty() {
            for ex in all.iter().take(50) {
                if seen_ids.insert(ex.id.clone()) {
                    results.push((ex.clone(), false));
                }
            }
        } else {
            for ex in exercise_db::search_exercises(&all, &query)
                .into_iter()
                .take(50)
            {
                if seen_ids.insert(ex.id.clone()) {
                    results.push((ex, false));
                }
            }
        }

        // Pin exercises from the active session to the top
        if !active_ids.is_empty() {
            results.sort_by_key(|(ex, _)| !active_ids.contains(&ex.id));
        }

        results
    });

    let total = all_exercises.read().len();

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
                for (exercise, is_custom) in exercises() {
                    ExerciseCard { key: "{exercise.id}", exercise, is_custom }
                }
            }
        }
        BottomNav { active_tab: ActiveTab::Exercises }
    }
}
