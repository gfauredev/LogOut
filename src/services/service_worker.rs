/// Service Worker registration for offline image caching
///
/// This module handles the registration of the Service Worker (sw.js) which provides
/// offline caching for exercise images loaded from the GitHub CDN.
///
/// ## Platform Compatibility
///
/// **Web Platform (with JavaScript):**
/// - Service Worker is registered to cache images for offline use
/// - Uses browser Cache API through sw.js JavaScript file
/// - Progressive enhancement: app works without it
///
/// **Blitz/Native Platforms (no JavaScript):**
/// - Service Worker is disabled (requires JavaScript engine)
/// - App runs without offline caching
/// - Images are fetched from network as needed
/// - Future: Could implement native caching using platform-specific APIs
///
/// ## Feature Flags
///
/// The Service Worker is only enabled when both conditions are met:
/// 1. Compiled for WASM (`target_arch = "wasm32"`)
/// 2. `web-platform` feature is enabled (default)
///
/// To build for Blitz without Service Worker:
/// ```bash
/// cargo build --no-default-features
/// ```

#[cfg(all(target_arch = "wasm32", feature = "web-platform"))]
pub fn register_service_worker() {
    use web_sys::window;

    if let Some(window) = window() {
        let navigator = window.navigator();
        let sw_container = navigator.service_worker();

        // Register the service worker
        // Use relative path for GitHub Pages compatibility
        let registration = sw_container.register("./sw.js");

        // Handle the registration promise asynchronously
        // Note: spawn_local failure is acceptable here as service worker registration
        // is a progressive enhancement feature - the app works without it
        let _ = wasm_bindgen_futures::spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(registration).await {
                Ok(_) => {
                    log::info!("Service Worker registered successfully for offline image caching");
                }
                Err(err) => {
                    log::error!("Service Worker registration failed: {:?}", err);
                    log::warn!("App will continue to work, but without offline image caching");
                }
            }
        });
    }
}

#[cfg(not(all(target_arch = "wasm32", feature = "web-platform")))]
pub fn register_service_worker() {
    // No-op on non-web platforms (Blitz, native desktop, etc.)
    // The app works perfectly fine without offline caching
    log::info!("Service Worker disabled: running on non-web platform (Blitz-compatible mode)");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_service_worker_noop_on_native() {
        // Verifies that calling register_service_worker on a non-wasm target
        // does not panic (the function is a no-op in this configuration).
        register_service_worker();
    }
}
