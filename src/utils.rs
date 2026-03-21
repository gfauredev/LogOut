/// Default base URL for the exercise database, served as a static website with
/// proper CORS headers (`Access-Control-Allow-Origin: *`).
pub(crate) const EXERCISE_DB_BASE_URL: &str = "https://gfauredev.github.io/free-exercise-db/";

/// Base URL for exercise images, which are served from raw GitHub content
/// (not included in release assets).
pub(crate) const EXERCISE_IMAGES_BASE_URL: &str =
    "https://raw.githubusercontent.com/gfauredev/free-exercise-db/main/";

/// localStorage / config-file key used to store a user-configured exercise database URL.
pub(crate) const EXERCISE_DB_URL_STORAGE_KEY: &str = "exercise_db_url";

/// Normalise a user-supplied exercise database URL so it is safe to use as a
/// base URL for building file paths.
///
/// Applies the following transformations:
/// - Leading/trailing whitespace is stripped.
/// - If no scheme (`http://` / `https://`) is present, `https://` is prepended.
/// - If no trailing `/` is present, one is appended.
///
/// An empty string is returned unchanged (it signals "reset to default").
#[must_use]
pub fn normalize_db_url(url: &str) -> String {
    let url = url.trim();
    if url.is_empty() {
        return String::new();
    }
    let url = if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("https://{url}")
    };
    if url.ends_with('/') {
        url
    } else {
        format!("{url}/")
    }
}

/// Returns the effective exercise database base URL.
/// On WASM, checks localStorage for a user-configured URL first.
/// On native, checks the app config file.
/// Falls back to [`EXERCISE_DB_BASE_URL`] if not set.
#[must_use]
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
    #[cfg(not(target_arch = "wasm32"))]
    {
        use crate::services::storage::native_storage;
        if let Some(url) = native_storage::get_config_value(EXERCISE_DB_URL_STORAGE_KEY) {
            if !url.is_empty() {
                return url;
            }
        }
    }
    EXERCISE_DB_BASE_URL.to_string()
}

/// Returns the base URL used for exercise images.
///
/// When the user has configured a custom database URL the same origin is used
/// for images (for self-hosted setups).  Otherwise the images are fetched from
/// the raw GitHub source repository, because images are **not** included in
/// the release assets.
#[must_use]
pub fn get_exercise_images_base_url() -> String {
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
    #[cfg(not(target_arch = "wasm32"))]
    {
        use crate::services::storage::native_storage;
        if let Some(url) = native_storage::get_config_value(EXERCISE_DB_URL_STORAGE_KEY) {
            if !url.is_empty() {
                return url;
            }
        }
    }
    EXERCISE_IMAGES_BASE_URL.to_string()
}

// ── Deep link parsing ─────────────────────────────────────────────────────────

/// A pending exercise entry parsed from a deep-link session-creation URL.
///
/// `weight_hg` is stored as hectograms (multiply kg × 10); `reps` is raw.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionExerciseEntry {
    /// Exercise ID as it appears in the exercise database.
    pub exercise_id: String,
    /// Weight in hectograms (`kg × 10`), or `None` if not specified.
    pub weight_hg: Option<u32>,
    /// Repetitions performed, or `None` if not specified.
    pub reps: Option<u32>,
}

/// Actions that can be triggered via a `logworkout://` deep link.
#[derive(Debug, Clone, PartialEq)]
pub enum DeepLinkAction {
    /// Navigate to the given route path (e.g. `"/"`, `"/exercises"`).
    Navigate(String),
    /// Navigate to exercises with an optional pre-filled search query.
    SearchExercises(String),
    /// Store a new exercise-database URL and trigger a reload.
    SetDbUrl(String),
    /// Create a completed past session containing the listed exercises.
    ///
    /// Exercise metadata is looked up from the loaded exercise list, so this
    /// action is deferred until exercises are available.
    CreateSession(Vec<SessionExerciseEntry>),
    /// Start a new active session with the given exercise IDs pre-queued.
    StartSession(Vec<String>),
}

