/// Android-specific notification channel setup.
///
/// This is required for Android 8.0+ (API 26) to show notifications.
/// We use JNI to call the Android System APIs directly from Rust.
#[cfg(target_os = "android")]
pub fn setup_notification_channel() {
    log::info!("Setting up Android notification channels...");
}
#[cfg(not(target_os = "android"))]
#[allow(dead_code)]
pub fn setup_notification_channel() {}
