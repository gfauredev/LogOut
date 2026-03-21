use crate::components::{ActiveTab, BottomNav, SessionView};
use crate::models::{format_time, WorkoutSession};
use crate::services::storage;
use crate::utils::format_session_date;
use crate::{ExerciseSearchSignal, Route};
use dioxus::prelude::*;
use dioxus_i18n::t;

/// Number of sessions loaded per scroll increment
const PAGE_SIZE: usize = 20;

#[component]
pub fn Home() -> Element {
    let sessions = storage::use_sessions();
    let mut completed_sessions = use_signal(Vec::<WorkoutSession>::new);
    let mut sessions_loaded_offset = use_signal(|| 0usize);
    let mut all_loaded = use_signal(|| false);
    let mut is_loading = use_signal(|| false);

    let has_active = use_memo(move || sessions.read().iter().any(WorkoutSession::is_active));

    // Track the set of IDs of completed sessions so we can detect whether a
    // session was added or deleted without watching every field of every session
    // (which would cause the paginated list to reset on every active-session
    // heartbeat update).
    let completed_session_ids = use_memo(move || {
        sessions
            .read()
            .iter()
            .filter(|s| !s.is_active())
            .map(|s| s.id.clone())
            .collect::<std::collections::HashSet<String>>()
    });

    // Keep the paginated completed_sessions view in sync with the sessions
    // signal without resetting the scroll position on every change.
    //
    // Three cases are handled:
    // 1. Initial load (completed_sessions is empty) → load the first page.
    // 2. Deletion → remove the deleted item directly from the local vector.
    // 3. New completion → prepend the newly-completed session(s) to the top.
    use_effect(move || {
        let new_ids = completed_session_ids.read().clone();

        // Peek to avoid creating a reactive dependency on completed_sessions,
        // which would create a feedback loop.
        let viewed_ids: std::collections::HashSet<String> = completed_sessions
            .peek()
            .iter()
            .map(|s| s.id.clone())
            .collect();

        // ── Case 1: initial load ──────────────────────────────────────────
        if viewed_ids.is_empty() {
            if !new_ids.is_empty() {
                sessions_loaded_offset.set(0);
                all_loaded.set(false);
                is_loading.set(true);
                spawn(async move {
                    let page = storage::load_completed_sessions_page(PAGE_SIZE, 0).await;
                    let len = page.len();
                    completed_sessions.set(page);
                    sessions_loaded_offset.set(len);
                    all_loaded.set(len < PAGE_SIZE);
                    is_loading.set(false);
                });
            }
            return;
        }

        // ── Case 2: deletion ──────────────────────────────────────────────
        let removed: Vec<String> = viewed_ids
            .difference(&new_ids)
            .cloned()
            .collect();
        if !removed.is_empty() {
            let new_len = {
                let mut cs = completed_sessions.write();
                cs.retain(|s| !removed.contains(&s.id));
                cs.len()
            };
            // Keep offset consistent with what we have loaded in memory.
            sessions_loaded_offset.set(new_len);
            return;
        }

        // ── Case 3: new completion ────────────────────────────────────────
        // Peek at sessions to get the full data for newly-completed entries
        // without creating an additional reactive subscription.
        let mut newly_completed: Vec<WorkoutSession> = sessions
            .peek()
            .iter()
            .filter(|s| !s.is_active() && !viewed_ids.contains(&s.id))
            .cloned()
            .collect();
        if !newly_completed.is_empty() {
            newly_completed.sort_by(|a, b| b.start_time.cmp(&a.start_time));
            let new_len = {
                let mut cs = completed_sessions.write();
                // Build a new vec with correct capacity to avoid double-allocation:
                // newly_completed first (newest), then existing paginated entries.
                let mut new_cs = Vec::with_capacity(newly_completed.len() + cs.len());
                new_cs.extend(newly_completed);
                new_cs.extend(cs.drain(..));
                *cs = new_cs;
                cs.len()
            };
            sessions_loaded_offset.set(new_len);
        }
    });

    let start_new_session = move |_| {
        let new_session = WorkoutSession::new();
        storage::save_session(new_session);
    };

    // Set up scroll-based auto-pagination on wasm32 via a web-sys `Closure`.
    //
    // Using `web-sys` instead of `document::eval` lets us hold a Rust reference
    // to the handler function.  The returned `ScrollGuard` is stored in the
    // component's hook state and dropped when the `Home` component unmounts,
    // which calls `window.removeEventListener` to prevent a memory leak.
    #[cfg(target_arch = "wasm32")]
    let _scroll_guard = use_hook(move || {
        use wasm_bindgen::prelude::Closure;
        use wasm_bindgen::JsCast as _;

        let closure: Closure<dyn Fn()> = Closure::wrap(Box::new(move || {
            if *is_loading.peek() || *all_loaded.peek() {
                return;
            }
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
            if scroll_height > 0.0 && scroll_top + client_height >= scroll_height - 300.0 {
                is_loading.set(true);
                let off = *sessions_loaded_offset.peek();
                wasm_bindgen_futures::spawn_local(async move {
                    let next =
                        crate::services::storage::load_completed_sessions_page(PAGE_SIZE, off)
                            .await;
                    let len = next.len();
                    completed_sessions.write().extend(next);
                    sessions_loaded_offset.set(off + len);
                    all_loaded.set(len < PAGE_SIZE);
                    is_loading.set(false);
                });
            }
        }));

        let func: js_sys::Function = closure.as_ref().unchecked_ref::<js_sys::Function>().clone();
        if let Some(window) = web_sys::window() {
            let _ = window.add_event_listener_with_callback("scroll", &func);
        }

        /// Drop guard that removes the scroll event listener when the `Home`
        /// component unmounts, preventing a JS interop memory leak.
        struct ScrollGuard {
            /// Keeps the underlying JS function alive until the listener is removed.
            closure: Closure<dyn Fn()>,
            func: js_sys::Function,
        }
        impl Drop for ScrollGuard {
            fn drop(&mut self) {
                if let Some(window) = web_sys::window() {
                    let _ = window.remove_event_listener_with_callback("scroll", &self.func);
                }
            }
        }

        ScrollGuard { closure, func }
    });

    rsx! {
        if *has_active.read() { SessionView {} } else {
            header {
                h1 { tabindex: 0, {t!("app-title")} }
                p { tabindex: 0, {t!("app-subtitle")} }
            }
            if completed_sessions.read().is_empty() {
                p { {t!("no-sessions")} }
                p { {t!("start-first-workout")} }
            } else {
                main { class: "sessions",
                    for session in completed_sessions.read().iter() {
                        SessionCard { key: "{session.id}", session: session.clone() }
                    }
                }
            }
            button { class: "icon more",
                onclick: start_new_session,
                title: t!("start-new-workout"),
                "+"
            }
        }
        BottomNav { active_tab: ActiveTab::Sessions }
    }
}

