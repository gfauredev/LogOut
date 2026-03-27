use super::super::session_timers::{RestTimerDisplay, SessionDurationDisplay};
use dioxus::prelude::*;
use dioxus_i18n::t;

/// Sticky session header showing the elapsed timer, rest timer, and session controls.
#[component]
pub fn SessionHeader(
    session_start_time: u64,
    session_is_active: bool,
    paused_at: Option<u64>,
    /// Total cumulative seconds spent paused (not counting the current pause).
    total_paused_duration: u64,
    exercise_count: usize,
    /// Timestamp when the current rest period began, or `None` when not resting.
    rest_start_time: Option<u64>,
    /// Configured rest duration (seconds).
    rest_duration: u64,
    on_click_timer: EventHandler<()>,
    on_pause: EventHandler<()>,
    on_finish: EventHandler<()>,
) -> Element {
    let is_paused = paused_at.is_some();
    rsx! {
        header { class: "session",
            h2 { tabindex: 0, {t!("session-title")} }
            div {
                class: "session-timers",
                onclick: move |_| on_click_timer.call(()),
                title: t!("session-timer-title"),
                time {
                    SessionDurationDisplay {
                        session_start_time,
                        session_is_active,
                        paused_at,
                        total_paused_duration,
                    }
                }
                RestTimerDisplay {
                    start_time: rest_start_time,
                    rest_duration,
                    paused_at,
                }
            }
            button {
                class: "edit",
                onclick: move |_| on_pause.call(()),
                title: if is_paused { t!("session-resume-btn") } else { t!("session-pause-btn") },
                if is_paused {
                    "▶️"
                } else {
                    "⏸️"
                }
            }
            if exercise_count == 0 {
                button {
                    class: "back",
                    onclick: move |_| on_finish.call(()),
                    title: t!("session-cancel-btn"),
                    "❌"
                }
            } else {
                button {
                    class: "save",
                    onclick: move |_| on_finish.call(()),
                    title: t!("session-finish-btn"),
                    "💾"
                }
            }
        }
    }
}
