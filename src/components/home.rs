use crate::components::{ActiveTab, BottomNav, SessionView};
use crate::models::{format_time, WorkoutSession};
use crate::services::storage;
use crate::utils::format_session_date;
use dioxus::prelude::*;

#[component]
pub fn HomePage() -> Element {
    let sessions = storage::use_sessions();

    let has_active = use_memo(move || sessions.read().iter().any(|s| s.is_active()));

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

    rsx! {
        main { if *has_active.read() {
                SessionView {}
            } else {
                section { class: "sessions-tab",
                    header { class: "sessions-tab__header",
                        h1 { class: "app-title", tabindex: 0, "💪 LogOut" }
                        p { class: "app-tagline", tabindex: 0, "Turn off your computer, Log your workOut" }
                    }
                    if completed_sessions().is_empty() {
                        div { class: "sessions-empty",
                            p { "No past sessions yet." }
                            p { "Tap + to start your first workout!" }
                        }
                    } else {
                        div { class: "sessions-list",
                            for session in completed_sessions() {
                                SessionCard { key: "{session.id}", session: session.clone() }
                            }
                        }
                    }
                }
            }
        }
        if !*has_active.read() {
            button {
                onclick: start_new_session,
                class: "new-session-button",
                title: "Start New Workout",
                "+"
            }
        }
        BottomNav { active_tab: ActiveTab::Sessions }
    }
}

#[component]
fn SessionCard(session: WorkoutSession) -> Element {
    let mut show_delete_confirm = use_signal(|| false);
    let mut show_all_exercises = use_signal(|| false);
    let session_id = session.id.clone();

    let duration = session
        .end_time
        .map(|end| end.saturating_sub(session.start_time))
        .unwrap_or(0);
    let date_str = format_session_date(session.start_time);

    // Collect unique exercise names (deduplicated by ID, preserving order)
    let unique_exercises: Vec<(String, String)> = {
        let mut seen = std::collections::HashSet::new();
        session
            .exercise_logs
            .iter()
            .filter_map(|log| {
                if seen.insert(log.exercise_id.clone()) {
                    Some((log.exercise_id.clone(), log.exercise_name.clone()))
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
    const MAX_VISIBLE: usize = 9;
    let total_unique = unique_exercises.len();
    let visible_count = if *show_all_exercises.read() {
        total_unique
    } else {
        total_unique.min(MAX_VISIBLE)
    };
    let hidden_count = total_unique.saturating_sub(visible_count);

    rsx! {
        article { class: "session-card",
            div { class: "session-card__top-line",
                time { class: "session-card__date", "{date_str}" }
                span { class: "session-card__stat", "⏱ {format_time(duration)}" }
                div { class: "session-card__actions",
                    if !pending_ids.is_empty() {
                        button {
                            onclick: {
                                let pending_ids = pending_ids.clone();
                                move |_| {
                                    let mut new_session = WorkoutSession::new();
                                    new_session.pending_exercise_ids = pending_ids.clone();
                                    storage::save_session(new_session);
                                }
                            },
                            class: "session-card__repeat-btn",
                            title: "Start a new session based on this one",
                            "🔄"
                        }
                    }
                    button {
                        onclick: move |_| show_delete_confirm.set(true),
                        class: "session-card__delete-btn",
                        title: "Delete session",
                        "🗑️"
                    }
                }
            }
            if !unique_exercises.is_empty() {
                div { class: "session-card__exercises",
                    for (_, name) in unique_exercises.iter().take(visible_count) {
                        span { class: "session-card__exercise-name", "{name}" }
                    }
                    if hidden_count > 0 {
                        button {
                            class: "session-card__more",
                            onclick: move |_| show_all_exercises.set(true),
                            "+{hidden_count} more"
                        }
                    }
                }
            }

            // Delete confirmation modal with backdrop
            if *show_delete_confirm.read() {
                div {
                    class: "modal-backdrop",
                    onclick: move |_| show_delete_confirm.set(false),
                }
                div {
                    class: "delete-modal",
                    onclick: move |evt| evt.stop_propagation(),
                    p { "Delete this session?" }
                    div { class: "delete-modal__buttons",
                        button {
                            onclick: {
                                let id = session_id.clone();
                                move |_| {
                                    storage::delete_session(&id);
                                    show_delete_confirm.set(false);
                                }
                            },
                            class: "btn btn--danger",
                            "Delete"
                        }
                        button {
                            onclick: move |_| show_delete_confirm.set(false),
                            class: "btn--cancel",
                            "Cancel"
                        }
                    }
                }
            }
        }
    }
}