/// Parse a `logworkout://` URL into a [`DeepLinkAction`], returning `None` for
/// unrecognised or malformed links.
///
/// Supported schemes:
/// - `logworkout://home`
/// - `logworkout://exercises[?q=<query>]`
/// - `logworkout://analytics`
/// - `logworkout://credits[?db_url=<url>]`
/// - `logworkout://more[?db_url=<url>]`
/// - `logworkout://exercise/add`
/// - `logworkout://session/start[?exercises=<id>,<id>,…]`
/// - `logworkout://session/create?exercises=<id>:<kg>:<reps>,…`
#[must_use]
pub fn parse_deep_link(url: &str) -> Option<DeepLinkAction> {
    let rest = url.strip_prefix("logworkout://")?;
    let (path, query) = rest.split_once('?').unwrap_or((rest, ""));
    parse_deep_link_path(path, query)
}

/// Parse web URL query parameters produced by a `?deeplink=logworkout://…` param
/// or the shorthand `?dl_*` flat params.  Returns `None` when no recognised deep
/// link parameter is present.
#[cfg(target_arch = "wasm32")]
pub fn parse_web_deep_link() -> Option<DeepLinkAction> {
    let window = web_sys::window()?;
    let location = window.location();
    let search = location.search().ok()?;
    let query = search.trim_start_matches('?');

    // ── Full logworkout:// link encoded as ?deeplink=… ────────────────────
    if let Some(dl) = get_query_param(query, "deeplink") {
        if let Some(action) = parse_deep_link(&dl) {
            return Some(action);
        }
    }

    // ── Flat shorthand params (easier to type in YAML test files) ─────────
    if let Some(url) = get_query_param(query, "dl_db_url") {
        return Some(DeepLinkAction::SetDbUrl(url));
    }
    if let Some(q) = get_query_param(query, "dl_q") {
        return Some(DeepLinkAction::SearchExercises(q));
    }
    if let Some(nav) = get_query_param(query, "dl_navigate") {
        return Some(DeepLinkAction::Navigate(route_name_to_path(&nav)));
    }
    if let Some(exercises) = get_query_param(query, "dl_session") {
        let entries = parse_session_exercises(&exercises);
        return Some(DeepLinkAction::CreateSession(entries));
    }
    if let Some(exercises) = get_query_param(query, "dl_start") {
        let ids = exercises
            .split(',')
            .filter(|s| !s.is_empty())
            .map(std::string::ToString::to_string)
            .collect();
        return Some(DeepLinkAction::StartSession(ids));
    }

    None
}

/// Internal: convert a path + query string from a logworkout:// URL into an action.
fn parse_deep_link_path(path: &str, query: &str) -> Option<DeepLinkAction> {
    match path {
        "home" => Some(DeepLinkAction::Navigate("/".to_string())),
        "exercises" => {
            if let Some(q) = get_query_param(query, "q") {
                Some(DeepLinkAction::SearchExercises(q))
            } else {
                Some(DeepLinkAction::Navigate("/exercises".to_string()))
            }
        }
        "analytics" => Some(DeepLinkAction::Navigate("/analytics".to_string())),
        "credits" | "more" => {
            if let Some(url) = get_query_param(query, "db_url") {
                Some(DeepLinkAction::SetDbUrl(url))
            } else {
                Some(DeepLinkAction::Navigate("/more".to_string()))
            }
        }
        "exercise/add" => Some(DeepLinkAction::Navigate("/add-exercise".to_string())),
        "session/start" => {
            let ids: Vec<String> = get_query_param(query, "exercises")
                .unwrap_or_default()
                .split(',')
                .filter(|s| !s.is_empty())
                .map(std::string::ToString::to_string)
                .collect();
            Some(DeepLinkAction::StartSession(ids))
        }
        "session/create" => {
            let exercises_str = get_query_param(query, "exercises")?;
            Some(DeepLinkAction::CreateSession(parse_session_exercises(
                &exercises_str,
            )))
        }
        _ => None,
    }
}

