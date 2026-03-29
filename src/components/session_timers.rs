use crate::models::{format_time, format_time_i64, get_current_timestamp, Force};
use dioxus::prelude::*;
#[cfg(target_arch = "wasm32")]
use dioxus_i18n::t;
/// Timer tick interval in milliseconds
#[cfg(target_arch = "wasm32")]
const TIMER_TICK_MS: u32 = 1_000;
/// How many milliseconds early to send a notification so that delivery latency
/// is absorbed and the sound reaches the user at the right moment.
#[cfg(target_arch = "wasm32")]
const NOTIF_ADVANCE_MS: f64 = 500.0;
/// Send a notification using the Web Notifications API via the active service
/// worker registration. Using the service-worker path instead of `new
/// Notification()` is required on mobile browsers (Android Chrome) to fire
/// sound and vibration correctly.
#[cfg(target_arch = "wasm32")]
pub(crate) fn send_notification(title: &str, body: &str, tag: &'static str) {
    use web_sys::{NotificationOptions, NotificationPermission};
    if web_sys::Notification::permission() != NotificationPermission::Granted {
        return;
    }
    let title = title.to_string();
    let body = body.to_string();
    let opts = NotificationOptions::new();
    opts.set_body(&body);
    opts.set_tag(tag);
    let vibrate = serde_wasm_bindgen::to_value(&[200u32, 100, 200]).ok();
    if let Some(v) = vibrate {
        opts.set_vibrate(&v);
    }
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

/// Schedule a notification to fire at `target_unix_ms` (Unix time in milliseconds).
///
/// Strategy (best-to-fallback):
/// 1. **Notification Triggers API** (`TimestampTrigger`) — schedules through the
///    service worker so the notification fires even when the page is closed or
///    backgrounded (Chrome 80+ with the feature enabled / Origin Trial).
/// 2. **Precise `gloo_timers` sleep** — waits for the exact remaining duration
///    then calls [`send_notification`].  Works only while the page is visible but
///    is accurate to ~10 ms.
/// 3. **Immediate** — if the target is already in the past, fires immediately.
///
/// The notification is sent `NOTIF_ADVANCE_MS` before the real target so that
/// delivery overhead is absorbed and the sound arrives on time.
#[cfg(target_arch = "wasm32")]
pub(crate) fn schedule_notification_at(
    title: String,
    body: String,
    tag: &'static str,
    target_unix_ms: f64,
) {
    use web_sys::NotificationPermission;
    if web_sys::Notification::permission() != NotificationPermission::Granted {
        return;
    }
    // Fire NOTIF_ADVANCE_MS before the real target so delivery latency is absorbed.
    let fire_at_ms = target_unix_ms - NOTIF_ADVANCE_MS;
    wasm_bindgen_futures::spawn_local(async move {
        // --- 1. Try Notification Triggers API via JS reflection ---
        let used_triggers = try_schedule_via_triggers_api(&title, &body, tag, fire_at_ms).await;
        if used_triggers {
            return;
        }
        // --- 2. Precise gloo_timers sleep ---
        let now_ms = js_sys::Date::now();
        let delay_ms = (fire_at_ms - now_ms).max(0.0) as u32;
        gloo_timers::future::TimeoutFuture::new(delay_ms).await;
        send_notification(&title, &body, tag);
    });
}

/// Attempt to schedule a notification through the Notification Triggers API
/// (`TimestampTrigger`).  Returns `true` when the API is available and the
/// schedule call succeeded; `false` otherwise (caller should fall back).
#[cfg(target_arch = "wasm32")]
async fn try_schedule_via_triggers_api(
    title: &str,
    body: &str,
    tag: &'static str,
    fire_at_ms: f64,
) -> bool {
    use wasm_bindgen::JsCast as _;
    // TimestampTrigger is an experimental API — check if it exists on globalThis.
    let global = js_sys::global();
    let trigger_key = wasm_bindgen::JsValue::from_str("TimestampTrigger");
    if !js_sys::Reflect::has(&global, &trigger_key).unwrap_or(false) {
        return false;
    }
    let trigger_cls = match js_sys::Reflect::get(&global, &trigger_key) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let args = js_sys::Array::of1(&wasm_bindgen::JsValue::from_f64(fire_at_ms));
    let trigger = match js_sys::Reflect::construct(&trigger_cls.unchecked_into(), &args) {
        Ok(t) => t,
        Err(_) => return false,
    };
    let Some(window) = web_sys::window() else {
        return false;
    };
    let sw = window.navigator().service_worker();
    let Ok(ready_promise) = sw.ready() else {
        return false;
    };
    let reg: web_sys::ServiceWorkerRegistration =
        match wasm_bindgen_futures::JsFuture::from(ready_promise).await {
            Ok(v) => v.into(),
            Err(_) => return false,
        };
    let opts = web_sys::NotificationOptions::new();
    opts.set_body(body);
    opts.set_tag(tag);
    if let Some(v) = serde_wasm_bindgen::to_value(&[200u32, 100, 200]).ok() {
        opts.set_vibrate(&v);
    }
    // Attach the trigger to the options object via reflection.
    let show_trigger_key = wasm_bindgen::JsValue::from_str("showTrigger");
    let _ = js_sys::Reflect::set(&opts, &show_trigger_key, &trigger);
    match reg.show_notification_with_options(title, &opts) {
        Ok(_) => {
            log::debug!("Notification scheduled via Triggers API at {fire_at_ms}");
            true
        }
        Err(e) => {
            log::warn!("Triggers API show_notification failed: {:?}", e);
            false
        }
    }
}

/// Renders the session elapsed time, updating every second.
#[component]
pub(super) fn SessionDurationDisplay(
    session_start_time: u64,
    session_is_active: bool,
    paused_at: Option<u64>,
    /// Total cumulative seconds the session has spent paused so far, not
    /// counting any ongoing pause (that is handled separately via `paused_at`).
    total_paused_duration: u64,
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
        let elapsed = effective_now.saturating_sub(session_start_time);
        let ongoing_pause = paused_at.map_or(0, |p| effective_now.saturating_sub(p));
        elapsed
            .saturating_sub(total_paused_duration)
            .saturating_sub(ongoing_pause)
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
/// - Fires a notification bell at each completed rest interval.  The first
///   interval notification is scheduled precisely (Triggers API or exact sleep);
///   subsequent repeats use the existing tick-based approach.
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
    // When rest starts (or restarts), schedule the first interval notification precisely.
    if *last_seen_start.read() != start_time {
        last_seen_start.set(start_time);
        bell_count.set(0);
        #[cfg(target_arch = "wasm32")]
        if let Some(start) = start_time {
            if rest_duration > 0 {
                let target_unix_ms = (start as f64 + rest_duration as f64) * 1_000.0;
                let title = t!("notif-rest-title").to_string();
                let body = t!("notif-rest-body").to_string();
                schedule_notification_at(title, body, "logout-rest", target_unix_ms);
            }
        }
    }
    let Some(start) = start_time else {
        return rsx! {
            div { class: "rest-timer", "🛋️ {format_time(rest_duration)}" }
        };
    };
    let effective_now = paused_at.unwrap_or_else(|| *now_tick.read());
    let elapsed = effective_now.saturating_sub(start);
    let rd = rest_duration;
    // Tick-based fallback for 2nd+ intervals (Triggers API only scheduled the 1st).
    if rd > 0 && elapsed > 0 {
        let intervals = elapsed / rd;
        let prev_count = *bell_count.read();
        if intervals > prev_count {
            bell_count.set(intervals);
            #[cfg(target_arch = "wasm32")]
            send_notification(
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
///
/// The notification is scheduled precisely when the component mounts (so it
/// fires even when the page is backgrounded via the Triggers API, or at the
/// exact millisecond via `gloo_timers`).  The tick-based check is kept as a
/// belt-and-suspenders fallback.
#[component]
pub(super) fn ExerciseElapsedTimer(
    exercise_start: Option<u64>,
    last_duration: Option<u64>,
    mut duration_bell_rung: Signal<bool>,
    paused_at: Option<u64>,
    /// Force type of the exercise; the "reached" highlight is only applied for
    /// `Force::Static` exercises.
    force: Option<Force>,
) -> Element {
    // Schedule the notification precisely when the exercise starts.
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        if !*duration_bell_rung.read() {
            if let (Some(start), Some(dur)) = (exercise_start, last_duration) {
                if dur > 0 {
                    let target_unix_ms = (start as f64 + dur as f64) * 1_000.0;
                    let title = t!("notif-duration-title").to_string();
                    let body = t!("notif-duration-body").to_string();
                    schedule_notification_at(title, body, "logout-duration", target_unix_ms);
                }
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
    // Tick-based fallback: mark bell as rung once elapsed reaches the target.
    if !*duration_bell_rung.read() {
        if let Some(dur) = last_duration {
            if dur > 0 && elapsed >= dur {
                duration_bell_rung.set(true);
                #[cfg(target_arch = "wasm32")]
                send_notification(
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
///
/// Unlike [`ExerciseElapsedTimer`] this component renders only the time value
/// so it can be embedded inside the exercise-edit grid without adding an extra
/// wrapper element.
#[component]
pub(super) fn InlineExerciseTimer(
    exercise_start: Option<u64>,
    last_duration: Option<u64>,
    mut duration_bell_rung: Signal<bool>,
    paused_at: Option<u64>,
    /// Force type of the exercise; the "reached" highlight is only applied for
    /// `Force::Static` exercises.
    force: Option<Force>,
) -> Element {
    // Schedule the notification precisely when the component mounts.
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        if !*duration_bell_rung.read() {
            if let (Some(start), Some(dur)) = (exercise_start, last_duration) {
                if dur > 0 {
                    let target_unix_ms = (start as f64 + dur as f64) * 1_000.0;
                    let title = t!("notif-duration-title").to_string();
                    let body = t!("notif-duration-body").to_string();
                    schedule_notification_at(title, body, "logout-duration", target_unix_ms);
                }
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
    // Tick-based fallback: mark bell as rung once elapsed reaches the target.
    if !*duration_bell_rung.read() {
        if let Some(dur) = last_duration {
            if dur > 0 && elapsed >= dur {
                duration_bell_rung.set(true);
                #[cfg(target_arch = "wasm32")]
                send_notification(
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
        time { class: if timer_reached { "exercise-timer reached" } else { "exercise-timer" },
            "{format_time(elapsed)}"
        }
    }
}
