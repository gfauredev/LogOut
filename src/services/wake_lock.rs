/// Wake Lock – prevent the device screen from sleeping while the app is open.
///
/// Uses the [Screen Wake Lock API](https://developer.mozilla.org/en-US/docs/Web/API/Screen_Wake_Lock_API)
/// via `js_sys` reflection so that no additional `web-sys` feature flags are
/// required.  The call is a progressive enhancement: if the API is unavailable
/// the function silently does nothing.
///
/// A new lock is requested whenever the page becomes visible again (e.g. after
/// the user switches back from another app), which keeps the lock active
/// throughout the session.
#[cfg(all(target_arch = "wasm32", feature = "web-platform"))]
pub fn enable_wake_lock() {
    use wasm_bindgen::prelude::*;
    wasm_bindgen_futures::spawn_local(async {
        if let Err(e) = request_wake_lock().await {
            log::warn!("Wake Lock request failed: {:?}", e);
        }
    });
    if let Some(window) = web_sys::window() {
        let document = match window.document() {
            Some(d) => d,
            None => return,
        };
        let closure = Closure::<dyn FnMut()>::new(|| {
            wasm_bindgen_futures::spawn_local(async {
                let Some(window) = web_sys::window() else {
                    return;
                };
                let Some(document) = window.document() else {
                    return;
                };
                if document.visibility_state() == web_sys::VisibilityState::Visible {
                    let _ = request_wake_lock().await;
                }
            });
        });
        let _ = document
            .add_event_listener_with_callback("visibilitychange", closure.as_ref().unchecked_ref());
        closure.forget();
    }
}
/// Calls `navigator.wakeLock.request("screen")` via JS reflection.
#[cfg(all(target_arch = "wasm32", feature = "web-platform"))]
async fn request_wake_lock() -> Result<(), String> {
    use js_sys::{Array, Function, Reflect};
    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_futures::JsFuture;
    let window = web_sys::window().ok_or("no window")?;
    let navigator = window.navigator();
    let wake_lock =
        Reflect::get(&navigator, &JsValue::from_str("wakeLock")).map_err(|e| format!("{:?}", e))?;
    if wake_lock.is_undefined() || wake_lock.is_null() {
        return Ok(());
    }
    let request_fn =
        Reflect::get(&wake_lock, &JsValue::from_str("request")).map_err(|e| format!("{:?}", e))?;
    let request_fn: Function = request_fn
        .dyn_into()
        .map_err(|_| "wakeLock.request is not a function".to_string())?;
    let args = Array::new();
    args.push(&JsValue::from_str("screen"));
    let promise: js_sys::Promise = request_fn
        .apply(&wake_lock, &args)
        .map_err(|e| format!("{:?}", e))?
        .dyn_into()
        .map_err(|_| "wakeLock.request did not return a Promise".to_string())?;
    JsFuture::from(promise)
        .await
        .map(|_| ())
        .map_err(|e| format!("{:?}", e))
}
#[cfg(not(all(target_arch = "wasm32", feature = "web-platform")))]
pub fn enable_wake_lock() {
    #[cfg(target_os = "android")]
    acquire_android_wake_lock();
}