/// Parse a comma-separated list of `<id>:<weight_kg>:<reps>` exercise entries.
/// Any field may be omitted or set to `-` to indicate "not specified".
///
/// Example: `"Bench_Press:80:10,Squat:60:6"`
#[must_use]
pub fn parse_session_exercises(s: &str) -> Vec<SessionExerciseEntry> {
    s.split(',')
        .filter(|e| !e.is_empty())
        .map(|entry| {
            let mut parts = entry.split(':');
            let exercise_id = parts.next().unwrap_or("").to_string();
            let weight_hg = parts.next().and_then(|w| {
                if w.is_empty() || w == "-" {
                    None
                } else {
                    w.parse::<f64>().ok().and_then(|kg| {
                        let hg = (kg * 10.0).round();
                        // Ensure the value fits in u32 (up to ~429,496,729.5 kg)
                        if (0.0..=f64::from(u32::MAX)).contains(&hg) {
                            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                            Some(hg as u32)
                        } else {
                            None
                        }
                    })
                }
            });
            let reps = parts.next().and_then(|r| {
                if r.is_empty() || r == "-" {
                    None
                } else {
                    r.parse::<u32>().ok()
                }
            });
            SessionExerciseEntry {
                exercise_id,
                weight_hg,
                reps,
            }
        })
        .collect()
}

/// Look up a single parameter value from a URL query string.
#[must_use]
pub fn get_query_param(query: &str, name: &str) -> Option<String> {
    query.split('&').find_map(|pair| {
        let (k, v) = pair.split_once('=')?;
        if k == name {
            // Basic percent-decoding for common characters
            Some(percent_decode(v))
        } else {
            None
        }
    })
}

/// Minimal percent-decoder that handles both ASCII and multi-byte UTF-8 sequences.
///
/// Percent-encoded sequences are collected as raw bytes and decoded together so
/// that multi-byte UTF-8 characters (e.g. `%C3%A9` → `é`) are handled correctly.
/// `+` is treated as a space (application/x-www-form-urlencoded convention).
fn percent_decode(s: &str) -> String {
    let mut bytes: Vec<u8> = Vec::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let h1 = chars.next().unwrap_or('0');
            let h2 = chars.next().unwrap_or('0');
            let hex = format!("{h1}{h2}");
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                bytes.push(byte);
            } else {
                bytes.push(b'%');
                let mut buf = [0u8; 4];
                bytes.extend_from_slice(h1.encode_utf8(&mut buf).as_bytes());
                bytes.extend_from_slice(h2.encode_utf8(&mut buf).as_bytes());
            }
        } else if c == '+' {
            bytes.push(b' ');
        } else {
            let mut buf = [0u8; 4];
            bytes.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
        }
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Map a human-readable route name (as used in `?dl_navigate=…`) to the
/// corresponding URL path.
#[cfg(target_arch = "wasm32")]
fn route_name_to_path(name: &str) -> String {
    match name {
        "home" | "/" => "/".to_string(),
        "exercises" => "/exercises".to_string(),
        "analytics" => "/analytics".to_string(),
        "credits" | "more" => "/more".to_string(),
        "add-exercise" | "add_exercise" => "/add-exercise".to_string(),
        other => format!("/{other}"),
    }
}

/// Format a session timestamp as a human-readable relative date string.
#[must_use]
pub fn format_session_date(timestamp: u64) -> String {
    let days_ago = days_since(timestamp);
    match days_ago {
        0 => "Today".to_string(),
        1 => "Yesterday".to_string(),
        n => format!("{n} days ago"),
    }
}

