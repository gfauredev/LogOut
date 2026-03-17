//! Data models for the `LogOut` application.
//!
//! This module defines the core entities (Exercises, Logs, Sessions) and their
//! supporting types (Enums, Units). All types are serialisable to JSON for
//! persistence in IndexedDB or SQLite.

pub mod enums;
pub mod exercise;
pub mod log;
pub mod session;
pub mod units;

pub use enums::*;
pub use exercise::*;
pub use log::*;
pub use session::*;
pub use units::*;

/// Current schema version for the workout-session data format.
pub const DATA_VERSION: u32 = 1;

/// Returns the current Unix timestamp in seconds.
/// Cross-platform: uses `js_sys` on Web and `SystemTime` on Native.
#[must_use]
pub fn get_current_timestamp() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::now() / 1000.0) as u64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// Helper for rendering timestamps as `HH:MM` or `HH:MM:SS`.
#[must_use]
pub fn format_time(seconds: u64) -> String {
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;
    let s = seconds % 60;
    if h > 0 {
        format!("{h:02}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

/// Shared logic for returning the CSS class and icon for an exercise type tag.
pub(crate) fn exercise_type_tag(
    category: Category,
    force: Option<Force>,
) -> (&'static str, &'static str) {
    if category == Category::Cardio {
        ("tag-cardio", "🏃")
    } else if force.is_some_and(Force::has_reps) {
        ("tag-strength", "💪")
    } else {
        ("tag-static", "⏱️")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_time_boundary_values() {
        assert_eq!(format_time(1), "00:01");
        assert_eq!(format_time(59), "00:59");
        assert_eq!(format_time(60), "01:00");
        assert_eq!(format_time(3599), "59:59");
        assert_eq!(format_time(3600), "01:00:00");
        assert_eq!(format_time(86399), "23:59:59");
    }

    #[test]
    fn get_current_timestamp_returns_reasonable_value() {
        let ts = get_current_timestamp();
        // Greater than March 2024
        assert!(ts > 1_710_000_000);
    }
}