/// Acquires an Android `PARTIAL_WAKE_LOCK` via JNI so that the CPU is not
/// suspended while the app is downloading images in the background (even when
/// the screen turns off).
///
/// The lock is acquired with a one-hour timeout as a safety net; in normal
/// usage it will be held until the process exits.  The `WAKE_LOCK` permission
/// is already declared in `Dioxus.toml`.
#[cfg(target_os = "android")]
fn acquire_android_wake_lock() {
    use jni::{objects::JObject, JavaVM};
    use ndk_context::android_context;

    let result = (|| -> Result<(), String> {
        let ctx = android_context();
        // SAFETY: the raw pointers come from the Android runtime and are valid
        // for the lifetime of the process.
        let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }
            .map_err(|e| format!("JavaVM::from_raw: {e}"))?;
        let mut env = vm
            .attach_current_thread()
            .map_err(|e| format!("attach_current_thread: {e}"))?;
        let activity = unsafe { JObject::from_raw(ctx.context() as jni::sys::jobject) };

        // val powerManager = context.getSystemService(Context.POWER_SERVICE)
        let power_service_str = env
            .get_static_field(
                "android/content/Context",
                "POWER_SERVICE",
                "Ljava/lang/String;",
            )
            .map_err(|e| format!("get POWER_SERVICE: {e}"))?
            .l()
            .map_err(|e| format!("POWER_SERVICE as object: {e}"))?;
        let power_manager = env
            .call_method(
                &activity,
                "getSystemService",
                "(Ljava/lang/String;)Ljava/lang/Object;",
                &[(&power_service_str).into()],
            )
            .map_err(|e| format!("getSystemService: {e}"))?
            .l()
            .map_err(|e| format!("PowerManager as object: {e}"))?;

        // val PARTIAL_WAKE_LOCK = PowerManager.PARTIAL_WAKE_LOCK  (= 1)
        let partial_wake_lock_flag = env
            .get_static_field("android/os/PowerManager", "PARTIAL_WAKE_LOCK", "I")
            .map_err(|e| format!("get PARTIAL_WAKE_LOCK: {e}"))?
            .i()
            .map_err(|e| format!("PARTIAL_WAKE_LOCK as int: {e}"))?;

        // val wl = powerManager.newWakeLock(PARTIAL_WAKE_LOCK, "logout:download")
        let tag = env
            .new_string("logout:download")
            .map_err(|e| format!("new_string: {e}"))?;
        let wake_lock = env
            .call_method(
                &power_manager,
                "newWakeLock",
                "(ILjava/lang/String;)Landroid/os/PowerManager$WakeLock;",
                &[
                    jni::objects::JValue::from(partial_wake_lock_flag),
                    (&tag).into(),
                ],
            )
            .map_err(|e| format!("newWakeLock: {e}"))?
            .l()
            .map_err(|e| format!("WakeLock as object: {e}"))?;

        // wl.acquire(3_600_000)  — 1-hour safety timeout
        env.call_method(
            &wake_lock,
            "acquire",
            "(J)V",
            &[jni::objects::JValue::from(3_600_000i64)],
        )
        .map_err(|e| format!("acquire: {e}"))?;

        Ok(())
    })();

    match result {
        Ok(()) => log::info!("Android PARTIAL_WAKE_LOCK acquired"),
        Err(e) => log::warn!("Failed to acquire Android wake lock: {e}"),
    }
}

/// JNI global reference to the screen wake lock, held while a session is active.
///
/// `None` when no session is active (wake lock not held).
#[cfg(target_os = "android")]
static SCREEN_WAKE_LOCK: std::sync::Mutex<Option<jni::objects::GlobalRef>> =
    std::sync::Mutex::new(None);

