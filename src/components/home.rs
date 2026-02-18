use dioxus::prelude::*;
use crate::models::{WorkoutSession, format_time};
use crate::services::storage;
use crate::components::{SessionView, BottomNav, ActiveTab};

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
            div { class: "app-content",
                if *has_active.read() {
                    SessionView {}
                } else {
                    div { class: "sessions-tab",
                        div { class: "sessions-tab__header",
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
        div { class: "session-card",
            div { class: "session-card__date", "{date_str}" }
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
        }
    }
}

fn format_session_date(timestamp: u64) -> String {
    #[cfg(target_arch = "wasm32")]
    let current_time = js_sys::Date::now() / 1000.0;
    #[cfg(not(target_arch = "wasm32"))]
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as f64;

    let days_ago = ((current_time - timestamp as f64) / 86400.0) as i64;
    match days_ago {
        0 => "Today".to_string(),
        1 => "Yesterday".to_string(),
        n => format!("{} days ago", n),
    }
}

