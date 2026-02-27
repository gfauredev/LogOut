use crate::models::{format_time, get_current_timestamp};
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
    // Vibrate is only honoured by service-worker notifications on mobile.
    let vibrate = serde_wasm_bindgen::to_value(&[200u32, 100, 200]).unwrap();
    opts.set_vibrate(&vibrate);

    // Prefer service-worker showNotification (works on mobile); fall back to
    // direct Notification API when no service worker is registered.
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
        // Fallback: direct Notification API (desktop / no service worker)
        let _ = web_sys::Notification::new_with_options(&title, &opts);
    });
}

// ── Isolated timer components ──────────────────────────────────────────────
// Each component owns its own tick coroutine so that only the timer display
// re-renders every second, preventing unnecessary re-renders of the main
// session form (input fields, exercise list, etc.).

/// Renders the session elapsed time, updating every second.
#[component]
pub(super) fn SessionDurationDisplay(session_start_time: u64, session_is_active: bool) -> Element {
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
    let tick = *now_tick.read();
    let duration = if session_is_active {
        tick.saturating_sub(session_start_time)
    } else {
        0
    };
    rsx! { "{format_time(duration)}" }
}

/// Renders the rest timer and fires a notification when the rest period ends.
#[component]
pub(super) fn RestTimerDisplay(
    rest_start_time: Signal<Option<u64>>,
    rest_duration: Signal<u64>,
    mut rest_bell_count: Signal<u64>,
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

    let tick = *now_tick.read();
    let Some(start) = *rest_start_time.read() else {
        return rsx! {};
    };
    let elapsed = tick.saturating_sub(start);
    let rd = *rest_duration.read();

    // Fire bell at each completed rest interval
    if rd > 0 && elapsed > 0 {
        let intervals = elapsed / rd;
        let prev_count = *rest_bell_count.read();
        if intervals > prev_count {
            rest_bell_count.set(intervals);
            #[cfg(target_arch = "wasm32")]
            send_notification(false);
        }
    }

    let exceeded = rd > 0 && elapsed >= rd;
    rsx! {
        div {
            class: if exceeded { "rest-timer rest-timer--exceeded" } else { "rest-timer" },
            "🛋️ Rest: {format_time(elapsed)}"
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

    let tick = *now_tick.read();
    let elapsed = if let Some(start) = exercise_start {
        tick.saturating_sub(start)
    } else {
        0
    };

    // Fire duration bell once when the previous exercise duration is reached
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
        div {
            class: if timer_reached { "exercise-static-timer exercise-static-timer--reached" } else { "exercise-static-timer" },
            "⏱ {format_time(elapsed)}"
        }
    }
}
