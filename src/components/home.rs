use dioxus::prelude::*;
use crate::models::{WorkoutSession, format_time};
use crate::services::storage;
use crate::components::{SessionView, BottomNav, ActiveTab};
use crate::utils::format_session_date;

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
        div { class: "app-container",
            main { class: "app-content",
                if *has_active.read() {
                    SessionView {}
                } else {
                    section { class: "sessions-tab",
                        header { class: "sessions-tab__header",
                            h1 { class: "app-title", "💪 LogOut" }
                            p { class: "app-tagline", "Turn off your computer, Log your workOut" }
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
                    class: "fab",
                    title: "Start New Workout",
                    "+"
                }
            }
            BottomNav { active_tab: ActiveTab::Sessions }
        }
    }
}

#[component]
fn SessionCard(session: WorkoutSession) -> Element {
    let mut show_delete_confirm = use_signal(|| false);
    let session_id = session.id.clone();

    let duration = session
        .end_time
        .map(|end| end.saturating_sub(session.start_time))
        .unwrap_or(0);
    let exercise_count = session.exercise_logs.len();
    let date_str = format_session_date(session.start_time);
    let exercise_label = if exercise_count != 1 {
        format!("{} exercises", exercise_count)
    } else {
        format!("{} exercise", exercise_count)
    };

    rsx! {
        article { class: "session-card",
            div { class: "session-card__header",
                div { class: "session-card__date", "{date_str}" }
                button {
                    onclick: move |_| show_delete_confirm.set(true),
                    class: "session-card__delete-btn",
                    title: "Delete session",
                    "🗑️"
                }
            }
            div { class: "session-card__stats",
                span { class: "session-card__stat", "⏱ {format_time(duration)}" }
                span { class: "session-card__stat", "🏋️ {exercise_label}" }
            }
            if !session.exercise_logs.is_empty() {
                div { class: "session-card__exercises",
                    for log in session.exercise_logs.iter().take(3) {
                        span { class: "session-card__exercise-name", "{log.exercise_name}" }
                    }
                    if session.exercise_logs.len() > 3 {
                        span { class: "session-card__more",
                            "+{session.exercise_logs.len() - 3} more"
                        }
                    }
                }
            }

            // Delete confirmation modal
            if *show_delete_confirm.read() {
                div {
                    class: "delete-modal-overlay",
                    onclick: move |_| show_delete_confirm.set(false),
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
}

