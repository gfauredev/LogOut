use crate::components::{ActiveTab, BottomNav, ExerciseCard};
use crate::services::{exercise_db, storage};
use crate::{ExerciseSearchSignal, Route};
use dioxus::prelude::*;

/// Number of exercises loaded per scroll increment.
const PAGE_SIZE: usize = 20;

#[component]
pub fn ExerciseListPage() -> Element {
    let all_exercises = exercise_db::use_exercises();
    let custom_exercises = storage::use_custom_exercises();
    let sessions = storage::use_sessions();
    let mut search_query = use_signal(String::new);
    let mut visible_count = use_signal(|| PAGE_SIZE);

    // If another page set a search query via the global signal, consume it.
    let mut search_signal = use_context::<ExerciseSearchSignal>().0;
    use_effect(move || {
        let q = search_signal.read().clone();
        if let Some(q) = q {
            search_query.set(q);
            search_signal.set(None);
        }
    });

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
            // Add all DB exercises (no hard limit – scroll pagination handles display)
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
                    results.push((ex.clone(), true));
                }
            }
            let db_results = exercise_db::search_exercises(&all, &query);
            for ex in db_results {
                if seen_ids.insert(ex.id.clone()) {
                    results.push((ex.clone(), false));
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

    // Set up scroll-based auto-pagination via a web-sys scroll event listener.
    // Using `use_hook` ensures the listener is registered once per component mount.
    // `window.onscroll` assignment (rather than addEventListener) avoids accumulating
    // duplicate listeners across component remounts.
    #[cfg(target_arch = "wasm32")]
    use_hook(move || {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;

        let closure = Closure::<dyn FnMut()>::new(move || {
            let Some(window) = web_sys::window() else {
                return;
            };
            let Some(doc) = window.document() else { return };
            let Some(el) = doc.document_element() else {
                return;
            };

            let scroll_top = window.scroll_y().unwrap_or(0.0);
            let client_height = el.client_height() as f64;
            let scroll_height = el.scroll_height() as f64;

            if scroll_top + client_height >= scroll_height - 300.0 {
                let cur = *visible_count.peek();
                let total = exercises.peek().len();
                if cur < total {
                    visible_count.set(cur + PAGE_SIZE);
                }
            }
        });

        if let Some(window) = web_sys::window() {
            let js_fn: &js_sys::Function = closure.as_ref().unchecked_ref();
            let _ = js_sys::Reflect::set(&window, &"onscroll".into(), js_fn);
        }
        // Leak the closure so it lives for the page lifetime.
        closure.forget();
    });

    // Visible items, annotated with whether instructions should be shown.
    let visible_items = use_memo(move || {
        let active_ids = active_session_ids();
        let cur_id = current_exercise_id.read().clone();
        let count = *visible_count.read();
        exercises
            .read()
            .iter()
            .take(count)
            .map(|(ex, is_custom)| {
                let show_instructions =
                    active_ids.contains(&ex.id) || cur_id.as_deref() == Some(ex.id.as_str());
                (ex.clone(), *is_custom, show_instructions)
            })
            .collect::<Vec<_>>()
    });

    let total = all_exercises.read().len();

    rsx! {
        header {
            h1 { tabindex: 0, "📚 Exercise Database" }
            p { "Browse {total} exercises" }
        }
        main { class: "exercises",
            div { class: "form-group",
                div { class: "search-with-add",
                    input {
                        r#type: "text",
                        placeholder: "Search exercises, muscles, or categories...",
                        value: "{search_query}",
                        oninput: move |evt| {
                            search_query.set(evt.value());
                            visible_count.set(PAGE_SIZE);
                        },
                        class: "search-input",
                    }
                    Link {
                        to: Route::AddCustomExercisePage {},
                        class: "add-exercise-btn",
                        title: "Add Custom Exercise",
                        "+"
                    }
                }
            }
            section {
                class: "exercise-list",
                for (exercise, is_custom, show_instructions) in visible_items() {
                    ExerciseCard {
                        key: "{exercise.id}",
                        exercise,
                        is_custom,
                        show_instructions_initial: show_instructions,
                    }
                }
            }
        }
        BottomNav { active_tab: ActiveTab::Exercises }
    }
}
