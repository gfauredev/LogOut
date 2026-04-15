//! Data models for the `LogOut` application.
//!
//! This module defines the core entities (Exercises, Logs, Sessions) and their
//! supporting types (Enums, Units). All types are serialisable to JSON for
//! persistence in `IndexedDB` or `SQLite`.
pub mod analytics;
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
/// Like [`format_time`] but accepts a signed integer so a negative countdown
/// can be rendered with a leading `"-"`.
#[must_use]
pub fn format_time_i64(seconds: i64) -> String {
    if seconds < 0 {
        format!("-{}", format_time(seconds.unsigned_abs()))
    } else {
        format_time(seconds.cast_unsigned())
    }
}
/// Shared logic for returning the CSS class and icon for an exercise type tag.
pub(crate) fn exercise_type_tag(
    category: Category,
    force: Option<Force>,
) -> (&'static str, &'static str) {
    match (category, force.is_some_and(Force::has_reps)) {
        (Category::Cardio, _) => ("tag-cardio", "🏃"),
        (_, true) => ("tag-strength", "💪"),
        _ => ("tag-static", "⏱️"),
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
        assert!(ts > 1_710_000_000);
    }
    #[test]
    fn format_time_i64_positive_delegates_to_format_time() {
        assert_eq!(format_time_i64(0), "00:00");
        assert_eq!(format_time_i64(90), "01:30");
        assert_eq!(format_time_i64(3661), "01:01:01");
    }
    #[test]
    fn format_time_i64_negative_prefixes_minus() {
        assert_eq!(format_time_i64(-1), "-00:01");
        assert_eq!(format_time_i64(-90), "-01:30");
    }
    #[test]
    fn exercise_type_tag_cardio_ignores_force() {
        assert_eq!(
            exercise_type_tag(Category::Cardio, None),
            ("tag-cardio", "🏃"),
        );
        assert_eq!(
            exercise_type_tag(Category::Cardio, Some(Force::Push)),
            ("tag-cardio", "🏃"),
        );
    }
    #[test]
    fn exercise_type_tag_strength_push_pull() {
        assert_eq!(
            exercise_type_tag(Category::Strength, Some(Force::Push)),
            ("tag-strength", "💪"),
        );
        assert_eq!(
            exercise_type_tag(Category::Strength, Some(Force::Pull)),
            ("tag-strength", "💪"),
        );
    }
    #[test]
    fn exercise_type_tag_static_force_or_no_force() {
        assert_eq!(
            exercise_type_tag(Category::Strength, Some(Force::Static)),
            ("tag-static", "⏱️"),
        );
        assert_eq!(
            exercise_type_tag(Category::Stretching, None),
            ("tag-static", "⏱️"),
        );
    }
}