/// Returns the number of elapsed calendar days between the local midnight of
/// `timestamp`'s day and the local midnight of today, using system’s local TZ
fn days_since(timestamp: u64) -> i64 {
    use time::OffsetDateTime;
    #[cfg(not(target_arch = "wasm32"))]
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    #[cfg(target_arch = "wasm32")]
    let now = {
        let millis = js_sys::Date::now();
        // js_sys::Date::get_timezone_offset() returns minutes WEST of UTC
        // (positive for UTC-N, negative for UTC+N).  time::UtcOffset uses
        // seconds EAST of UTC, so we negate and convert.
        let tz_offset_secs =
            -(js_sys::Date::new_0().get_timezone_offset() as i32) * 60;
        let offset = time::UtcOffset::from_whole_seconds(tz_offset_secs)
            .unwrap_or(time::UtcOffset::UTC);
        OffsetDateTime::from_unix_timestamp_nanos((millis as i128) * 1_000_000)
            .unwrap_or(OffsetDateTime::now_utc())
            .to_offset(offset)
    };
    let offset = now.offset();
    let ts_dt = OffsetDateTime::from_unix_timestamp(timestamp.cast_signed())
        .unwrap_or(OffsetDateTime::UNIX_EPOCH)
        .to_offset(offset);
    let now_date = now.date();
    let ts_date = ts_dt.date();
    (now_date - ts_date).whole_days()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn today_midnight_local_secs() -> u64 {
        use time::OffsetDateTime;
        let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
        // Build a datetime at local midnight for today and convert back to unix seconds.
        let midnight = now.replace_time(time::Time::MIDNIGHT);
        midnight.unix_timestamp().max(0).cast_unsigned()
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
            let _g = crate::services::storage::native_storage::test_lock();
            let url = super::get_exercise_db_url();
            assert_eq!(url, super::EXERCISE_DB_BASE_URL);
        }
    }

    #[test]
    fn exercise_db_url_storage_key_is_stable() {
        // The localStorage key should not change accidentally.
        assert_eq!(super::EXERCISE_DB_URL_STORAGE_KEY, "exercise_db_url");
    }

    #[test]
    fn exercise_db_base_url_is_github_pages() {
        // Default URL must point to the GitHub Pages static website (CORS-friendly).
        assert!(
            super::EXERCISE_DB_BASE_URL.contains("github.io"),
            "EXERCISE_DB_BASE_URL should be a GitHub Pages URL, got: {}",
            super::EXERCISE_DB_BASE_URL
        );
    }

    #[test]
    fn exercise_images_base_url_is_raw_github() {
        // Images come from the raw GitHub source, not from release assets.
        assert!(
            super::EXERCISE_IMAGES_BASE_URL.contains("raw.githubusercontent.com"),
            "EXERCISE_IMAGES_BASE_URL should be a raw.githubusercontent.com URL, got: {}",
            super::EXERCISE_IMAGES_BASE_URL
        );
    }

    #[test]
    fn get_exercise_images_base_url_returns_images_url_by_default() {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _g = crate::services::storage::native_storage::test_lock();
            let url = super::get_exercise_images_base_url();
            assert_eq!(url, super::EXERCISE_IMAGES_BASE_URL);
        }
    }

    // ── Deep link parsing ─────────────────────────────────────────────────────

    #[test]
    fn parse_deep_link_home() {
        assert_eq!(
            super::parse_deep_link("logworkout://home"),
            Some(DeepLinkAction::Navigate("/".to_string()))
        );
    }

    #[test]
    fn parse_deep_link_exercises_no_query() {
        assert_eq!(
            super::parse_deep_link("logworkout://exercises"),
            Some(DeepLinkAction::Navigate("/exercises".to_string()))
        );
    }

    #[test]
    fn parse_deep_link_exercises_with_query() {
        assert_eq!(
            super::parse_deep_link("logworkout://exercises?q=bench+press"),
            Some(DeepLinkAction::SearchExercises("bench press".to_string()))
        );
    }

    #[test]
    fn parse_deep_link_analytics() {
        assert_eq!(
            super::parse_deep_link("logworkout://analytics"),
            Some(DeepLinkAction::Navigate("/analytics".to_string()))
        );
    }

    #[test]
    fn parse_deep_link_credits_no_url() {
        assert_eq!(
            super::parse_deep_link("logworkout://credits"),
            Some(DeepLinkAction::Navigate("/more".to_string()))
        );
    }

    #[test]
    fn parse_deep_link_more_no_url() {
        assert_eq!(
            super::parse_deep_link("logworkout://more"),
            Some(DeepLinkAction::Navigate("/more".to_string()))
        );
    }

    #[test]
    fn parse_deep_link_credits_with_db_url() {
        assert_eq!(
            super::parse_deep_link("logworkout://credits?db_url=http://localhost:8080"),
            Some(DeepLinkAction::SetDbUrl(
                "http://localhost:8080".to_string()
            ))
        );
    }

    #[test]
    fn parse_deep_link_add_exercise() {
        assert_eq!(
            super::parse_deep_link("logworkout://exercise/add"),
            Some(DeepLinkAction::Navigate("/add-exercise".to_string()))
        );
    }

    #[test]
    fn parse_deep_link_session_start_no_exercises() {
        assert_eq!(
            super::parse_deep_link("logworkout://session/start"),
            Some(DeepLinkAction::StartSession(vec![]))
        );
    }

    #[test]
    fn parse_deep_link_session_start_with_exercises() {
        assert_eq!(
            super::parse_deep_link(
                "logworkout://session/start?exercises=Bench_Press,Barbell_Squat"
            ),
            Some(DeepLinkAction::StartSession(vec![
                "Bench_Press".to_string(),
                "Barbell_Squat".to_string(),
            ]))
        );
    }

    #[test]
    fn parse_deep_link_session_create() {
        assert_eq!(
            super::parse_deep_link(
                "logworkout://session/create?exercises=Bench_Press:80:10,Barbell_Squat:60:6"
            ),
            Some(DeepLinkAction::CreateSession(vec![
                SessionExerciseEntry {
                    exercise_id: "Bench_Press".to_string(),
                    weight_hg: Some(800),
                    reps: Some(10),
                },
                SessionExerciseEntry {
                    exercise_id: "Barbell_Squat".to_string(),
                    weight_hg: Some(600),
                    reps: Some(6),
                },
            ]))
        );
    }

    #[test]
    fn parse_deep_link_session_create_no_weight() {
        let result = super::parse_deep_link("logworkout://session/create?exercises=Run:-:- ");
        let Some(DeepLinkAction::CreateSession(entries)) = result else {
            panic!("expected CreateSession")
        };
        assert_eq!(entries[0].weight_hg, None);
        assert_eq!(entries[0].reps, None);
    }

    #[test]
    fn parse_deep_link_unknown_returns_none() {
        assert_eq!(super::parse_deep_link("logworkout://unknown/path"), None);
    }

    #[test]
    fn parse_deep_link_wrong_scheme_returns_none() {
        assert_eq!(super::parse_deep_link("https://example.com"), None);
    }

    #[test]
    fn get_query_param_basic() {
        assert_eq!(
            super::get_query_param("foo=bar&baz=qux", "foo"),
            Some("bar".to_string())
        );
        assert_eq!(
            super::get_query_param("foo=bar&baz=qux", "baz"),
            Some("qux".to_string())
        );
        assert_eq!(super::get_query_param("foo=bar&baz=qux", "missing"), None);
    }

    #[test]
    fn percent_decode_handles_common_chars() {
        assert_eq!(
            super::percent_decode("hello%20world"),
            "hello world".to_string()
        );
        assert_eq!(super::percent_decode("a+b"), "a b".to_string());
        assert_eq!(
            super::percent_decode("http%3A%2F%2Flocalhost%3A8080"),
            "http://localhost:8080".to_string()
        );
    }

    #[test]
    fn percent_decode_handles_multibyte_utf8() {
        // %C3%A9 is the UTF-8 encoding of 'é'
        assert_eq!(super::percent_decode("%C3%A9"), "é".to_string());
    }

    #[test]
    fn parse_session_exercises_weight_rounding() {
        // 77.5 kg → 775 hg (rounded)
        let entries = super::parse_session_exercises("Bench:77.5:10");
        assert_eq!(entries[0].weight_hg, Some(775));
        assert_eq!(entries[0].reps, Some(10));
    }

    // ── normalize_db_url ─────────────────────────────────────────────────────

    #[test]
    fn normalize_db_url_empty_returns_empty() {
        assert_eq!(super::normalize_db_url(""), "");
        assert_eq!(super::normalize_db_url("   "), "");
    }

    #[test]
    fn normalize_db_url_adds_trailing_slash() {
        assert_eq!(
            super::normalize_db_url("https://example.com"),
            "https://example.com/"
        );
        assert_eq!(
            super::normalize_db_url("http://localhost:8080"),
            "http://localhost:8080/"
        );
    }

    #[test]
    fn normalize_db_url_keeps_existing_trailing_slash() {
        assert_eq!(
            super::normalize_db_url("https://example.com/"),
            "https://example.com/"
        );
    }

    #[test]
    fn normalize_db_url_adds_https_scheme() {
        assert_eq!(
            super::normalize_db_url("example.com"),
            "https://example.com/"
        );
        assert_eq!(
            super::normalize_db_url("localhost:8080"),
            "https://localhost:8080/"
        );
    }

    #[test]
    fn normalize_db_url_keeps_http_scheme() {
        assert_eq!(
            super::normalize_db_url("http://localhost:8080"),
            "http://localhost:8080/"
        );
    }

    #[test]
    fn normalize_db_url_trims_whitespace() {
        assert_eq!(
            super::normalize_db_url("  https://example.com  "),
            "https://example.com/"
        );
    }
}
