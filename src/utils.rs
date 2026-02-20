/// Base URL for the exercise database fork (single source of truth).
/// All exercise data (JSON catalog, images) is served from this origin.
pub(crate) const EXERCISE_DB_BASE_URL: &str =
    "https://raw.githubusercontent.com/gfauredev/free-exercise-db/main/";

/// localStorage key used to store a user-configured exercise database URL.
pub(crate) const EXERCISE_DB_URL_STORAGE_KEY: &str = "exercise_db_url";

/// Returns the effective exercise database base URL.
/// On WASM, checks localStorage for a user-configured URL first.
/// Falls back to [`EXERCISE_DB_BASE_URL`] if not set.
pub fn get_exercise_db_url() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(url)) = storage.get_item(EXERCISE_DB_URL_STORAGE_KEY) {
                    if !url.is_empty() {
                        return url;
                    }
                }
            }
        }
    }
    EXERCISE_DB_BASE_URL.to_string()
}

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
        use time::{OffsetDateTime, UtcOffset};

        // Obtain the local UTC offset; fall back to UTC if it cannot be determined
        // (e.g. in sandboxed CI environments without /etc/localtime).
        let offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);

        let now = OffsetDateTime::now_utc().to_offset(offset);
        let ts_dt = OffsetDateTime::from_unix_timestamp(timestamp as i64)
            .unwrap_or(OffsetDateTime::UNIX_EPOCH)
            .to_offset(offset);

        let now_date = now.date();
        let ts_date = ts_dt.date();

        (now_date - ts_date).whole_days()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn today_midnight_local_secs() -> u64 {
        use time::{OffsetDateTime, UtcOffset};
        let offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
        let now = OffsetDateTime::now_utc().to_offset(offset);
        // Build a datetime at local midnight for today and convert back to unix seconds.
        let midnight = now.replace_time(time::Time::MIDNIGHT);
        midnight.unix_timestamp().max(0) as u64
    }

    #[test]
    fn format_session_date_today() {
        // A timestamp within today's local day
        let ts = today_midnight_local_secs() + 3600; // 1h into today
        assert_eq!(format_session_date(ts), "Today");
    }

    #[test]
    fn format_session_date_yesterday() {
        let ts = today_midnight_local_secs() - 1; // 1 second before today's midnight
        assert_eq!(format_session_date(ts), "Yesterday");
    }

    #[test]
    fn format_session_date_days_ago() {
        let ts = today_midnight_local_secs() - 86400 * 3; // 3 days before today
        assert_eq!(format_session_date(ts), "3 days ago");
    }

    #[test]
    fn format_session_date_beginning_of_today() {
        let ts = today_midnight_local_secs();
        assert_eq!(format_session_date(ts), "Today");
    }

    #[test]
    fn format_session_date_end_of_yesterday() {
        let ts = today_midnight_local_secs() - 1;
        assert_eq!(format_session_date(ts), "Yesterday");
    }

    #[test]
    fn format_session_date_two_days_ago() {
        let ts = today_midnight_local_secs() - 86400 * 2;
        assert_eq!(format_session_date(ts), "2 days ago");
    }

    #[test]
    fn days_since_uses_local_midnight_boundary() {
        // Verify that a timestamp at local midnight counts as "today",
        // not as "yesterday" (which UTC truncation would give for negative UTC offsets).
        let midnight = today_midnight_local_secs();
        let days = super::days_since(midnight);
        assert_eq!(days, 0, "local midnight should be day 0");
    }

    #[test]
    fn get_exercise_db_url_returns_default_on_native() {
        // On non-wasm targets, get_exercise_db_url() must return the default constant.
        #[cfg(not(target_arch = "wasm32"))]
        {
            let url = super::get_exercise_db_url();
            assert_eq!(url, super::EXERCISE_DB_BASE_URL);
        }
    }

    #[test]
    fn exercise_db_url_storage_key_is_stable() {
        // The localStorage key should not change accidentally.
        assert_eq!(super::EXERCISE_DB_URL_STORAGE_KEY, "exercise_db_url");
    }
}
