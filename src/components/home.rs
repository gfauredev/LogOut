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
    use_hook(|| {
        is_loading.set(true);
        spawn(async move {
            let page = storage::load_completed_sessions_page(PAGE_SIZE, 0).await;
            let len = page.len();
            completed_sessions.set(page);
            sessions_loaded_offset.set(len);
            all_loaded.set(len < PAGE_SIZE);
            is_loading.set(false);
        });
    });
    let completed_session_ids = use_memo(move || {
        sessions
            .read()
            .iter()
            .filter(|s| !s.is_active())
            .map(|s| s.id.clone())
            .collect::<std::collections::HashSet<String>>()
    });
    use_effect(move || {
        let new_ids = completed_session_ids.read().clone();
        let viewed_ids: std::collections::HashSet<String> = completed_sessions
            .peek()
            .iter()
            .map(|s| s.id.clone())
            .collect();
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
                let mut new_cs = Vec::with_capacity(newly_completed.len() + cs.len());
                new_cs.extend(newly_completed);
                new_cs.extend(cs.drain(..));
                *cs = new_cs;
                cs.len()
            };
            sessions_loaded_offset.set(new_len);
        }
        let _ = new_ids;
    });
    let start_new_session = move |_| {
        let new_session = WorkoutSession::new();
        storage::save_session(new_session);
    };
    rsx! {
        if *has_active.read() {
            SessionView {}
        } else {
            header {
                h1 { tabindex: 0, {t!("app-title")} }
                p { tabindex: 0, {t!("app-subtitle")} }
            }
            if completed_sessions.read().is_empty() && !*is_loading.read() {
                p { {t!("no-sessions")} }
                p { {t!("start-first-workout")} }
            } else {
                main { class: "sessions",
                    for session in completed_sessions.read().iter() {
                        SessionCard {
                            key: "{session.id}",
                            session: session.clone(),
                            on_delete: move |id: String| {
                                let new_len = {
                                    let mut cs = completed_sessions.write();
                                    cs.retain(|s| s.id != id);
                                    cs.len()
                                };
                                sessions_loaded_offset.set(new_len);
                            },
                        }
                    }
                    if !*all_loaded.read() {
                        InfiniteScrollSentinel {
                            is_loading,
                            all_loaded,
                            sessions_loaded_offset,
                            completed_sessions,
                        }
                    }
                }
            }
            button {
                class: "icon more",
                onclick: start_new_session,
                title: t!("start-new-workout"),
                "+"
            }
        }
        BottomNav { active_tab: ActiveTab::Sessions }
    }
}
#[component]
fn SessionCard(session: WorkoutSession, on_delete: EventHandler<String>) -> Element {
    const MAX_VISIBLE: usize = 9;
    let mut show_delete_confirm = use_signal(|| false);
    let mut show_all_exercises = use_signal(|| false);
    let session_id = session.id.clone();
    let mut search_signal = use_context::<ExerciseSearchSignal>().0;
    let navigator = use_navigator();
    let duration = session.duration_seconds();
    let date_str = format_session_date(session.start_time);
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
                div {
                    label { "⏱️" }
                    time { "{format_time(duration)}" }
                }
                if !pending_ids.is_empty() {
                    button {
                        class: "edit",
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
                button {
                    class: "del",
                    onclick: move |_| show_delete_confirm.set(true),
                    title: "Delete session",
                    "🗑️"
                }
            }
            if !unique_exercises.is_empty() {
                ul {
                    for (_ , name , tag_class , tag_icon) in unique_exercises.iter().take(visible_count) {
                        li {
                            class: "{tag_class}",
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
                        li {
                            class: "more",
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
                dialog { open: true, onclick: move |evt| evt.stop_propagation(),
                    p { "Delete this session?" }
                    div {
                        button {
                            onclick: {
                                let id = session_id.clone();
                                move |_| {
                                    storage::delete_session(&id);
                                    on_delete.call(id.clone());
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
/// Sentinel element placed at the bottom of the session list.
///
/// On the web platform it uses the browser's `IntersectionObserver` API to
/// detect when the bottom of the list scrolls into the viewport and
/// transparently loads the next page of sessions.  The observer is properly
/// disconnected when the component unmounts so no JS callbacks are leaked.
///
/// On native platforms the component renders nothing (sessions are loaded via
/// SQL `LIMIT`/`OFFSET` on demand from the app's control flow).
#[component]
fn InfiniteScrollSentinel(
    is_loading: Signal<bool>,
    all_loaded: Signal<bool>,
    sessions_loaded_offset: Signal<usize>,
    completed_sessions: Signal<Vec<WorkoutSession>>,
) -> Element {
    #[cfg(target_arch = "wasm32")]
    {
        use std::rc::Rc;
        use wasm_bindgen::prelude::Closure;
        use wasm_bindgen::JsCast as _;
        let _observer = use_hook(move || {
            let callback: Closure<dyn FnMut(js_sys::Array)> =
                Closure::wrap(Box::new(move |entries: js_sys::Array| {
                    for entry in entries.iter() {
                        let entry: web_sys::IntersectionObserverEntry = entry.unchecked_into();
                        if entry.is_intersecting() {
                            if *is_loading.peek() || *all_loaded.peek() {
                                break;
                            }
                            is_loading.set(true);
                            let off = *sessions_loaded_offset.peek();
                            wasm_bindgen_futures::spawn_local(async move {
                                let next = crate::services::storage::load_completed_sessions_page(
                                    PAGE_SIZE, off,
                                )
                                .await;
                                let len = next.len();
                                completed_sessions.write().extend(next);
                                sessions_loaded_offset.set(off + len);
                                all_loaded.set(len < PAGE_SIZE);
                                is_loading.set(false);
                            });
                            break;
                        }
                    }
                }));
            let observer = web_sys::IntersectionObserver::new(callback.as_ref().unchecked_ref())
                .expect("IntersectionObserver::new should succeed");
            struct ObserverGuard {
                observer: web_sys::IntersectionObserver,
                #[allow(dead_code)]
                callback: Closure<dyn FnMut(js_sys::Array)>,
            }
            impl Drop for ObserverGuard {
                fn drop(&mut self) {
                    self.observer.disconnect();
                }
            }
            Rc::new(ObserverGuard { observer, callback })
        });
        return rsx! {
            div {
                class: "sentinel",
                onmounted: {
                    let guard = _observer.clone();
                    move |evt: Event<MountedData>| {
                        if let Some(element) = evt.downcast::<web_sys::Element>().cloned() {
                            guard.observer.observe(&element);
                        }
                    }
                },
            }
        };
    }
    #[cfg(not(target_arch = "wasm32"))]
    rsx! {}
}
