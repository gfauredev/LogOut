use crate::models::{
    Category, DbI18n, Equipment, Exercise, ExerciseI18n, ExerciseLangEntry, Force, Level, Muscle,
};
use dioxus::prelude::*;
use std::sync::Arc;
/// Newtype wrapper for the exercise-database signal so its `TypeId` is distinct
/// from the `Signal<Vec<Arc<Exercise>>>` used by `storage::provide_app_state` for
/// custom exercises.  Without this wrapper both `use_context_provider` calls in
/// `App` would share the same context slot, causing both signals to point at the
/// same `Signal<Vec<Arc<Exercise>>>` and leading to doubled counts, missing DB
/// exercises, and all exercises being treated as custom.
#[derive(Clone, Copy)]
pub(crate) struct AllExercisesSignal(pub(crate) Signal<Vec<Arc<Exercise>>>);
/// Number of seconds between automatic exercise database refreshes (7 days).
const EXERCISE_DB_REFRESH_INTERVAL_SECS: u64 = 7 * 24 * 60 * 60;
/// Storage key used to track when exercises were last downloaded
/// (localStorage on WASM, config file on native).
const LAST_FETCH_KEY: &str = "exercise_db_last_fetch";
/// Storage key used to persist the `ETag` returned by the last successful
/// `exercises.json` download (localStorage on WASM, config on native).
const EXERCISES_ETAG_KEY: &str = "exercise_db_etag";
/// Language codes for which per-exercise translation files are fetched and
/// merged into the exercise database on download.
const SUPPORTED_TRANSLATION_LANGS: &[&str] = &["fr"];
/// Returns the URL for the exercises JSON file.
/// Available on all platforms; `get_exercise_db_url()` handles per-platform config.
fn exercises_json_url() -> String {
    let base_url = crate::utils::get_exercise_db_url();
    format!("{base_url}exercises.json")
}
/// Returns the URL for a per-language exercise translation file.
/// For example, `exercises_lang_json_url("fr")` returns the URL for `exercises.fr.json`.
fn exercises_lang_json_url(lang: &str) -> String {
    let base_url = crate::utils::get_exercise_db_url();
    format!("{base_url}exercises.{lang}.json")
}
/// Returns the URL for the enum-translation file (`i18n.json`).
fn db_i18n_url() -> String {
    let base_url = crate::utils::get_exercise_db_url();
    format!("{base_url}i18n.json")
}
/// Provide the exercises signal in the Dioxus context.
/// On first launch, downloads exercises from the API and stores them in `IndexedDB`
/// (web) or a local file (native).  On subsequent launches, loads from cache.
pub use crate::services::exercise_loader::{provide_exercises, reload_exercises, use_exercises};
/// Returns true when the locally-cached exercise list is older than
/// [`EXERCISE_DB_REFRESH_INTERVAL_SECS`] or has never been fetched.
#[cfg(target_arch = "wasm32")]
pub(crate) fn is_refresh_due() -> bool {
    let Some(window) = web_sys::window() else {
        return true;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return true;
    };
    let Ok(Some(ts_str)) = storage.get_item(LAST_FETCH_KEY) else {
        return true;
    };
    let Ok(last_fetch) = ts_str.parse::<f64>() else {
        return true;
    };
    let now_secs = time::OffsetDateTime::now_utc()
        .unix_timestamp()
        .max(0)
        .cast_unsigned();
    let last_secs = last_fetch as u64;
    now_secs.saturating_sub(last_secs) >= EXERCISE_DB_REFRESH_INTERVAL_SECS
}
/// Returns true when the locally-cached exercise list is older than
/// [`EXERCISE_DB_REFRESH_INTERVAL_SECS`] or has never been fetched.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn is_refresh_due() -> bool {
    use crate::services::storage::native_storage;
    let last_fetch =
        native_storage::get_config_value(LAST_FETCH_KEY).and_then(|s| s.parse::<u64>().ok());
    let now = time::OffsetDateTime::now_utc()
        .unix_timestamp()
        .max(0)
        .cast_unsigned();
    is_refresh_due_for(now, last_fetch)
}
/// Pure helper: returns true when a refresh is due given the current time and the
/// last-fetch timestamp (both as Unix seconds).  Extracted for unit-testability.
#[cfg(not(target_arch = "wasm32"))]
fn is_refresh_due_for(now_secs: u64, last_fetch_secs: Option<u64>) -> bool {
    match last_fetch_secs {
        None => true,
        Some(last) => now_secs.saturating_sub(last) >= EXERCISE_DB_REFRESH_INTERVAL_SECS,
    }
}
/// Stores the current timestamp as the last exercise-fetch time.
#[cfg(target_arch = "wasm32")]
pub(crate) fn record_fetch_timestamp() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return;
    };
    let now = time::OffsetDateTime::now_utc().unix_timestamp().to_string();
    let _ = storage.set_item(LAST_FETCH_KEY, &now);
}
/// Stores the current timestamp as the last exercise-fetch time.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn record_fetch_timestamp() {
    use crate::services::storage::native_storage;
    let now = time::OffsetDateTime::now_utc().unix_timestamp().to_string();
    let _ = native_storage::set_config_value(LAST_FETCH_KEY, &now);
}
/// Clears the locally-cached fetch timestamp so that the exercise database is
/// re-downloaded from the current URL on the next application load.
#[cfg(target_arch = "wasm32")]
pub fn clear_fetch_cache() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return;
    };
    let _ = storage.remove_item(LAST_FETCH_KEY);
    let _ = storage.remove_item(EXERCISES_ETAG_KEY);
}
/// Clears the locally-cached fetch timestamp so that the exercise database is
/// re-downloaded from the current URL on the next application load.
#[cfg(not(target_arch = "wasm32"))]
pub fn clear_fetch_cache() {
    use crate::services::storage::native_storage;
    let _ = native_storage::remove_config_value(LAST_FETCH_KEY);
    let _ = native_storage::remove_config_value(EXERCISES_ETAG_KEY);
}
/// Returns the stored `ETag` for `exercises.json`, if any.
#[cfg(target_arch = "wasm32")]
fn get_stored_etag() -> Option<String> {
    web_sys::window()?
        .local_storage()
        .ok()??
        .get_item(EXERCISES_ETAG_KEY)
        .ok()?
}
/// Returns the stored `ETag` for `exercises.json`, if any.
#[cfg(not(target_arch = "wasm32"))]
fn get_stored_etag() -> Option<String> {
    crate::services::storage::native_storage::get_config_value(EXERCISES_ETAG_KEY)
}
/// Persists an `ETag` value for `exercises.json`.
#[cfg(target_arch = "wasm32")]
fn store_etag(etag: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return;
    };
    let _ = storage.set_item(EXERCISES_ETAG_KEY, etag);
}
/// Persists an `ETag` value for `exercises.json`.
#[cfg(not(target_arch = "wasm32"))]
fn store_etag(etag: &str) {
    let _ = crate::services::storage::native_storage::set_config_value(EXERCISES_ETAG_KEY, etag);
}
/// Downloads the exercises JSON from the configured URL using `reqwest`, then
/// fetches and merges all available per-language translation files
/// (e.g. `exercises.fr.json`) so that each [`Exercise::i18n`] field is
/// populated with translated name / instructions where available.
///
/// Sends `If-None-Match` with the stored `ETag` on each request.  On a
/// `304 Not Modified` response the server confirms the cached copy is still
/// current and the function returns `Ok(None)` – the caller should keep
/// using its cached exercises unchanged.  On a successful `200` the response
/// `ETag` (if provided) is persisted for the next request, and the parsed
/// exercise list is returned as `Ok(Some(exercises))`.
///
/// Works on all platforms: reqwest uses the browser's `fetch` on WASM and
/// native TLS on Android / desktop.
pub(crate) async fn download_exercises() -> Result<Option<Vec<Exercise>>, String> {
    let url = exercises_json_url();
    let mut request = reqwest::Client::new().get(&url);
    if let Some(etag) = get_stored_etag() {
        request = request.header("If-None-Match", etag);
    }
    let response = request
        .send()
        .await
        .map_err(|e| format!("HTTP error: {e}"))?;
    if response.status().as_u16() == 304 {
        log::info!("exercises.json is up to date (304 Not Modified)");
        return Ok(None);
    }
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }
    // Persist the ETag for the next conditional request.
    if let Some(etag) = response
        .headers()
        .get(reqwest::header::ETAG)
        .and_then(|v| v.to_str().ok())
    {
        store_etag(etag);
    }
    let mut exercises: Vec<Exercise> = response
        .json()
        .await
        .map_err(|e| format!("JSON parse error: {e}"))?;
    for lang in SUPPORTED_TRANSLATION_LANGS {
        if let Ok(entries) = download_exercise_lang(lang).await {
            merge_lang_entries(&mut exercises, lang, &entries);
        }
    }
    Ok(Some(exercises))
}
/// Downloads a per-language exercise translation file (e.g. `exercises.fr.json`)
/// and returns the parsed entries.  Returns `Ok(vec![])` on HTTP 404 so the
/// caller can safely ignore missing languages.
async fn download_exercise_lang(lang: &str) -> Result<Vec<ExerciseLangEntry>, String> {
    let url = exercises_lang_json_url(lang);
    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("HTTP error fetching {lang} lang file: {e}"))?;
    if response.status().as_u16() == 404 {
        return Ok(Vec::new());
    }
    if !response.status().is_success() {
        return Err(format!(
            "HTTP {} fetching {lang} lang file",
            response.status()
        ));
    }
    response
        .json()
        .await
        .map_err(|e| format!("JSON parse error in {lang} lang file: {e}"))
}
/// Merges a slice of [`ExerciseLangEntry`] values into the in-memory exercise
/// list by matching on `id`.  Each entry's `name` and `instructions` are
/// inserted into the exercise's `i18n` map under the given language code.
fn merge_lang_entries(exercises: &mut [Exercise], lang: &str, entries: &[ExerciseLangEntry]) {
    use std::collections::HashMap;
    let entry_map: HashMap<&str, &ExerciseLangEntry> =
        entries.iter().map(|e| (e.id.as_str(), e)).collect();
    for exercise in exercises.iter_mut() {
        if let Some(entry) = entry_map.get(exercise.id.as_str()) {
            if entry.name.is_some() || entry.instructions.is_some() {
                let map = exercise.i18n.get_or_insert_with(HashMap::new);
                map.insert(
                    lang.to_owned(),
                    ExerciseI18n {
                        name: entry.name.clone(),
                        instructions: entry.instructions.clone(),
                    },
                );
            }
        }
    }
}
/// Downloads the enum-translation file (`i18n.json`) from the configured URL.
/// Returns `Ok(data)` on success, or `Err(message)` when a network or parse
/// error occurs so the caller can surface it via the toast signal.
pub(crate) async fn download_db_i18n() -> Result<DbI18n, String> {
    let url = db_i18n_url();
    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Network error fetching i18n.json: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("HTTP {} fetching i18n.json", response.status()));
    }
    response
        .json()
        .await
        .map_err(|e| format!("JSON parse error in i18n.json: {e}"))
}
/// Normalises a string for error-tolerant search: lowercases and strips
/// hyphens, apostrophes, and spaces so that e.g. "push-ups", "Pushups", and
/// "Push Ups" all collapse to the same canonical form.
fn normalize_for_search(s: &str) -> String {
    s.chars()
        .filter(|c| !matches!(c, '-' | '\'' | ' ' | '.'))
        .flat_map(char::to_lowercase)
        .collect()
}
/// Returns true if an already-lowercased `name_lc` matches the given
/// pre-computed search components (all lowercase / normalised).
fn name_lc_matches(name_lc: &str, query_lower: &str, query_norm: &str, tokens: &[String]) -> bool {
    name_lc.contains(query_lower)
        || (!tokens.is_empty() && {
            let name_norm = normalize_for_search(name_lc);
            tokens.iter().all(|t| name_norm.contains(t.as_str()))
        })
        || (!query_norm.is_empty() && {
            let name_norm = normalize_for_search(name_lc);
            query_norm.contains(&name_norm)
        })
}
/// Relevance score tiers for exercise search results.
/// Higher = better match.
const SCORE_EXACT_NAME: u32 = 100;
const SCORE_NAME_STARTS: u32 = 90;
const SCORE_NAME_NORM_EXACT: u32 = 85;
const SCORE_NAME_CONTAINS: u32 = 80;
const SCORE_NAME_NORM_TOKEN_START: u32 = 75;
const SCORE_NAME_NORM_CONTAINS: u32 = 70;
const SCORE_NAME_ALL_TOKENS: u32 = 65;
const SCORE_NAME_REVERSE: u32 = 60;
const SCORE_I18N_NAME: u32 = 55;
/// Computes a relevance score for `exercise` against the pre-computed query
/// components.  Returns 0 if the exercise does not match the query at all.
/// Only the exercise title (English and all available localised names) is
/// searched; attribute filtering is handled exclusively by hard filters.
fn score_exercise(
    exercise: &Exercise,
    query_lower: &str,
    query_norm: &str,
    tokens: &[String],
) -> u32 {
    let computed_name_lower;
    let name_lc: &str = if exercise.name_lower.is_empty() {
        computed_name_lower = exercise.name.to_lowercase();
        &computed_name_lower
    } else {
        &exercise.name_lower
    };
    let name_norm = normalize_for_search(name_lc);
    if name_lc == query_lower {
        return SCORE_EXACT_NAME;
    }
    if name_lc.starts_with(query_lower) {
        return SCORE_NAME_STARTS;
    }
    if !query_norm.is_empty() && name_norm == query_norm {
        return SCORE_NAME_NORM_EXACT;
    }
    if name_lc.contains(query_lower) {
        return SCORE_NAME_CONTAINS;
    }
    if !tokens.is_empty() && name_norm.starts_with(tokens[0].as_str()) {
        return SCORE_NAME_NORM_TOKEN_START;
    }
    if !query_norm.is_empty() && name_norm.contains(query_norm) {
        return SCORE_NAME_NORM_CONTAINS;
    }
    if !tokens.is_empty() && tokens.iter().all(|t| name_norm.contains(t.as_str())) {
        return SCORE_NAME_ALL_TOKENS;
    }
    if !query_norm.is_empty() && !name_norm.is_empty() && query_norm.contains(&name_norm) {
        return SCORE_NAME_REVERSE;
    }
    if exercise.i18n.as_ref().is_some_and(|map| {
        map.values().any(|i18n| {
            i18n.name.as_deref().is_some_and(|n| {
                let n_lc = n.to_lowercase();
                name_lc_matches(&n_lc, query_lower, query_norm, tokens)
            })
        })
    }) {
        return SCORE_I18N_NAME;
    }
    0
}
/// Search exercises by title (English name and all available localised names).
///
/// Attribute values (muscles, category, force, equipment, level) are
/// intentionally excluded from search; use hard filters (`SearchFilter`) for
/// attribute-based filtering and `detect_filter_suggestions` to turn a query
/// into a suggested filter chip.
///
/// Results are sorted by relevance: exact / near-exact name matches appear
/// first, followed by prefix / token matches.
///
/// Works with any element type that dereferences to [`Exercise`] (e.g. plain
/// `Exercise` in tests, `Arc<Exercise>` in production signals).
pub fn search_exercises<'a, E>(exercises: &'a [E], query: &str) -> Vec<&'a E>
where
    E: AsRef<Exercise>,
{
    let query_lower = query.to_lowercase();
    let query_norm = normalize_for_search(query);
    let tokens: Vec<String> = query_lower
        .split_whitespace()
        .map(normalize_for_search)
        .filter(|t| t.chars().any(char::is_alphanumeric))
        .collect();
    let mut scored: Vec<(u32, &E)> = exercises
        .iter()
        .filter_map(|exercise| {
            let score = score_exercise(exercise.as_ref(), &query_lower, &query_norm, &tokens);
            if score > 0 {
                Some((score, exercise))
            } else {
                None
            }
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.into_iter().map(|(_, ex)| ex).collect()
}
/// A hard filter that restricts the exercise list to a specific attribute value.
///
/// Up to 4 filters can be active simultaneously.  Filters of the **same
/// variant** (e.g. two `Category` filters) form a **union** (OR) so that
/// contradictory values like "strength + cardio" return exercises that satisfy
/// either constraint.  Filters of **different variants** form an
/// **intersection** (AND).
#[derive(Clone, PartialEq, Debug)]
pub enum SearchFilter {
    Category(Category),
    Force(Force),
    Equipment(Equipment),
    Level(Level),
    /// Matches exercises where `muscle` is either a primary or secondary muscle.
    Muscle(Muscle),
}
impl SearchFilter {
    /// Human-readable label for display in the UI (e.g. "💪 strength").
    pub fn label(&self) -> String {
        match self {
            Self::Category(c) => format!("🏷 {c}"),
            Self::Force(f) => format!("⚡ {f}"),
            Self::Equipment(e) => format!("🔧 {e}"),
            Self::Level(l) => format!("📊 {l}"),
            Self::Muscle(m) => format!("💪 {m}"),
        }
    }
    /// Returns true if `self` and `other` are the same variant (regardless of
    /// the inner value).  Used to group contradictory filters into OR unions.
    pub fn same_kind(&self, other: &SearchFilter) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
    /// Returns true when `exercise` satisfies this individual filter.
    fn matches(&self, exercise: &Exercise) -> bool {
        match self {
            Self::Category(c) => &exercise.category == c,
            Self::Force(f) => exercise.force.as_ref() == Some(f),
            Self::Equipment(e) => exercise.equipment.as_ref() == Some(e),
            Self::Level(l) => exercise.level.as_ref() == Some(l),
            Self::Muscle(m) => {
                exercise.primary_muscles.contains(m) || exercise.secondary_muscles.contains(m)
            }
        }
    }
}
/// Returns true when `exercise` passes **all** active filters.
///
/// Filters of the same variant form an OR group; OR groups are `AND`ed together.
pub fn exercise_matches_filters(exercise: &Exercise, filters: &[SearchFilter]) -> bool {
    if filters.is_empty() {
        return true;
    }
    let mut handled: Vec<&SearchFilter> = Vec::new();
    for filter in filters {
        if handled.iter().any(|h| h.same_kind(filter)) {
            continue;
        }
        handled.push(filter);
        let group_ok = filters
            .iter()
            .filter(|f| f.same_kind(filter))
            .any(|f| f.matches(exercise));
        if !group_ok {
            return false;
        }
    }
    true
}
/// Examines `query` and returns filter suggestions – one per matching attribute
/// value.  A suggestion is emitted when the query **exactly equals** (case-
/// insensitive) or starts with a known attribute value (or vice-versa) so that
/// typing "card", "cardio", or "CARDIO" all suggest the `Category::Cardio`
/// filter.
pub fn detect_filter_suggestions(query: &str) -> Vec<SearchFilter> {
    use strum::IntoEnumIterator;
    let q = query.to_lowercase();
    if q.len() < 2 {
        return Vec::new();
    }
    let mut suggestions = Vec::new();
    for cat in Category::iter() {
        let val = cat.as_ref().to_lowercase();
        if val.contains(&q) || q.contains(&val) {
            suggestions.push(SearchFilter::Category(cat));
        }
    }
    for force in Force::iter() {
        let val = force.as_ref().to_lowercase();
        if val.contains(&q) || q.contains(&val) {
            suggestions.push(SearchFilter::Force(force));
        }
    }
    for equip in Equipment::iter() {
        let val = equip.as_ref().to_lowercase();
        if val.contains(&q) || q.contains(&val) {
            suggestions.push(SearchFilter::Equipment(equip));
        }
    }
    for level in Level::iter() {
        let val = level.as_ref().to_lowercase();
        if val.contains(&q) || q.contains(&val) {
            suggestions.push(SearchFilter::Level(level));
        }
    }
    for muscle in Muscle::iter() {
        let val = muscle.as_ref().to_lowercase();
        if val.contains(&q) || q.contains(&val) {
            suggestions.push(SearchFilter::Muscle(muscle));
        }
    }
    suggestions
}
/// Looks up an exercise by ID in a slice.
///
/// Works with any element type that dereferences to [`Exercise`] (e.g. plain
/// `Exercise` in tests, `Arc<Exercise>` in production signals).
pub fn get_exercise_by_id<'a, E>(exercises: &'a [E], id: &str) -> Option<&'a E>
where
    E: AsRef<Exercise>,
{
    exercises.iter().find(|e| e.as_ref().id == id)
}
/// Resolves an exercise by ID: checks the main DB slice first, then falls back
/// to the custom-exercises slice.  Centralises the lookup logic used across
/// multiple components.
///
/// Works with any element type that dereferences to [`Exercise`] (e.g. plain
/// `Exercise` in tests, `Arc<Exercise>` in production signals).
pub fn resolve_exercise<'a, E>(db: &'a [E], custom: &'a [E], id: &str) -> Option<&'a E>
where
    E: AsRef<Exercise>,
{
    get_exercise_by_id(db, id).or_else(|| get_exercise_by_id(custom, id))
}
#[cfg(test)]
pub fn get_equipment_types(exercises: &[Exercise]) -> Vec<Equipment> {
    let mut equipment: Vec<Equipment> = exercises.iter().filter_map(|e| e.equipment).collect();
    equipment.sort_by_key(std::string::ToString::to_string);
    equipment.dedup();
    equipment
}
#[cfg(test)]
pub fn get_muscle_groups(exercises: &[Exercise]) -> Vec<Muscle> {
    let mut muscles: Vec<Muscle> = exercises
        .iter()
        .flat_map(|e| e.primary_muscles.iter().copied())
        .collect();
    muscles.sort_by_key(std::string::ToString::to_string);
    muscles.dedup();
    muscles
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Category, Equipment, Force, Level, Muscle};
    fn sample_exercises() -> Vec<Exercise> {
        vec![
            Exercise {
                id: "bench_press".into(),
                name: "Bench Press".into(),
                name_lower: String::new(),
                force: Some(Force::Push),
                level: Some(Level::Intermediate),
                mechanic: None,
                equipment: Some(Equipment::Barbell),
                primary_muscles: vec![Muscle::Chest],
                secondary_muscles: vec![Muscle::Triceps],
                instructions: vec![],
                category: Category::Strength,
                images: vec![],
                i18n: None,
            }
            .with_lowercase(),
            Exercise {
                id: "pull_up".into(),
                name: "Pull-Up".into(),
                name_lower: String::new(),
                force: Some(Force::Pull),
                level: Some(Level::Beginner),
                mechanic: None,
                equipment: Some(Equipment::BodyOnly),
                primary_muscles: vec![Muscle::Lats],
                secondary_muscles: vec![Muscle::Biceps],
                instructions: vec![],
                category: Category::Strength,
                images: vec![],
                i18n: None,
            }
            .with_lowercase(),
            Exercise {
                id: "running".into(),
                name: "Running".into(),
                name_lower: String::new(),
                force: None,
                level: Some(Level::Beginner),
                mechanic: None,
                equipment: None,
                primary_muscles: vec![Muscle::Quadriceps, Muscle::Hamstrings],
                secondary_muscles: vec![],
                instructions: vec![],
                category: Category::Cardio,
                images: vec![],
                i18n: None,
            }
            .with_lowercase(),
        ]
    }
    #[test]
    fn search_by_name() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "bench");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "bench_press");
    }
    #[test]
    fn search_by_muscle_returns_empty() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "lats");
        assert!(results.is_empty());
    }
    #[test]
    fn search_by_category_returns_empty() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "cardio");
        assert!(results.is_empty());
    }
    #[test]
    fn search_by_force_returns_empty() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "push");
        assert!(results.is_empty());
    }
    #[test]
    fn search_by_equipment_returns_empty() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "barbell");
        assert!(results.is_empty());
    }
    #[test]
    fn search_by_level_returns_empty() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "beginner");
        assert!(results.is_empty());
    }
    #[test]
    fn search_case_insensitive() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "BENCH");
        assert_eq!(results.len(), 1);
    }
    #[test]
    fn search_no_match() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "zzz_no_match");
        assert!(results.is_empty());
    }
    #[test]
    fn search_hyphenated_query_finds_unhyphenated_name() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "pull-up");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "pull_up");
    }
    #[test]
    fn search_plain_query_finds_hyphenated_name() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "pullup");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "pull_up");
    }
    #[test]
    fn search_pluralised_query_finds_exercise() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "bench press");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "bench_press");
    }
    #[test]
    fn search_multi_word_finds_interleaved_words() {
        let exercises = vec![Exercise {
            id: "wide_grip_bench".into(),
            name: "Wide-Grip Barbell Bench Press".into(),
            name_lower: String::new(),
            force: Some(Force::Push),
            level: Some(Level::Intermediate),
            mechanic: None,
            equipment: Some(Equipment::Barbell),
            primary_muscles: vec![Muscle::Chest],
            secondary_muscles: vec![],
            instructions: vec![],
            category: Category::Strength,
            images: vec![],
            i18n: None,
        }
        .with_lowercase()];
        let results = search_exercises(&exercises, "wide grip bench");
        assert_eq!(
            results.len(),
            1,
            "token-based search should find the exercise"
        );
        assert_eq!(results[0].id, "wide_grip_bench");
    }
    #[test]
    fn search_punctuation_only_token_is_ignored() {
        let exercises = vec![Exercise {
            id: "pushups".into(),
            name: "Pushups".into(),
            name_lower: String::new(),
            force: Some(Force::Push),
            level: Some(Level::Beginner),
            mechanic: None,
            equipment: Some(Equipment::BodyOnly),
            primary_muscles: vec![Muscle::Chest],
            secondary_muscles: vec![],
            instructions: vec![],
            category: Category::Strength,
            images: vec![],
            i18n: None,
        }
        .with_lowercase()];
        let results = search_exercises(&exercises, "… pushups");
        assert_eq!(results.len(), 1, "punctuation-only token should be ignored");
        assert_eq!(results[0].id, "pushups");
    }
    #[test]
    fn normalize_strips_hyphens_apostrophes_spaces() {
        assert_eq!(normalize_for_search("push-ups"), "pushups");
        assert_eq!(normalize_for_search("Pull-Up"), "pullup");
        assert_eq!(normalize_for_search("farmer's walk"), "farmerswalk");
        assert_eq!(normalize_for_search("Bench Press"), "benchpress");
    }
    #[test]
    fn search_empty_query_returns_all() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "");
        assert_eq!(results.len(), exercises.len());
    }
    #[test]
    fn get_exercise_by_id_found() {
        let exercises = sample_exercises();
        let found = get_exercise_by_id(&exercises, "pull_up");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Pull-Up");
    }
    #[test]
    fn get_exercise_by_id_not_found() {
        let exercises = sample_exercises();
        let found = get_exercise_by_id(&exercises, "nonexistent");
        assert!(found.is_none());
    }
    #[test]
    fn resolve_exercise_finds_in_db() {
        let db = sample_exercises();
        let custom = vec![];
        let found = resolve_exercise(&db, &custom, "pull_up");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Pull-Up");
    }
    #[test]
    fn resolve_exercise_falls_back_to_custom() {
        let db = sample_exercises();
        let custom = vec![Exercise {
            id: "custom_1".into(),
            name: "Custom Move".into(),
            name_lower: String::new(),
            force: None,
            level: None,
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec![],
            category: crate::models::Category::Strength,
            images: vec![],
            i18n: None,
        }];
        let found = resolve_exercise(&db, &custom, "custom_1");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Custom Move");
    }
    #[test]
    fn resolve_exercise_db_takes_priority_over_custom() {
        let db = sample_exercises();
        let custom = vec![Exercise {
            id: "pull_up".into(),
            name: "Custom Pull-Up".into(),
            name_lower: String::new(),
            force: None,
            level: None,
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec![],
            category: crate::models::Category::Strength,
            images: vec![],
            i18n: None,
        }];
        let found = resolve_exercise(&db, &custom, "pull_up");
        assert_eq!(found.unwrap().name, "Pull-Up");
    }
    #[test]
    fn resolve_exercise_not_found() {
        let db = sample_exercises();
        let custom = vec![];
        let found = resolve_exercise(&db, &custom, "nonexistent");
        assert!(found.is_none());
    }
    #[test]
    fn get_equipment_types_deduplicates() {
        let exercises = sample_exercises();
        let equipment = get_equipment_types(&exercises);
        assert_eq!(equipment.len(), 2);
    }
    #[test]
    fn get_muscle_groups_deduplicates() {
        let exercises = sample_exercises();
        let muscles = get_muscle_groups(&exercises);
        assert_eq!(muscles.len(), 4);
    }
    #[test]
    fn search_with_none_force_does_not_match_by_name_of_pull() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "pull");
        for r in &results {
            assert_ne!(r.id, "running");
        }
    }
    #[test]
    fn search_with_body_only_equipment_returns_empty() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "body only");
        assert!(results.is_empty());
    }
    #[test]
    fn search_by_normalized_id_returns_empty() {
        let exercises = vec![Exercise {
            id: "kettlebell_pistol_squat".into(),
            name: "KB Pistol Squat".into(),
            name_lower: String::new(),
            force: Some(Force::Push),
            level: Some(Level::Intermediate),
            mechanic: None,
            equipment: Some(Equipment::Kettlebells),
            primary_muscles: vec![Muscle::Quadriceps],
            secondary_muscles: vec![],
            instructions: vec![],
            category: Category::Strength,
            images: vec![],
            i18n: None,
        }
        .with_lowercase()];
        let results = search_exercises(&exercises, "kettlebell");
        assert!(
            results.is_empty(),
            "ID token matching is removed; title 'KB Pistol Squat' does not contain 'kettlebell'",
        );
    }
    #[test]
    fn search_by_secondary_muscle_returns_empty() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "triceps");
        assert!(results.is_empty());
    }
    #[test]
    fn search_by_secondary_muscle_biceps_returns_empty() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "biceps");
        assert!(results.is_empty());
    }
    #[test]
    fn search_muscle_word_start_no_false_positive() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "ring");
        assert!(!results.iter().any(|e| e.id == "running"));
    }
    #[test]
    fn search_muscle_word_start_prefix_returns_empty() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "ham");
        assert!(!results.iter().any(|e| e.id == "running"));
    }
    #[test]
    fn exercises_json_url_uses_fork() {
        #[cfg(not(target_arch = "wasm32"))]
        let _g = crate::services::storage::native_storage::test_lock();
        let url = exercises_json_url();
        assert!(
            url.contains("gfauredev"),
            "Expected gfauredev fork URL, got: {url}"
        );
        assert!(url.ends_with("exercises.json"));
    }
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn is_refresh_due_true_when_no_timestamp() {
        assert!(is_refresh_due_for(1_000_000, None));
    }
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn is_refresh_due_false_when_recent() {
        let now = 1_000_000u64;
        let last_fetch = now - 60;
        assert!(!is_refresh_due_for(now, Some(last_fetch)));
    }
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn is_refresh_due_true_when_stale() {
        let interval = EXERCISE_DB_REFRESH_INTERVAL_SECS;
        let now = interval + 1_000_000;
        let last_fetch = 1_000_000u64;
        assert!(is_refresh_due_for(now, Some(last_fetch)));
    }
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn is_refresh_due_false_at_exact_interval_boundary() {
        let interval = EXERCISE_DB_REFRESH_INTERVAL_SECS;
        let now = interval + 999_999u64;
        let last_fetch = 999_999u64;
        assert!(is_refresh_due_for(now, Some(last_fetch)));
    }
    #[test]
    fn search_custom_exercise_by_muscle_returns_empty() {
        let exercises = vec![Exercise {
            id: "custom_squat".into(),
            name: "Custom Squat".into(),
            name_lower: String::new(),
            force: Some(Force::Push),
            level: Some(Level::Beginner),
            mechanic: None,
            equipment: None,
            primary_muscles: vec![Muscle::Quadriceps],
            secondary_muscles: vec![Muscle::Glutes],
            instructions: vec![],
            category: Category::Strength,
            images: vec![],
            i18n: None,
        }];
        let results = search_exercises(&exercises, "quadriceps");
        assert!(results.is_empty());
    }
    #[test]
    fn search_custom_exercise_by_secondary_muscle_returns_empty() {
        let exercises = vec![Exercise {
            id: "custom_squat".into(),
            name: "Custom Squat".into(),
            name_lower: String::new(),
            force: Some(Force::Push),
            level: Some(Level::Beginner),
            mechanic: None,
            equipment: None,
            primary_muscles: vec![Muscle::Quadriceps],
            secondary_muscles: vec![Muscle::Glutes],
            instructions: vec![],
            category: Category::Strength,
            images: vec![],
            i18n: None,
        }];
        let results = search_exercises(&exercises, "glutes");
        assert!(results.is_empty());
    }
    #[test]
    fn search_custom_exercise_by_category_returns_empty() {
        let exercises = vec![Exercise {
            id: "custom_run".into(),
            name: "My Run".into(),
            name_lower: String::new(),
            force: None,
            level: None,
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec![],
            category: Category::Cardio,
            images: vec![],
            i18n: None,
        }];
        let results = search_exercises(&exercises, "cardio");
        assert!(results.is_empty());
    }
    #[test]
    fn search_by_i18n_name() {
        let mut i18n_map = std::collections::HashMap::new();
        i18n_map.insert(
            "fr".to_string(),
            crate::models::ExerciseI18n {
                name: Some("Développé couché".to_string()),
                instructions: None,
            },
        );
        let exercises = vec![Exercise {
            id: "bench_press".into(),
            name: "Bench Press".into(),
            name_lower: String::new(),
            force: Some(Force::Push),
            level: Some(Level::Intermediate),
            mechanic: None,
            equipment: Some(Equipment::Barbell),
            primary_muscles: vec![Muscle::Chest],
            secondary_muscles: vec![],
            instructions: vec![],
            category: Category::Strength,
            images: vec![],
            i18n: Some(i18n_map),
        }
        .with_lowercase()];
        let results = search_exercises(&exercises, "développé");
        assert_eq!(results.len(), 1, "should find by French name");
        assert_eq!(results[0].id, "bench_press");
    }
    #[test]
    fn search_by_translated_tag_returns_empty() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "musculation");
        assert!(results.is_empty());
    }
    #[test]
    fn search_by_translated_tag_without_db_i18n_does_not_match() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "musculation");
        assert!(results.is_empty());
    }
    #[test]
    fn get_equipment_types_only_returns_some_equipment() {
        let exercises = sample_exercises();
        let equipment = get_equipment_types(&exercises);
        assert!(equipment.iter().all(|e| !e.as_ref().is_empty()));
    }
    #[test]
    fn get_muscle_groups_only_returns_primary_muscles() {
        let exercises = sample_exercises();
        let muscles = get_muscle_groups(&exercises);
        assert_eq!(muscles.len(), 4);
    }
    #[cfg(not(target_arch = "wasm32"))]
    mod native {
        use super::*;
        use crate::services::storage::native_storage;
        /// One lock that serialises every test touching the shared config file.
        fn cfg_lock() -> std::sync::MutexGuard<'static, ()> {
            native_storage::test_lock()
        }
        /// RAII helper that removes a config key on drop, ensuring cleanup even
        /// if the test body panics.
        struct ConfigKeyGuard(&'static str);
        impl Drop for ConfigKeyGuard {
            fn drop(&mut self) {
                let _ = native_storage::remove_config_value(self.0);
            }
        }
        #[test]
        fn record_fetch_timestamp_writes_numeric_value() {
            let _g = cfg_lock();
            let _ = native_storage::remove_config_value(LAST_FETCH_KEY);
            record_fetch_timestamp();
            let val = native_storage::get_config_value(LAST_FETCH_KEY)
                .expect("timestamp should be written");
            let ts: u64 = val.parse().expect("value should be a numeric timestamp");
            assert!(ts > 0, "timestamp should be positive");
        }
        #[test]
        fn clear_fetch_cache_removes_config_value() {
            let _g = cfg_lock();
            record_fetch_timestamp();
            assert!(native_storage::get_config_value(LAST_FETCH_KEY).is_some());
            clear_fetch_cache();
            assert!(
                native_storage::get_config_value(LAST_FETCH_KEY).is_none(),
                "config value should be removed after clear_fetch_cache",
            );
        }
        #[test]
        fn is_refresh_due_true_when_no_config_entry() {
            let _g = cfg_lock();
            let _ = native_storage::remove_config_value(LAST_FETCH_KEY);
            assert!(
                is_refresh_due(),
                "refresh should be due with no cached timestamp"
            );
        }
        #[test]
        fn is_refresh_due_false_after_fresh_timestamp() {
            let _g = cfg_lock();
            record_fetch_timestamp();
            assert!(
                !is_refresh_due(),
                "refresh should not be due immediately after recording a fresh timestamp",
            );
        }
        /// Starts a minimal TCP server in a background thread that sends
        /// `response_bytes` to the first incoming connection, then exits.
        /// Returns the TCP port the server is listening on.
        fn start_one_shot_server(response_bytes: Vec<u8>) -> u16 {
            use std::io::{Read, Write};
            use std::net::TcpListener;
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();
            std::thread::spawn(move || {
                if let Ok((mut stream, _)) = listener.accept() {
                    let mut buf = [0u8; 4096];
                    let _ = stream.read(&mut buf);
                    let _ = stream.write_all(&response_bytes);
                }
            });
            port
        }
        #[test]
        fn download_exercises_returns_error_on_connection_refused() {
            let _g = cfg_lock();
            let port = {
                let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
                l.local_addr().unwrap().port()
            };
            let _url = ConfigKeyGuard(crate::utils::EXERCISE_DB_URL_STORAGE_KEY);
            let _ = native_storage::set_config_value(
                crate::utils::EXERCISE_DB_URL_STORAGE_KEY,
                &format!("http://127.0.0.1:{port}/"),
            );
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            let result = rt.block_on(download_exercises());
            assert!(
                result.is_err(),
                "expected connection error, got: {result:?}"
            );
            assert!(
                result.unwrap_err().contains("HTTP error"),
                "error message should mention 'HTTP error'",
            );
        }
        #[test]
        fn download_exercises_returns_error_on_http_404() {
            let _g = cfg_lock();
            let response =
                b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    .to_vec();
            let port = start_one_shot_server(response);
            let _url = ConfigKeyGuard(crate::utils::EXERCISE_DB_URL_STORAGE_KEY);
            let _ = native_storage::set_config_value(
                crate::utils::EXERCISE_DB_URL_STORAGE_KEY,
                &format!("http://127.0.0.1:{port}/"),
            );
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            let result = rt.block_on(download_exercises());
            assert!(result.is_err(), "expected HTTP error, got: {result:?}");
            let err = result.unwrap_err();
            assert!(
                err.contains("HTTP 404"),
                "error should mention HTTP 404, got: {err}",
            );
        }
        #[test]
        fn download_exercises_returns_empty_vec_on_200_empty_json() {
            let _g = cfg_lock();
            let body = b"[]";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len(),
            )
                .into_bytes()
                .into_iter()
                .chain(body.iter().copied())
                .collect::<Vec<u8>>();
            let port = start_one_shot_server(response);
            let _url = ConfigKeyGuard(crate::utils::EXERCISE_DB_URL_STORAGE_KEY);
            let _ = native_storage::set_config_value(
                crate::utils::EXERCISE_DB_URL_STORAGE_KEY,
                &format!("http://127.0.0.1:{port}/"),
            );
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            let result = rt.block_on(download_exercises());
            assert!(result.is_ok(), "expected Ok(Some([])), got: {result:?}");
            assert!(result.unwrap().unwrap().is_empty());
        }
        #[test]
        fn download_exercises_returns_none_on_304() {
            let _g = cfg_lock();
            let response =
                b"HTTP/1.1 304 Not Modified\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    .to_vec();
            let port = start_one_shot_server(response);
            let _url = ConfigKeyGuard(crate::utils::EXERCISE_DB_URL_STORAGE_KEY);
            let _ = native_storage::set_config_value(
                crate::utils::EXERCISE_DB_URL_STORAGE_KEY,
                &format!("http://127.0.0.1:{port}/"),
            );
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            let result = rt.block_on(download_exercises());
            assert!(
                matches!(result, Ok(None)),
                "expected Ok(None) on 304, got: {result:?}",
            );
        }
        #[test]
        fn download_db_i18n_returns_err_on_connection_refused() {
            let _g = cfg_lock();
            let port = {
                let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
                l.local_addr().unwrap().port()
            };
            let _url = ConfigKeyGuard(crate::utils::EXERCISE_DB_URL_STORAGE_KEY);
            let _ = native_storage::set_config_value(
                crate::utils::EXERCISE_DB_URL_STORAGE_KEY,
                &format!("http://127.0.0.1:{port}/"),
            );
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            let result = rt.block_on(download_db_i18n());
            assert!(
                result.is_err(),
                "download_db_i18n should return Err on connection error",
            );
        }
    }
    #[test]
    fn merge_lang_entries_inserts_translation_for_matching_id() {
        let mut exercises = vec![Exercise {
            id: "bench_press".into(),
            name: "Bench Press".into(),
            name_lower: String::new(),
            force: None,
            level: None,
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec!["Step 1".into()],
            category: Category::Strength,
            images: vec![],
            i18n: None,
        }];
        let entries = vec![ExerciseLangEntry {
            id: "bench_press".into(),
            name: Some("Développé Couché".into()),
            instructions: Some(vec!["Étape 1".into()]),
        }];
        merge_lang_entries(&mut exercises, "fr", &entries);
        let i18n = exercises[0].i18n.as_ref().expect("i18n map should be set");
        let fr = i18n.get("fr").expect("'fr' entry should exist");
        assert_eq!(fr.name.as_deref(), Some("Développé Couché"));
        assert_eq!(
            fr.instructions.as_deref(),
            Some(&["Étape 1".to_owned()][..])
        );
    }
    #[test]
    fn merge_lang_entries_skips_unmatched_ids() {
        let mut exercises = vec![Exercise {
            id: "squat".into(),
            name: "Squat".into(),
            name_lower: String::new(),
            force: None,
            level: None,
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec![],
            category: Category::Strength,
            images: vec![],
            i18n: None,
        }];
        let entries = vec![ExerciseLangEntry {
            id: "bench_press".into(),
            name: Some("Développé Couché".into()),
            instructions: None,
        }];
        merge_lang_entries(&mut exercises, "fr", &entries);
        assert!(
            exercises[0].i18n.is_none(),
            "unmatched entry should not create an i18n map",
        );
    }
    #[test]
    fn merge_lang_entries_preserves_existing_i18n_for_other_langs() {
        use std::collections::HashMap;
        let mut existing_i18n = HashMap::new();
        existing_i18n.insert(
            "es".to_owned(),
            ExerciseI18n {
                name: Some("Press de Banca".into()),
                instructions: None,
            },
        );
        let mut exercises = vec![Exercise {
            id: "bench_press".into(),
            name: "Bench Press".into(),
            name_lower: String::new(),
            force: None,
            level: None,
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec![],
            category: Category::Strength,
            images: vec![],
            i18n: Some(existing_i18n),
        }];
        let entries = vec![ExerciseLangEntry {
            id: "bench_press".into(),
            name: Some("Développé Couché".into()),
            instructions: None,
        }];
        merge_lang_entries(&mut exercises, "fr", &entries);
        let i18n = exercises[0].i18n.as_ref().unwrap();
        assert!(i18n.contains_key("es"), "'es' entry should be preserved");
        assert!(i18n.contains_key("fr"), "'fr' entry should be added");
    }
    #[test]
    fn exercises_lang_json_url_returns_correct_format() {
        #[cfg(not(target_arch = "wasm32"))]
        let _g = crate::services::storage::native_storage::test_lock();
        let url = exercises_lang_json_url("fr");
        assert!(url.contains("gfauredev"), "URL should reference gfauredev");
        assert!(
            url.ends_with("exercises.fr.json"),
            "URL should end with exercises.fr.json, got: {url}",
        );
    }
    #[test]
    fn db_i18n_url_returns_correct_format() {
        #[cfg(not(target_arch = "wasm32"))]
        let _g = crate::services::storage::native_storage::test_lock();
        let url = db_i18n_url();
        assert!(url.contains("gfauredev"), "URL should reference gfauredev");
        assert!(
            url.ends_with("i18n.json"),
            "URL should end with i18n.json, got: {url}"
        );
    }
    #[test]
    fn search_name_ranks_above_force_attribute() {
        let exercises = vec![
            Exercise {
                id: "bench_press".into(),
                name: "Bench Press".into(),
                name_lower: String::new(),
                force: Some(Force::Push),
                level: Some(Level::Intermediate),
                mechanic: None,
                equipment: Some(Equipment::Barbell),
                primary_muscles: vec![Muscle::Chest],
                secondary_muscles: vec![Muscle::Triceps],
                instructions: vec![],
                category: Category::Strength,
                images: vec![],
                i18n: None,
            }
            .with_lowercase(),
            Exercise {
                id: "push_up".into(),
                name: "Push-Up".into(),
                name_lower: String::new(),
                force: Some(Force::Push),
                level: Some(Level::Beginner),
                mechanic: None,
                equipment: Some(Equipment::BodyOnly),
                primary_muscles: vec![Muscle::Chest],
                secondary_muscles: vec![Muscle::Triceps],
                instructions: vec![],
                category: Category::Strength,
                images: vec![],
                i18n: None,
            }
            .with_lowercase(),
        ];
        let results = search_exercises(&exercises, "push-up");
        assert!(
            !results.is_empty(),
            "search should find at least the Push-Up exercise"
        );
        assert_eq!(
            results[0].id, "push_up",
            "Push-Up should be the first result when searching 'push-up'",
        );
    }
    #[test]
    fn search_exact_name_ranks_first() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "bench press");
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "bench_press");
    }
    #[test]
    fn search_name_prefix_ranks_above_substring() {
        let exercises = vec![
            Exercise {
                id: "supine_pull".into(),
                name: "Supine Pull".into(),
                name_lower: String::new(),
                force: Some(Force::Pull),
                level: Some(Level::Beginner),
                mechanic: None,
                equipment: None,
                primary_muscles: vec![Muscle::Lats],
                secondary_muscles: vec![],
                instructions: vec![],
                category: Category::Strength,
                images: vec![],
                i18n: None,
            }
            .with_lowercase(),
            Exercise {
                id: "pull_up".into(),
                name: "Pull-Up".into(),
                name_lower: String::new(),
                force: Some(Force::Pull),
                level: Some(Level::Beginner),
                mechanic: None,
                equipment: Some(Equipment::BodyOnly),
                primary_muscles: vec![Muscle::Lats],
                secondary_muscles: vec![Muscle::Biceps],
                instructions: vec![],
                category: Category::Strength,
                images: vec![],
                i18n: None,
            }
            .with_lowercase(),
        ];
        let results = search_exercises(&exercises, "pull");
        assert!(results.len() >= 2);
        assert_eq!(
            results[0].id, "pull_up",
            "Pull-Up (starts with 'pull') should rank above 'Supine Pull' (contains 'pull')",
        );
    }
    #[test]
    fn filter_by_category_matches_correct_exercises() {
        let exercises = sample_exercises();
        let filters = vec![SearchFilter::Category(Category::Cardio)];
        let results: Vec<_> = exercises
            .iter()
            .filter(|e| exercise_matches_filters(e, &filters))
            .collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "running");
    }
    #[test]
    fn filter_by_muscle_matches_primary_and_secondary() {
        let exercises = sample_exercises();
        let filters = vec![SearchFilter::Muscle(Muscle::Triceps)];
        let results: Vec<_> = exercises
            .iter()
            .filter(|e| exercise_matches_filters(e, &filters))
            .collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "bench_press");
    }
    #[test]
    fn filter_contradictory_same_kind_returns_union() {
        let exercises = sample_exercises();
        let filters = vec![
            SearchFilter::Category(Category::Cardio),
            SearchFilter::Category(Category::Strength),
        ];
        let results: Vec<_> = exercises
            .iter()
            .filter(|e| exercise_matches_filters(e, &filters))
            .collect();
        assert_eq!(results.len(), 3);
    }
    #[test]
    fn filter_different_kinds_intersect() {
        let exercises = sample_exercises();
        let filters = vec![
            SearchFilter::Category(Category::Strength),
            SearchFilter::Force(Force::Push),
        ];
        let results: Vec<_> = exercises
            .iter()
            .filter(|e| exercise_matches_filters(e, &filters))
            .collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "bench_press");
    }
    #[test]
    fn filter_empty_returns_all() {
        let exercises = sample_exercises();
        let results: Vec<_> = exercises
            .iter()
            .filter(|e| exercise_matches_filters(e, &[]))
            .collect();
        assert_eq!(results.len(), exercises.len());
    }
    #[test]
    fn detect_filter_suggests_category_for_cardio() {
        let suggestions = detect_filter_suggestions("cardio");
        assert!(
            suggestions
                .iter()
                .any(|f| f == &SearchFilter::Category(Category::Cardio)),
            "should suggest Cardio category filter",
        );
    }
    #[test]
    fn detect_filter_suggests_muscle_prefix() {
        let suggestions = detect_filter_suggestions("bicep");
        assert!(
            suggestions
                .iter()
                .any(|f| f == &SearchFilter::Muscle(Muscle::Biceps)),
            "should suggest Biceps muscle filter for prefix 'bicep'",
        );
    }
    #[test]
    fn detect_filter_short_query_returns_empty() {
        let suggestions = detect_filter_suggestions("a");
        assert!(
            suggestions.is_empty(),
            "single-character query should return no suggestions",
        );
    }
    #[test]
    fn detect_filter_suggests_level_beginner() {
        let suggestions = detect_filter_suggestions("beginner");
        assert!(
            suggestions
                .iter()
                .any(|f| f == &SearchFilter::Level(Level::Beginner)),
            "should suggest Beginner level filter",
        );
    }
    #[test]
    fn filter_label_is_human_readable() {
        assert_eq!(SearchFilter::Category(Category::Cardio).label(), "🏷 cardio");
        assert_eq!(SearchFilter::Force(Force::Push).label(), "⚡ push");
        assert_eq!(
            SearchFilter::Equipment(Equipment::Barbell).label(),
            "🔧 barbell"
        );
        assert_eq!(SearchFilter::Level(Level::Beginner).label(), "📊 beginner");
        assert_eq!(SearchFilter::Muscle(Muscle::Biceps).label(), "💪 biceps");
    }
    #[test]
    fn filter_same_kind_detects_contradictory() {
        let a = SearchFilter::Category(Category::Cardio);
        let b = SearchFilter::Category(Category::Strength);
        let c = SearchFilter::Force(Force::Push);
        assert!(a.same_kind(&b), "two Category filters are same kind");
        assert!(!a.same_kind(&c), "Category and Force are different kinds");
    }
}
