use crate::models::{format_time, format_time_i64, get_current_timestamp, Force};
use dioxus::prelude::*;
use dioxus_i18n::t;

/// Timer tick interval in milliseconds.
#[cfg(target_arch = "wasm32")]
const TIMER_TICK_MS: u32 = 1_000;

/// How many milliseconds ahead of the target time to fire notifications.
///
/// Sending slightly early compensates for scheduling jitter so the alert
/// arrives as close to the target moment as possible.
pub const NOTIF_EARLY_MS: u64 = 250;

/// Schedule a one-shot duration-reached notification for the given exercise.
///
/// * On **WASM**: uses `gloo_timers` in a `spawn_local` task so the timeout
///   fires accurately without blocking the UI.  `duration_bell_rung` is set
///   inside the callback to prevent the tick-based fallback from sending a
///   duplicate.
/// * On **native**: the tick-based path is accurate enough (±1 s) and avoids
///   the complexity of crossing Dioxus signal boundaries from a `tokio::spawn`
///   thread; no extra task is spawned here.
#[allow(unused_mut)]
fn schedule_duration_notification(
    exercise_start: Option<u64>,
    last_duration: Option<u64>,
    mut duration_bell_rung: Signal<bool>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        let Some(start) = exercise_start else { return };
        let Some(dur) = last_duration else { return };
        if dur == 0 || *duration_bell_rung.read() {
            return;
        }
        let fire_at_secs = start + dur;
        let now = get_current_timestamp();
        // Only schedule if the duration hasn't already elapsed; the tick-based
        // fallback fires immediately when elapsed >= dur on the next tick.
        if fire_at_secs <= now {
            return;
        }
        let delay_ms = ((fire_at_secs - now) * 1_000)
            .saturating_sub(NOTIF_EARLY_MS)
            .min(u32::MAX as u64) as u32;
        let title = t!("notif-duration-title").to_string();
        let body = t!("notif-duration-body").to_string();
        wasm_bindgen_futures::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(delay_ms).await;
            // Re-check to avoid a duplicate if the tick fired first.
            if !*duration_bell_rung.peek() {
                duration_bell_rung.set(true);
                crate::services::notifications::send_notification(&title, &body, "logout-duration");
            }
        });
    }
    // On native, suppress unused-variable warnings; the tick handles it.
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (exercise_start, last_duration, duration_bell_rung);
}

/// Renders the rest-timer with a countdown.
///
/// Notification scheduling (one-shot + repeated-exceed) is handled by
/// [`GlobalSessionHeader`](super::active_session::GlobalSessionHeader) which
/// has access to the rest context.  This component is kept for cases where a
/// standalone rest timer with notification is needed independently of the
/// session header.
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

    let Some(start) = start_time else {
        return rsx! {
            div { class: "rest-timer", "🛋️ {format_time(rest_duration)}" }
        };
    };

    let effective_now = paused_at.unwrap_or_else(|| *now_tick.read());
    let elapsed = effective_now.saturating_sub(start);
    let rd = rest_duration;

    // Tick-based check for 2nd+ exceeded intervals.
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
/// All Time High duration from the last log is reached.
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
    // Schedule a precise one-shot notification (WASM only; native uses tick).
    use_effect(move || {
        schedule_duration_notification(exercise_start, last_duration, duration_bell_rung);
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

    // Tick-based: fires immediately on native, or as a fallback on WASM.
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
    // Schedule a precise one-shot notification (WASM only; native uses tick).
    use_effect(move || {
        schedule_duration_notification(exercise_start, last_duration, duration_bell_rung);
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
