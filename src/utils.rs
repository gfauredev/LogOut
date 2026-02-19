/// Format a session timestamp as a human-readable relative date string.
pub fn format_session_date(timestamp: u64) -> String {
    let days_ago = days_since(timestamp);
    match days_ago {
        0 => "Today".to_string(),
        1 => "Yesterday".to_string(),
        n => format!("{} days ago", n),
    }
}

/// Returns the number of elapsed calendar days between the local midnight of
/// `timestamp`'s day and the local midnight of today.
fn days_since(timestamp: u64) -> i64 {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsValue;

        // Build a Date for the session timestamp and reset to local midnight.
        let ts_ms = (timestamp as f64) * 1000.0;
        let session_date = js_sys::Date::new(&JsValue::from_f64(ts_ms));
        session_date.set_hours(0);
        session_date.set_minutes(0);
        session_date.set_seconds(0);
        session_date.set_milliseconds(0);

        // Build a Date for today and reset to local midnight.
        let today_date = js_sys::Date::new_0();
        today_date.set_hours(0);
        today_date.set_minutes(0);
        today_date.set_seconds(0);
        today_date.set_milliseconds(0);

        let diff_ms = today_date.get_time() - session_date.get_time();
        (diff_ms / 86_400_000.0) as i64
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // On non-WASM targets (tests) use UTC day numbers as an approximation.
        let current_day = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            / 86400;
        let ts_day = timestamp / 86400;
        (current_day as i64) - (ts_day as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn today_midnight_utc_secs() -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        // Truncate to start of UTC day
        (now / 86400) * 86400
    }

    #[test]
    fn format_session_date_today() {
        // A timestamp within today's UTC day
        let ts = today_midnight_utc_secs() + 3600; // 1h into today
        assert_eq!(format_session_date(ts), "Today");
    }

    #[test]
    fn format_session_date_yesterday() {
        let ts = today_midnight_utc_secs() - 1; // 1 second before today's midnight
        assert_eq!(format_session_date(ts), "Yesterday");
    }

    #[test]
    fn format_session_date_days_ago() {
        let ts = today_midnight_utc_secs() - 86400 * 3; // 3 days before today
        assert_eq!(format_session_date(ts), "3 days ago");
    }

    #[test]
    fn format_session_date_beginning_of_today() {
        let ts = today_midnight_utc_secs();
        assert_eq!(format_session_date(ts), "Today");
    }

    #[test]
    fn format_session_date_end_of_yesterday() {
        let ts = today_midnight_utc_secs() - 1;
        assert_eq!(format_session_date(ts), "Yesterday");
    }
}
