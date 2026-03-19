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
    #[cfg_attr(not(target_arch = "wasm32"), allow(unused_mut))]
    let mut visible_count = use_signal(|| PAGE_SIZE);

    let has_active = use_memo(move || sessions.read().iter().any(WorkoutSession::is_active));

    let start_new_session = move |_| {
        let new_session = WorkoutSession::new();
        storage::save_session(new_session);
    };

    let completed_sessions = use_memo(move || {
        let mut completed: Vec<WorkoutSession> = sessions
            .read()
            .iter()
            .filter(|s| !s.is_active())
            .cloned()
            .collect();
        // antichronological order
        completed.sort_by(|a, b| b.start_time.cmp(&a.start_time));
        completed
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
                    if (scrollHeight > 0 && scrollTop + clientHeight >= scrollHeight - 300) {{
                        dioxus.send(true);
                    }}
                }};
                window.addEventListener('scroll', handler);
                // Dioxus eval will automatically clean up when the future is dropped
                // but we can also handle it if we want.
            }})()
            "
        );
        spawn(async move {
            let mut eval = dioxus::prelude::document::eval(&js);
            while eval.recv::<bool>().await.is_ok() {
                let cur = *visible_count.peek();
                let total = completed_sessions.peek().len();
                if cur < total {
                    visible_count.set(cur + PAGE_SIZE);
                }
            }
        });
    });

    rsx! {
        if *has_active.read() { SessionView {} } else {
            header {
                h1 { tabindex: 0, {t!("app-title")} }
                p { tabindex: 0, {t!("app-subtitle")} }
            }
            if completed_sessions().is_empty() {
                p { {t!("no-sessions")} }
                p { {t!("start-first-workout")} }
            } else {
                main { class: "sessions",
                    for session in completed_sessions().into_iter().take(*visible_count.read()) {
                        SessionCard { key: "{session.id}", session: session.clone() }
                    }
                }
            }
            button { class: "icon add",
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

    // Collect exercise IDs in order (including repeats) for the repeat action.
    // Each exercise is included as many times as it was performed so that the
    // pre-added queue in the new session mirrors the original session exactly.
    let pending_ids: Vec<String> = session
        .exercise_logs
        .iter()
        .map(|log| log.exercise_id.clone())
        .collect();

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
                        "🔄"
                    }
                }
                button { class: "no",
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
                            class: "no label",
                            "🗑️ Delete"
                        }
                        button {
                            onclick: move |_| show_delete_confirm.set(false),
                            class: "yes", // Safer
                            "❌"
                        }
                    }
                }
            }
        }
    }
}