/// Configure Android lock-screen behaviour based on whether a session is active.
///
/// When `active` is `true`:
/// - `Activity.setShowWhenLocked(true)`: the app is displayed over the lock
///   screen when the user presses the power button or the screen times out.
/// - `Activity.setTurnScreenOn(true)`: the screen turns on when the activity
///   resumes (e.g. from a notification tap).
/// - A `SCREEN_BRIGHT_WAKE_LOCK | ACQUIRE_CAUSES_WAKEUP` `PowerManager` wake
///   lock is acquired so the screen stays on while the app is visible over the
///   lock screen — useful at the gym to keep the session visible without
///   touching the phone.
///
/// When `active` is `false` all of the above are reverted and the wake lock is
/// released, restoring normal screen-timeout behaviour.
///
/// # Thread safety
/// This function may be called from any thread; no UI-thread marshalling is
/// required because it uses `PowerManager` wake locks instead of
/// `Window.addFlags`, and `setShowWhenLocked` / `setTurnScreenOn` are
/// thread-safe `Activity` methods.
#[cfg(target_os = "android")]
pub fn set_active_session_lock_screen(active: bool) {
    use jni::{
        objects::{JObject, JValue},
        JavaVM,
    };
    use ndk_context::android_context;

    let result = (|| -> Result<(), String> {
        let ctx = android_context();
        // SAFETY: the raw pointers come from the Android runtime and are valid
        // for the lifetime of the process.
        let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }
            .map_err(|e| format!("JavaVM::from_raw: {e}"))?;
        let mut env = vm
            .attach_current_thread()
            .map_err(|e| format!("attach_current_thread: {e}"))?;
        let activity = unsafe { JObject::from_raw(ctx.context() as jni::sys::jobject) };

        if active {
            // Only acquire once; skip if already held.
            let mut guard = SCREEN_WAKE_LOCK.lock().unwrap();
            if guard.is_none() {
                // Show the app over the lock screen and turn on the screen.
                env.call_method(
                    &activity,
                    "setShowWhenLocked",
                    "(Z)V",
                    &[JValue::Bool(1u8)],
                )
                .map_err(|e| format!("setShowWhenLocked(true): {e}"))?;
                env.call_method(
                    &activity,
                    "setTurnScreenOn",
                    "(Z)V",
                    &[JValue::Bool(1u8)],
                )
                .map_err(|e| format!("setTurnScreenOn(true): {e}"))?;

                // Acquire SCREEN_BRIGHT_WAKE_LOCK | ACQUIRE_CAUSES_WAKEUP.
                // SCREEN_BRIGHT_WAKE_LOCK (0x0a) keeps the screen bright.
                // ACQUIRE_CAUSES_WAKEUP (0x10000000) turns the screen on when
                // the lock is acquired, even if the screen was already off.
                // Both flags are deprecated since API 17 but remain functional
                // and are the correct choice here because they work from any
                // thread without needing Window/UI-thread access.
                let power_service = env
                    .get_static_field(
                        "android/content/Context",
                        "POWER_SERVICE",
                        "Ljava/lang/String;",
                    )
                    .map_err(|e| format!("get POWER_SERVICE: {e}"))?
                    .l()
                    .map_err(|e| format!("POWER_SERVICE obj: {e}"))?;
                let pm = env
                    .call_method(
                        &activity,
                        "getSystemService",
                        "(Ljava/lang/String;)Ljava/lang/Object;",
                        &[(&power_service).into()],
                    )
                    .map_err(|e| format!("getSystemService: {e}"))?
                    .l()
                    .map_err(|e| format!("PowerManager obj: {e}"))?;

                #[allow(clippy::cast_possible_wrap)]
                let flags: i32 = 0x0000_000a_i32 | (0x1000_0000_u32 as i32);
                let tag = env
                    .new_string("logout:screenwake")
                    .map_err(|e| format!("new_string: {e}"))?;
                let wake_lock = env
                    .call_method(
                        &pm,
                        "newWakeLock",
                        "(ILjava/lang/String;)Landroid/os/PowerManager$WakeLock;",
                        &[JValue::Int(flags), (&tag).into()],
                    )
                    .map_err(|e| format!("newWakeLock: {e}"))?
                    .l()
                    .map_err(|e| format!("WakeLock obj: {e}"))?;

                // setReferenceCounted(false): acquire/release are not counted,
                // so a single release() always frees the lock.
                env.call_method(
                    &wake_lock,
                    "setReferenceCounted",
                    "(Z)V",
                    &[JValue::Bool(0u8)],
                )
                .map_err(|e| format!("setReferenceCounted: {e}"))?;

                env.call_method(&wake_lock, "acquire", "()V", &[])
                    .map_err(|e| format!("acquire: {e}"))?;

                let global = env
                    .new_global_ref(&wake_lock)
                    .map_err(|e| format!("new_global_ref: {e}"))?;
                *guard = Some(global);
            }
        } else {
            let mut guard = SCREEN_WAKE_LOCK.lock().unwrap();
            if let Some(global) = guard.take() {
                // Release the screen wake lock.
                let wake_lock = global.as_obj();
                let _ = env.call_method(wake_lock, "release", "()V", &[]);
            }
            // Restore normal lock-screen behaviour.
            env.call_method(
                &activity,
                "setShowWhenLocked",
                "(Z)V",
                &[JValue::Bool(0u8)],
            )
            .map_err(|e| format!("setShowWhenLocked(false): {e}"))?;
            env.call_method(
                &activity,
                "setTurnScreenOn",
                "(Z)V",
                &[JValue::Bool(0u8)],
            )
            .map_err(|e| format!("setTurnScreenOn(false): {e}"))?;
        }

        Ok(())
    })();

    match result {
        Ok(()) => log::info!("Android lock-screen mode active={active}"),
        Err(e) => log::warn!("Failed to set Android lock-screen mode: {e}"),
    }
}

/// No-op shim for non-Android targets.
#[cfg(not(target_os = "android"))]
pub fn set_active_session_lock_screen(_active: bool) {}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn enable_wake_lock_noop_on_native() {
        enable_wake_lock();
    }
}
