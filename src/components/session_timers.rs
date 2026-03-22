use crate::models::{format_time, format_time_i64, get_current_timestamp};
use dioxus::prelude::*;
/// Timer tick interval in milliseconds
#[cfg(target_arch = "wasm32")]
const TIMER_TICK_MS: u32 = 1_000;
/// Send a notification using the Web Notifications API via the active service
/// worker registration. Using the service-worker path instead of `new
/// Notification()` is required on mobile browsers (Android Chrome) to fire
/// sound and vibration correctly.
/// `is_duration_bell` selects a different message to distinguish from rest alerts.
#[cfg(target_arch = "wasm32")]
pub(super) fn send_notification(is_duration_bell: bool) {
    use web_sys::{NotificationOptions, NotificationPermission};
    if web_sys::Notification::permission() != NotificationPermission::Granted {
        return;
    }
    let (title, body) = if is_duration_bell {
        ("Duration reached", "Target exercise duration reached!")
    } else {
        ("Rest over", "Time to start your next set!")
    };
    let title = title.to_string();
    let opts = NotificationOptions::new();
    opts.set_body(body);
    opts.set_tag(if is_duration_bell {
        "logout-duration"
    } else {
        "logout-rest"
    });
    let vibrate = serde_wasm_bindgen::to_value(&[200u32, 100, 200]).unwrap();
    opts.set_vibrate(&vibrate);
    wasm_bindgen_futures::spawn_local(async move {
        if let Some(window) = web_sys::window() {
            let sw = window.navigator().service_worker();
            if let Ok(ready_promise) = sw.ready() {
                match wasm_bindgen_futures::JsFuture::from(ready_promise).await {
                    Ok(reg_val) => {
                        let reg: web_sys::ServiceWorkerRegistration = reg_val.into();
                        let _ = reg.show_notification_with_options(&title, &opts);
                        return;
                    }
                    Err(e) => {
                        log::warn!("Service worker not ready for notification: {:?}", e);
                    }
                }
            }
        }
        let _ = web_sys::Notification::new_with_options(&title, &opts);
    });
}
/// Renders the session elapsed time, updating every second.
#[component]
pub(super) fn SessionDurationDisplay(
    session_start_time: u64,
    session_is_active: bool,
    paused_at: Option<u64>,
) -> Element {
    let mut now_tick = use_signal(get_current_timestamp);
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        if !session_is_active || paused_at.is_some() {
            return;
        }
        loop {
            #[cfg(target_arch = "wasm32")]
            gloo_timers::future::TimeoutFuture::new(TIMER_TICK_MS).await;
            #[cfg(not(target_arch = "wasm32"))]
            tokio::time::sleep(std::time::Duration::from_millis(1_000)).await;
            now_tick.set(get_current_timestamp());
        }
    });
    let effective_now = paused_at.unwrap_or_else(|| *now_tick.read());
    let duration = if session_is_active {
        effective_now.saturating_sub(session_start_time)
    } else {
        0
    };
    rsx! { "{format_time(duration)}" }
}
/// Renders the rest timer inside the session header.
///
/// Behaviour:
/// - When `start_time` is `None` (not currently resting): shows the configured
///   `rest_duration` as a static reference value so the user can see the setting.
/// - When `start_time` is `Some`: counts *down* from `rest_duration` toward zero
///   and continues into negative values without limit.
/// - Font colour turns red (via the `exceeded` CSS class) once the countdown
///   reaches zero or below.
/// - Fires a notification bell at each completed rest interval.
#[component]
pub(super) fn RestTimerDisplay(
    /// Timestamp when the current rest period started, or `None` when idle.
    start_time: Option<u64>,
    /// Configured rest duration in seconds (used for countdown and idle display).
    rest_duration: u64,
    paused_at: Option<u64>,
) -> Element {
    let mut now_tick = use_signal(get_current_timestamp);
    let mut bell_count = use_signal(|| 0u64);
    let mut last_seen_start = use_signal(|| start_time);
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        loop {
            #[cfg(target_arch = "wasm32")]
            gloo_timers::future::TimeoutFuture::new(TIMER_TICK_MS).await;
            #[cfg(not(target_arch = "wasm32"))]
            tokio::time::sleep(std::time::Duration::from_millis(1_000)).await;
            now_tick.set(get_current_timestamp());
        }
    });
    if *last_seen_start.read() != start_time {
        last_seen_start.set(start_time);
        bell_count.set(0);
    }
    let Some(start) = start_time else {
        return rsx! {
            div { class: "rest-timer", "🛋️ {format_time(rest_duration)}" }
        };
    };
    let effective_now = paused_at.unwrap_or_else(|| *now_tick.read());
    let elapsed = effective_now.saturating_sub(start);
    let rd = rest_duration;
    if rd > 0 && elapsed > 0 {
        let intervals = elapsed / rd;
        let prev_count = *bell_count.read();
        if intervals > prev_count {
            bell_count.set(intervals);
            #[cfg(target_arch = "wasm32")]
            send_notification(false);
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
pub(super) fn ExerciseElapsedTimer(
    exercise_start: Option<u64>,
    last_duration: Option<u64>,
    mut duration_bell_rung: Signal<bool>,
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
    let effective_now = paused_at.unwrap_or_else(|| *now_tick.read());
    let elapsed = if let Some(start) = exercise_start {
        effective_now.saturating_sub(start)
    } else {
        0
    };
    if !*duration_bell_rung.read() {
        if let Some(dur) = last_duration {
            if dur > 0 && elapsed >= dur {
                duration_bell_rung.set(true);
                #[cfg(target_arch = "wasm32")]
                send_notification(true);
            }
        }
    }
    let timer_reached = last_duration.is_some_and(|d| d > 0 && elapsed >= d);
    rsx! {
        div { class: if timer_reached { "exercise-timer reached" } else { "exercise-timer" },
            "⏱ {format_time(elapsed)}"
        }
    }
}
