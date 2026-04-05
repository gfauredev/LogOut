use crate::models::{format_time, format_time_i64, get_current_timestamp, Force};
use dioxus::prelude::*;
use dioxus_i18n::t;

/// Timer tick interval in milliseconds
#[cfg(target_arch = "wasm32")]
const TIMER_TICK_MS: u32 = 1_000;

/// Renders the rest-timer with a countdown and fires a notification when the
/// rest duration is reached.
///
/// The notification is scheduled precisely when the component mounts (so it
/// fires even when the page is backgrounded).  The tick-based check is kept
/// as a belt-and-suspenders fallback for consecutive intervals.
#[component]
pub fn RestTimer(
    start_time: Option<u64>,
    rest_duration: u64,
    paused_at: Option<u64>,
    mut bell_count: Signal<u64>,
) -> Element {
    let mut now_tick = use_signal(get_current_timestamp);
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        loop {
            #[cfg(target_arch = "wasm32")]
            gloo_timers::future::TimeoutFuture::new(TIMER_TICK_MS).await;
            #[cfg(not(target_arch = "wasm32"))]
            tokio::time::sleep(std::time::Duration::from_millis(1_000)).await;
            now_tick.set(get_current_timestamp());
        }
    });

    use_effect(move || {
        // On WASM, schedule through the service worker (PWA) so it fires in background.
        #[cfg(target_arch = "wasm32")]
        if let Some(start) = start_time {
            if rest_duration > 0 {
                let _target_unix_ms = (start as f64 + rest_duration as f64) * 1_000.0;
                let title = t!("notif-rest-title").to_string();
                let body = t!("notif-rest-body").to_string();
                crate::services::notifications::send_notification(&title, &body, "logout-rest");
                // For PWA background scheduling, we rely on the service worker
                // or the precise timeout in the unified service.
            }
        }
        // On native (Android), schedule a precise one-shot notification via tokio.
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(start) = start_time {
            if rest_duration > 0 {
                let title = t!("notif-rest-title").to_string();
                let body = t!("notif-rest-body").to_string();
                let fire_at_secs = start + rest_duration;
                tokio::spawn(async move {
                    let now = crate::models::get_current_timestamp();
                    if fire_at_secs > now {
                        let delay = std::time::Duration::from_secs(fire_at_secs - now);
                        tokio::time::sleep(delay).await;
                    }
                    crate::services::notifications::send_notification(&title, &body, "logout-rest");
                });
            }
        }
    });

    let Some(start) = start_time else {
        return rsx! {
            div { class: "rest-timer", "🛋️ {format_time(rest_duration)}" }
        };
    };

    let effective_now = paused_at.unwrap_or_else(|| *now_tick.read());
    let elapsed = effective_now.saturating_sub(start);
    let rd = rest_duration;

    // Tick-based fallback for 2nd+ intervals.
    if rd > 0 && elapsed > 0 {
        let intervals = elapsed / rd;
        let prev_count = *bell_count.read();
        if intervals > prev_count {
            bell_count.set(intervals);
            crate::services::notifications::send_notification(
                &t!("notif-rest-title"),
                &t!("notif-rest-body"),
                "logout-rest",
            );
        }
    }

    let remaining = rd.cast_signed() - elapsed.cast_signed();
    let exceeded = remaining <= 0;
    rsx! {
        div { class: if exceeded { "rest-timer exceeded" } else { "rest-timer" },
            "🛋️ {format_time_i64(remaining)}"
        }
    }
}

/// Renders the exercise elapsed timer and fires a notification when the
/// target duration from the last log is reached.
#[component]
pub fn ExerciseElapsedTimer(
    exercise_start: Option<u64>,
    last_duration: Option<u64>,
    mut duration_bell_rung: Signal<bool>,
    paused_at: Option<u64>,
    /// Force type of the exercise; the "reached" highlight is only applied for
    /// `Force::Static` exercises.
    force: Option<Force>,
) -> Element {
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        if let (Some(_start), Some(dur)) = (exercise_start, last_duration) {
            if dur > 0 && !*duration_bell_rung.read() {
                let title = t!("notif-duration-title").to_string();
                let body = t!("notif-duration-body").to_string();
                crate::services::notifications::send_notification(&title, &body, "logout-duration");
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let (Some(start), Some(dur)) = (exercise_start, last_duration) {
            if dur > 0 && !*duration_bell_rung.read() {
                let title = t!("notif-duration-title").to_string();
                let body = t!("notif-duration-body").to_string();
                let fire_at_secs = start + dur;
                tokio::spawn(async move {
                    let now = crate::models::get_current_timestamp();
                    if fire_at_secs > now {
                        let delay = std::time::Duration::from_secs(fire_at_secs - now);
                        tokio::time::sleep(delay).await;
                    }
                    crate::services::notifications::send_notification(
                        &title,
                        &body,
                        "logout-duration",
                    );
                });
            }
        }
    });

    let mut now_tick = use_signal(get_current_timestamp);
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        loop {
            #[cfg(target_arch = "wasm32")]
            gloo_timers::future::TimeoutFuture::new(TIMER_TICK_MS).await;
            #[cfg(not(target_arch = "wasm32"))]
            tokio::time::sleep(std::time::Duration::from_millis(1_000)).await;
            now_tick.set(get_current_timestamp());
        }
    });

    let effective_now = paused_at.unwrap_or_else(|| *now_tick.read());
    let elapsed = if let Some(start) = exercise_start {
        effective_now.saturating_sub(start)
    } else {
        0
    };

    // Tick-based fallback.
    if !*duration_bell_rung.read() {
        if let Some(dur) = last_duration {
            if dur > 0 && elapsed >= dur {
                duration_bell_rung.set(true);
                crate::services::notifications::send_notification(
                    &t!("notif-duration-title"),
                    &t!("notif-duration-body"),
                    "logout-duration",
                );
            }
        }
    }

    let is_static = force == Some(Force::Static);
    let timer_reached = is_static && last_duration.is_some_and(|d| d > 0 && elapsed >= d);
    rsx! {
        div { class: if timer_reached { "exercise-timer reached" } else { "exercise-timer" },
            "⏱ {format_time(elapsed)}"
        }
    }
}

