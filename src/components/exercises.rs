use crate::components::{ActiveTab, BottomNav, ExerciseCard};
use crate::models::Exercise;
use crate::services::{exercise_db, storage};
use crate::services::exercise_db::{SearchFilter, detect_filter_suggestions, exercise_matches_filters};
use crate::{DbI18nSignal, ExerciseSearchSignal, Route};
use dioxus::prelude::*;
use dioxus_i18n::t;

/// Maximum number of simultaneously active hard filters.
const MAX_FILTERS: usize = 4;
/// Number of exercises loaded per scroll increment.
const PAGE_SIZE: usize = 20;
/// Pixels from the bottom of the page at which an auto-pagination is triggered.
const SCROLL_THRESHOLD_PX: u32 = 300;

#[component]
pub fn Exercises() -> Element {
    let all_exercises = exercise_db::use_exercises();
    let custom_exercises = storage::use_custom_exercises();
    let sessions = storage::use_sessions();
    let db_i18n_sig = use_context::<DbI18nSignal>().0;
    let mut search_query = use_signal(String::new);
    let mut visible_count = use_signal(|| PAGE_SIZE);
    let mut active_filters: Signal<Vec<SearchFilter>> = use_signal(Vec::new);

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

    // Filter suggestions derived from the current search query.
    // A suggestion is only shown if it is not already an active filter.
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

    // Merge DB exercises and user-created exercises into a unified list.
    // Unified search applies to both custom and DB exercises (by name, muscle, category, etc.).
    let exercises = use_memo(move || {
        let query = search_query.read();
        let all = all_exercises.read();
        let custom = custom_exercises.read();
        let active_ids = active_session_ids();
        let filters = active_filters.read();

        // Pre-filter by active hard filters (if any).
        let all_filtered: Vec<Exercise>;
        let custom_filtered: Vec<Exercise>;
        let all_slice: &[Exercise];
        let custom_slice: &[Exercise];
        if filters.is_empty() {
            all_slice = &all;
            custom_slice = &custom;
        } else {
            all_filtered = all
                .iter()
                .filter(|e| exercise_matches_filters(e, &filters))
                .cloned()
                .collect();
            custom_filtered = custom
                .iter()
                .filter(|e| exercise_matches_filters(e, &filters))
                .cloned()
                .collect();
            all_slice = &all_filtered;
            custom_slice = &custom_filtered;
        }

        let mut results = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        if query.is_empty() {
            // No text query – just filters (or none at all).
            // Add all user-created exercises first (they have priority)
            for ex in custom_slice.iter() {
                if seen_ids.insert(ex.id.clone()) {
                    results.push((ex.clone(), true));
                }
            }
            // Add all DB exercises (no hard limit – scroll pagination handles display)
            for ex in all_slice.iter() {
                if seen_ids.insert(ex.id.clone()) {
                    results.push((ex.clone(), false));
                }
            }
        } else {
            // Unified search: use search_exercises for both custom and DB exercises
            // so that muscle, category, equipment, etc. are all searchable.
            let db_i18n = db_i18n_sig.read();
            let db_i18n_ref = Some(&*db_i18n).filter(|m| !m.is_empty());
            let custom_results = exercise_db::search_exercises(custom_slice, &query, db_i18n_ref);
            for ex in custom_results {
                if seen_ids.insert(ex.id.clone()) {
                    results.push((ex.clone(), true));
                }
            }
            let db_results = exercise_db::search_exercises(all_slice, &query, db_i18n_ref);
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

    // Set up scroll-based auto-pagination via document::eval (cross-platform).
    // Injects a scroll listener that sends a message whenever the user is near
    // the bottom; Rust receives it and increments visible_count.
    use_hook(move || {
        let js = format!(
            r"
            (function() {{
                const handler = function() {{
                    var el = document.documentElement;
                    var scrollTop = window.scrollY || el.scrollTop || 0;
                    var clientHeight = el.clientHeight || window.innerHeight || 0;
                    var scrollHeight = el.scrollHeight || 0;
                    if (scrollHeight > 0 && scrollTop + clientHeight >= scrollHeight - {SCROLL_THRESHOLD_PX}) {{
                        dioxus.send(true);
                    }}
                }};
                window.onscroll = handler;
            }})()
            "
        );
        spawn(async move {
            let mut eval = dioxus::prelude::document::eval(&js);
            while eval.recv::<bool>().await.is_ok() {
                let cur = *visible_count.peek();
                let total = exercises.peek().len();
                if cur < total {
                    visible_count.set(cur + PAGE_SIZE);
                }
            }
        });
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
            h1 { tabindex: 0, "📚 Exercises" }
            p { {t!("browse-exercises", count: { total.to_string() })} }
            div { class: "inputs",
                input { r#type: "text",
                    placeholder: t!("search-placeholder"),
                    value: "{search_query}",
                    oninput: move |evt| {
                        search_query.set(evt.value());
                        visible_count.set(PAGE_SIZE);
                    }
                }
                Link { class: "add",
                    to: Route::AddExercise {},
                    title: t!("add-exercise"),
                    "+"
                }
            }
            // Active filter chips – click to remove the filter.
            if !active_filters.read().is_empty() {
                div { class: "filter-chips",
                    for (i, filter) in active_filters.read().iter().enumerate() {
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
            // Filter suggestion buttons – shown when the search term matches
            // an attribute value.  Clicking activates the filter, clears
            // the search input, and allows the user to search within the
            // filtered results.
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
            for (exercise, is_custom, show_instructions) in visible_items() {
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