#[component]
fn SessionCard(session: WorkoutSession) -> Element {
    const MAX_VISIBLE: usize = 9;
    let mut show_delete_confirm = use_signal(|| false);
    let mut show_all_exercises = use_signal(|| false);
    let session_id = session.id.clone();
    let mut search_signal = use_context::<ExerciseSearchSignal>().0;
    let navigator = use_navigator();

    let duration = session.duration_seconds();
    let date_str = format_session_date(session.start_time);

    // Collect unique exercise names (deduplicated by ID, preserving order)
    // Each entry also carries the type-tag CSS class and icon for visual styling.
    let unique_exercises: Vec<(String, String, &'static str, &'static str)> = {
        let mut seen = std::collections::HashSet::new();
        session
            .exercise_logs
            .iter()
            .filter_map(|log| {
                if seen.insert(log.exercise_id.clone()) {
                    let (tag_class, tag_icon) = log.type_tag();
                    Some((
                        log.exercise_id.clone(),
                        log.exercise_name.clone(),
                        tag_class,
                        tag_icon,
                    ))
                } else {
                    None
                }
            })
            .collect()
    };

    // Collect unique exercise IDs in order (deduplicated, preserving first occurrence)
    // for the repeat action so each exercise appears only once in the pre-added queue.
    let pending_ids: Vec<String> = {
        let mut seen = std::collections::HashSet::new();
        session
            .exercise_logs
            .iter()
            .filter_map(|log| {
                if seen.insert(log.exercise_id.clone()) {
                    Some(log.exercise_id.clone())
                } else {
                    None
                }
            })
            .collect()
    };

    // Up to 9 tags visible initially (~3 lines of 3 tags each)
    let total_unique = unique_exercises.len();
    let visible_count = if *show_all_exercises.read() {
        total_unique
    } else {
        total_unique.min(MAX_VISIBLE)
    };
    let hidden_count = total_unique.saturating_sub(visible_count);

    rsx! {
        article {
            header {
                time { "{date_str}" }
                div { label { "⏱️" } time { "{format_time(duration)}" } }
                if !pending_ids.is_empty() {
                    button { class: "edit",
                        onclick: {
                            let pending_ids = pending_ids.clone();
                            move |_| {
                                        let mut new_session = WorkoutSession::new();
                                        new_session.pending_exercise_ids.clone_from(&pending_ids);

                                storage::save_session(new_session);
                            }
                        },
                        title: "Start a new session based on this one",
                        "🔁"
                    }
                }
                button { class: "del",
                    onclick: move |_| show_delete_confirm.set(true),
                    title: "Delete session",
                    "🗑️"
                }
            }
            if !unique_exercises.is_empty() {
                ul {
                    for (_, name, tag_class, tag_icon) in unique_exercises.iter().take(visible_count) {
                        li { class: "{tag_class}",
                            onclick: {
                                let name = name.clone();
                                move |_| {
                                    search_signal.set(Some(name.clone()));
                                    navigator.push(Route::Exercises {});
                                }
                            },
                            "{tag_icon} {name}"
                        }
                    }
                    if hidden_count > 0 {
                        li { class: "more",
                            onclick: move |_| show_all_exercises.set(true),
                            "+{hidden_count} more"
                        }
                    }
                }
            }
            if *show_delete_confirm.read() {
                div {
                    class: "backdrop",
                    onclick: move |_| show_delete_confirm.set(false),
                }
                dialog {
                    open: true,
                    onclick: move |evt| evt.stop_propagation(),
                    p { "Delete this session?" }
                    div {
                        button {
                            onclick: {
                                let id = session_id.clone();
                                move |_| {
                                    storage::delete_session(&id);
                                    show_delete_confirm.set(false);
                                }
                            },
                            class: "del label",
                            "🗑️ Delete"
                        }
                        button {
                            onclick: move |_| show_delete_confirm.set(false),
                            class: "back label",
                            "❌ Cancel"
                        }
                    }
                }
            }
        }
    }
}