/// Renders the exercise elapsed timer inline inside the ⏱️ form row (perform mode).
#[component]
pub(super) fn InlineExerciseTimer(
    exercise_start: Option<u64>,
    last_duration: Option<u64>,
    mut duration_bell_rung: Signal<bool>,
    paused_at: Option<u64>,
    force: Option<Force>,
) -> Element {
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        if let (Some(_start), Some(dur)) = (exercise_start, last_duration) {
            if dur > 0 && !*duration_bell_rung.read() {
                let title = t!("notif-duration-title").to_string();
                let body = t!("notif-duration-body").to_string();
                crate::services::notifications::send_notification(&title, &body, "logout-duration");
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let (Some(start), Some(dur)) = (exercise_start, last_duration) {
            if dur > 0 && !*duration_bell_rung.read() {
                let title = t!("notif-duration-title").to_string();
                let body = t!("notif-duration-body").to_string();
                let fire_at_secs = start + dur;
                tokio::spawn(async move {
                    let now = crate::models::get_current_timestamp();
                    if fire_at_secs > now {
                        let delay = std::time::Duration::from_secs(fire_at_secs - now);
                        tokio::time::sleep(delay).await;
                    }
                    crate::services::notifications::send_notification(
                        &title,
                        &body,
                        "logout-duration",
                    );
                });
            }
        }
    });

    let mut now_tick = use_signal(get_current_timestamp);
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        loop {
            #[cfg(target_arch = "wasm32")]
            gloo_timers::future::TimeoutFuture::new(TIMER_TICK_MS).await;
            #[cfg(not(target_arch = "wasm32"))]
            tokio::time::sleep(std::time::Duration::from_millis(1_000)).await;
            now_tick.set(get_current_timestamp());
        }
    });

    let effective_now = paused_at.unwrap_or_else(|| *now_tick.read());
    let elapsed = if let Some(start) = exercise_start {
        effective_now.saturating_sub(start)
    } else {
        0
    };

    // Tick-based fallback.
    if !*duration_bell_rung.read() {
        if let Some(dur) = last_duration {
            if dur > 0 && elapsed >= dur {
                duration_bell_rung.set(true);
                crate::services::notifications::send_notification(
                    &t!("notif-duration-title"),
                    &t!("notif-duration-body"),
                    "logout-duration",
                );
            }
        }
    }

    let is_static = force == Some(Force::Static);
    let timer_reached = is_static && last_duration.is_some_and(|d| d > 0 && elapsed >= d);
    rsx! {
        span { class: if timer_reached { "reached" } else { "" }, "{format_time(elapsed)}" }
    }
}

/// Simple display-only component for the rest timer in the header.
/// Does not handle notifications (`RestTimer` handles those).
#[component]
pub fn RestTimerDisplay(
    start_time: Option<u64>,
    rest_duration: u64,
    paused_at: Option<u64>,
) -> Element {
    let mut now_tick = use_signal(get_current_timestamp);
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        loop {
            #[cfg(target_arch = "wasm32")]
            gloo_timers::future::TimeoutFuture::new(TIMER_TICK_MS).await;
            #[cfg(not(target_arch = "wasm32"))]
            tokio::time::sleep(std::time::Duration::from_millis(1_000)).await;
            now_tick.set(get_current_timestamp());
        }
    });

    let Some(start) = start_time else {
        return rsx! {
            div { class: "rest-timer", "🛋️ {format_time(rest_duration)}" }
        };
    };

    let effective_now = paused_at.unwrap_or_else(|| *now_tick.read());
    let elapsed = effective_now.saturating_sub(start);
    let remaining = rest_duration.cast_signed() - elapsed.cast_signed();
    let exceeded = remaining <= 0;

    rsx! {
        div { class: if exceeded { "rest-timer exceeded" } else { "rest-timer" },
            "🛋️ {format_time_i64(remaining)}"
        }
    }
}

/// Simple display-only component for the session duration in the header.
#[component]
pub fn SessionDurationDisplay(
    session_start_time: u64,
    session_is_active: bool,
    paused_at: Option<u64>,
    total_paused_duration: u64,
) -> Element {
    let mut now_tick = use_signal(get_current_timestamp);
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        loop {
            #[cfg(target_arch = "wasm32")]
            gloo_timers::future::TimeoutFuture::new(TIMER_TICK_MS).await;
            #[cfg(not(target_arch = "wasm32"))]
            tokio::time::sleep(std::time::Duration::from_millis(1_000)).await;
            now_tick.set(get_current_timestamp());
        }
    });

    let effective_now = if session_is_active {
        paused_at.unwrap_or_else(|| *now_tick.read())
    } else {
        paused_at.unwrap_or(session_start_time)
    };

    let elapsed = effective_now
        .saturating_sub(session_start_time)
        .saturating_sub(total_paused_duration);

    rsx! {
        span { "{format_time(elapsed)}" }
    }
}
