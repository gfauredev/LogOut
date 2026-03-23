use crate::components::{ActiveTab, BottomNav, ExerciseCard};
use crate::models::Exercise;
use crate::services::exercise_db::{
    detect_filter_suggestions, exercise_matches_filters, SearchFilter,
};
use crate::services::{exercise_db, storage};
use crate::{ExerciseSearchSignal, Route};
use dioxus::prelude::*;
use dioxus_i18n::t;
use std::sync::Arc;
/// Maximum number of simultaneously active hard filters.
const MAX_FILTERS: usize = 4;
/// Number of exercises loaded per scroll increment.
const PAGE_SIZE: usize = 20;
/// Pixels from the bottom of the page at which an auto-pagination is triggered.
const SCROLL_THRESHOLD_PX: u32 = 300;
/// Debounce delay in milliseconds before re-running the expensive exercise filter.
const SEARCH_DEBOUNCE_MS: u32 = 200;
#[component]
pub fn Exercises() -> Element {
    let all_exercises = exercise_db::use_exercises();
    let custom_exercises = storage::use_custom_exercises();
    let sessions = storage::use_sessions();
    // Raw query updated on every keystroke (drives the input value and filter-suggestion chips).
    let mut search_query = use_signal(String::new);
    // Debounced query – only updated `SEARCH_DEBOUNCE_MS` after the user stops typing.
    // Used for the expensive exercise-scoring memo so typing stays responsive.
    let mut debounced_query = use_signal(String::new);
    let mut debounce_gen = use_signal(|| 0u64);
    let mut visible_count = use_signal(|| PAGE_SIZE);
    let mut active_filters: Signal<Vec<SearchFilter>> = use_signal(Vec::new);
    let mut search_signal = use_context::<ExerciseSearchSignal>().0;
    use_effect(move || {
        let q = search_signal.read().clone();
        if let Some(q) = q {
            search_query.set(q.clone());
            debounced_query.set(q);
            search_signal.set(None);
        }
    });
    // Debounce: update `debounced_query` only after SEARCH_DEBOUNCE_MS of inactivity.
    use_effect(move || {
        let q = search_query.read().clone();
        let cur_gen = {
            let mut g = debounce_gen.write();
            *g += 1;
            *g
        };
        spawn(async move {
            #[cfg(target_arch = "wasm32")]
            gloo_timers::future::TimeoutFuture::new(SEARCH_DEBOUNCE_MS).await;
            #[cfg(not(target_arch = "wasm32"))]
            tokio::time::sleep(std::time::Duration::from_millis(u64::from(
                SEARCH_DEBOUNCE_MS,
            )))
            .await;
            if *debounce_gen.peek() == cur_gen {
                debounced_query.set(q);
                visible_count.set(PAGE_SIZE);
            }
        });
    });
    let active_session_ids = use_memo(move || {
        let mut ids = std::collections::HashSet::new();
        if let Some(session) = sessions.read().iter().find(|s| s.is_active()) {
            for log in &session.exercise_logs {
                ids.insert(log.exercise_id.clone());
            }
        }
        ids
    });
    let current_exercise_id = use_memo(move || {
        sessions
            .read()
            .iter()
            .find(|s| s.is_active())
            .and_then(|s| s.current_exercise_id.clone())
    });
    let filter_suggestions = use_memo(move || {
        let query = search_query.read();
        if query.is_empty() {
            return Vec::new();
        }
        let current = active_filters.read();
        detect_filter_suggestions(&query)
            .into_iter()
            .filter(|s| !current.contains(s))
            .collect::<Vec<_>>()
    });
    // Step 1: filter the full list by active filter chips (only re-runs when chips change).
    let filter_pool = use_memo(move || {
        let all = all_exercises.read();
        let custom = custom_exercises.read();
        let filters = active_filters.read();
        if filters.is_empty() {
            return (all.clone(), custom.clone());
        }
        let filtered_all: Vec<Arc<Exercise>> = all
            .iter()
            .filter(|e| exercise_matches_filters(e.as_ref(), &filters))
            .cloned()
            .collect();
        let filtered_custom: Vec<Arc<Exercise>> = custom
            .iter()
            .filter(|e| exercise_matches_filters(e.as_ref(), &filters))
            .cloned()
            .collect();
        (filtered_all, filtered_custom)
    });
    // Step 2: text-search (or list) within the pre-filtered pool (re-runs on debounced keystrokes).
    let exercises = use_memo(move || {
        let query = debounced_query.read();
        let (all_pool, custom_pool) = filter_pool();
        let active_ids = active_session_ids();
        let mut results = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();
        if query.is_empty() {
            for ex in &custom_pool {
                if seen_ids.insert(ex.id.clone()) {
                    results.push((ex.clone(), true));
                }
            }
            for ex in &all_pool {
                if seen_ids.insert(ex.id.clone()) {
                    results.push((ex.clone(), false));
                }
            }
        } else {
            let custom_results = exercise_db::search_exercises(&custom_pool, &query);
            for ex in custom_results {
                if seen_ids.insert(ex.id.clone()) {
                    results.push((ex.clone(), true));
                }
            }
            let db_results = exercise_db::search_exercises(&all_pool, &query);
            for ex in db_results {
                if seen_ids.insert(ex.id.clone()) {
                    results.push((ex.clone(), false));
                }
            }
        }
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
    #[cfg(target_arch = "wasm32")]
    let _scroll_guard = use_hook(move || {
        use std::rc::Rc;
        use wasm_bindgen::prelude::Closure;
        use wasm_bindgen::JsCast as _;
        let closure: Closure<dyn FnMut()> = Closure::wrap(Box::new(move || {
            let Some(window) = web_sys::window() else {
                return;
            };
            let Some(document) = window.document() else {
                return;
            };
            let Some(el) = document.document_element() else {
                return;
            };
            let scroll_top = window.scroll_y().unwrap_or(0.0);
            let client_height = f64::from(el.client_height());
            let scroll_height = f64::from(el.scroll_height());
            if scroll_height > 0.0
                && scroll_top + client_height
                    >= scroll_height - f64::from(SCROLL_THRESHOLD_PX)
            {
                let cur = *visible_count.peek();
                let total = exercises.peek().len();
                if cur < total {
                    visible_count.set(cur + PAGE_SIZE);
                }
            }
        }));
        let func: js_sys::Function =
            closure.as_ref().unchecked_ref::<js_sys::Function>().clone();
        if let Some(window) = web_sys::window() {
            let _ = window.add_event_listener_with_callback("scroll", &func);
        }
        /// Drop guard that removes the scroll event listener when the `Exercises`
        /// component unmounts, preventing a JS interop memory leak.
        struct ScrollGuard {
            /// Keeps the underlying JS function alive until the listener is removed.
            #[allow(dead_code)]
            closure: Closure<dyn FnMut()>,
            func: js_sys::Function,
        }
        impl Drop for ScrollGuard {
            fn drop(&mut self) {
                if let Some(window) = web_sys::window() {
                    let _ =
                        window.remove_event_listener_with_callback("scroll", &self.func);
                }
            }
        }
        Rc::new(ScrollGuard { closure, func })
    });
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
            h1 { tabindex: 0, "📚 Exercises" }
            p { {t!("browse-exercises", count : { total.to_string() })} }
            div { class: "inputs",
                input {
                    r#type: "text",
                    placeholder: t!("search-placeholder"),
                    value: "{search_query}",
                    oninput: move |evt| {
                        search_query.set(evt.value());
                    },
                }
                Link {
                    class: "more",
                    to: Route::AddExercise {},
                    title: t!("add-exercise"),
                    "+"
                }
            }
            if !active_filters.read().is_empty() {
                div { class: "filter-chips",
                    for (i , filter) in active_filters.read().iter().enumerate() {
                        button {
                            class: "filter-chip active",
                            title: t!("filter-remove"),
                            onclick: move |_| {
                                let mut filters = active_filters.write();
                                if i < filters.len() {
                                    filters.remove(i);
                                }
                                visible_count.set(PAGE_SIZE);
                            },
                            "{filter.label()} ✕"
                        }
                    }
                }
            }
            if !filter_suggestions.read().is_empty() {
                div { class: "filter-chips",
                    for suggestion in filter_suggestions.read().iter() {
                        if active_filters.read().len() < MAX_FILTERS {
                            button {
                                class: "filter-chip suggestion",
                                title: t!("filter-add"),
                                onclick: {
                                    let suggestion = suggestion.clone();
                                    move |_| {
                                        active_filters.write().push(suggestion.clone());
                                        search_query.set(String::new());
                                        debounced_query.set(String::new());
                                        visible_count.set(PAGE_SIZE);
                                    }
                                },
                                "🔍 {suggestion.label()}"
                            }
                        }
                    }
                }
            }
        }
        main { class: "exercises",
            for (exercise , is_custom , show_instructions) in visible_items() {
                ExerciseCard {
                    key: "{exercise.id}",
                    exercise,
                    is_custom,
                    show_instructions_initial: show_instructions,
                }
            }
        }
        BottomNav { active_tab: ActiveTab::Exercises }
    }
}
