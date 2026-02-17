/// Service Worker registration for offline image caching
/// 
/// This module handles the registration of the Service Worker (sw.js) which provides
/// offline caching for exercise images loaded from the GitHub CDN.
/// 
/// Note: The actual Service Worker implementation (sw.js) must remain as a JavaScript file
/// because Service Workers run in a separate browser context and use Web APIs that are only
/// available in that context. However, the registration logic is implemented in Rust
/// following Dioxus best practices.

#[cfg(target_arch = "wasm32")]
pub fn register_service_worker() {
    use wasm_bindgen::JsValue;
    use web_sys::window;

    if let Some(window) = window() {
        if let Ok(navigator) = js_sys::Reflect::get(&window, &JsValue::from_str("navigator")) {
            if let Ok(service_worker) = js_sys::Reflect::get(&navigator, &JsValue::from_str("serviceWorker")) {
                // Check if service worker is supported
                if !service_worker.is_undefined() {
                    let navigator = window.navigator();
                    let sw_container = navigator.service_worker();
                    
                    // Register the service worker
                    let registration = sw_container.register("/sw.js");
                    
                    // Handle the registration promise
                    let _ = wasm_bindgen_futures::spawn_local(async move {
                        match wasm_bindgen_futures::JsFuture::from(registration).await {
                            Ok(_) => {
                                log::info!("Service Worker registered successfully");
                            }
                            Err(err) => {
                                log::error!("Service Worker registration failed: {:?}", err);
                            }
                        }
                    });
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn register_service_worker() {
    // No-op on non-WASM targets
}
