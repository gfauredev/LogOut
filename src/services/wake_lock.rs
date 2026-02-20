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

    // Re-acquire the lock when the page becomes visible again.
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
                if document.visibility_state()
                    == web_sys::VisibilityState::Visible
                {
                    let _ = request_wake_lock().await;
                }
            });
        });

        let _ = document.add_event_listener_with_callback(
            "visibilitychange",
            closure.as_ref().unchecked_ref(),
        );
        // Intentionally leak the closure so it lives for the page lifetime.
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

    // Check that navigator.wakeLock exists (not all browsers support it yet).
    let wake_lock = Reflect::get(&navigator, &JsValue::from_str("wakeLock"))
        .map_err(|e| format!("{:?}", e))?;
    if wake_lock.is_undefined() || wake_lock.is_null() {
        return Ok(()); // API not supported – silently skip
    }

    let request_fn = Reflect::get(&wake_lock, &JsValue::from_str("request"))
        .map_err(|e| format!("{:?}", e))?;
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
    // No-op on non-web platforms.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enable_wake_lock_noop_on_native() {
        // Should not panic on non-wasm targets.
        enable_wake_lock();
    }
}
