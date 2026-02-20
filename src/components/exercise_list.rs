use crate::components::{ActiveTab, BottomNav, ExerciseCard};
use crate::services::{exercise_db, storage};
use dioxus::prelude::*;

/// Number of exercises displayed per page.
const PAGE_SIZE: usize = 20;

#[component]
pub fn ExerciseListPage() -> Element {
    let all_exercises = exercise_db::use_exercises();
    let custom_exercises = storage::use_custom_exercises();
    let sessions = storage::use_sessions();
    let mut search_query = use_signal(String::new);
    let mut page = use_signal(|| 0usize);

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

    // Track the exercise currently being performed (if any) to pin it to the top
    let current_exercise_id = use_memo(move || {
        sessions
            .read()
            .iter()
            .find(|s| s.is_active())
            .and_then(|s| s.current_exercise_id.clone())
    });

    // Merge DB exercises and user-created exercises into a unified list.
    // Unified search applies to both custom and DB exercises (by name, muscle, category, etc.).
    let exercises = use_memo(move || {
        let query = search_query.read();
        let all = all_exercises.read();
        let custom = custom_exercises.read();
        let active_ids = active_session_ids();

        let mut results = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        if query.is_empty() {
            // Add all user-created exercises first (they have priority)
            for ex in custom.iter() {
                if seen_ids.insert(ex.id.clone()) {
                    results.push((ex.clone(), true));
                }
            }
            // Add all DB exercises (no hard limit – pagination handles display)
            for ex in all.iter() {
                if seen_ids.insert(ex.id.clone()) {
                    results.push((ex.clone(), false));
                }
            }
        } else {
            // Unified search: use search_exercises for both custom and DB exercises
            // so that muscle, category, equipment, etc. are all searchable.
            let custom_results = exercise_db::search_exercises(&custom, &query);
            for ex in custom_results {
                if seen_ids.insert(ex.id.clone()) {
                    results.push((ex, true));
                }
            }
            let db_results = exercise_db::search_exercises(&all, &query);
            for ex in db_results {
                if seen_ids.insert(ex.id.clone()) {
                    results.push((ex, false));
                }
            }
        }

        // Pin exercises: currently-performing first, then completed in the session, then rest
        let cur_id = current_exercise_id.read().clone();
        if !active_ids.is_empty() || cur_id.is_some() {
            results.sort_by_key(|(ex, _)| {
                let is_current = cur_id.as_deref() == Some(ex.id.as_str());
                let is_active = active_ids.contains(&ex.id);
                (!is_current, !is_active)
            });
        }

        results
    });

    // Current page items, annotated with whether instructions should be shown.
    let page_items = use_memo(move || {
        let active_ids = active_session_ids();
        let cur_id = current_exercise_id.read().clone();
        let page_start = page() * PAGE_SIZE;
        exercises
            .read()
            .iter()
            .skip(page_start)
            .take(PAGE_SIZE)
            .map(|(ex, is_custom)| {
                let show_instructions = active_ids.contains(&ex.id)
                    || cur_id.as_deref() == Some(ex.id.as_str());
                (ex.clone(), *is_custom, show_instructions)
            })
            .collect::<Vec<_>>()
    });

    let total = all_exercises.read().len();
    let total_results = exercises.read().len();
    let total_pages = total_results.div_ceil(PAGE_SIZE).max(1);

    rsx! {
        main { class: "page-content container container--narrow",

            header {
                class: "page-header",
                h1 { class: "page-title", "📚 Exercise Database" }
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
                    oninput: move |evt| {
                        search_query.set(evt.value());
                        page.set(0);
                    },
                    class: "search-input",
                }
            }

            section {
                class: "exercise-list",
                for (exercise, is_custom, show_instructions) in page_items() {
                    ExerciseCard {
                        key: "{exercise.id}",
                        exercise,
                        is_custom,
                        show_instructions_initial: show_instructions,
                    }
                }
            }

            if total_pages > 1 {
                div { class: "pagination",
                    button {
                        class: "btn btn--secondary",
                        disabled: page() == 0,
                        onclick: move |_| page.set(page() - 1),
                        "← Previous"
                    }
                    span { class: "pagination__info",
                        "Page {page() + 1} of {total_pages}"
                    }
                    button {
                        class: "btn btn--secondary",
                        disabled: (page() + 1) >= total_pages,
                        onclick: move |_| page.set(page() + 1),
                        "Next →"
                    }
                }
            }
        }
        BottomNav { active_tab: ActiveTab::Exercises }
    }
}
