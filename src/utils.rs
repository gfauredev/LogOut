/// Format a session timestamp as a human-readable relative date string.
pub fn format_session_date(timestamp: u64) -> String {
    #[cfg(target_arch = "wasm32")]
    let current_time = js_sys::Date::now() / 1000.0;
    #[cfg(not(target_arch = "wasm32"))]
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as f64;

    let days_ago = ((current_time - timestamp as f64) / 86400.0) as i64;
    match days_ago {
        0 => "Today".to_string(),
        1 => "Yesterday".to_string(),
        n => format!("{} days ago", n),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_session_date_today() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        assert_eq!(format_session_date(now), "Today");
    }

    #[test]
    fn format_session_date_yesterday() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let yesterday = now - 86400;
        assert_eq!(format_session_date(yesterday), "Yesterday");
    }

    #[test]
    fn format_session_date_days_ago() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let three_days_ago = now - 86400 * 3;
        assert_eq!(format_session_date(three_days_ago), "3 days ago");
    }
}
