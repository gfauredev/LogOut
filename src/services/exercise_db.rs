use crate::models::{Category, DbI18n, Equipment, Exercise, ExerciseI18n, ExerciseLangEntry,
    Force, Level, Muscle};
use dioxus::prelude::*;

/// Newtype wrapper for the exercise-database signal so its `TypeId` is distinct
/// from the `Signal<Vec<Exercise>>` used by `storage::provide_app_state` for
/// custom exercises.  Without this wrapper both `use_context_provider` calls in
/// `App` would share the same context slot, causing both signals to point at the
/// same `Signal<Vec<Exercise>>` and leading to doubled counts, missing DB
/// exercises, and all exercises being treated as custom.
#[derive(Clone, Copy)]
pub(crate) struct AllExercisesSignal(pub(crate) Signal<Vec<Exercise>>);

/// Number of seconds between automatic exercise database refreshes (7 days).
const EXERCISE_DB_REFRESH_INTERVAL_SECS: u64 = 7 * 24 * 60 * 60;

/// Storage key used to track when exercises were last downloaded
/// (localStorage on WASM, config file on native).
const LAST_FETCH_KEY: &str = "exercise_db_last_fetch";

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
// Dioxus integration (provide/use context hooks + async loader) lives in the
// sibling `exercise_loader` module to keep this file focused on data-access
// logic and testable at ≥90% coverage.
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
}

/// Clears the locally-cached fetch timestamp so that the exercise database is
/// re-downloaded from the current URL on the next application load.
#[cfg(not(target_arch = "wasm32"))]
pub fn clear_fetch_cache() {
    use crate::services::storage::native_storage;
    let _ = native_storage::remove_config_value(LAST_FETCH_KEY);
}

/// Downloads the exercises JSON from the configured URL using `reqwest`, then
/// fetches and merges all available per-language translation files
/// (e.g. `exercises.fr.json`) so that each [`Exercise::i18n`] field is
/// populated with translated name / instructions where available.
///
/// Works on all platforms: reqwest uses the browser's `fetch` on WASM and
/// native TLS on Android / desktop.
pub(crate) async fn download_exercises() -> Result<Vec<Exercise>, String> {
    let url = exercises_json_url();
    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("HTTP error: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let mut exercises: Vec<Exercise> = response
        .json()
        .await
        .map_err(|e| format!("JSON parse error: {e}"))?;

    // Merge per-language translation files into each exercise's `i18n` map.
    for lang in SUPPORTED_TRANSLATION_LANGS {
        if let Ok(entries) = download_exercise_lang(lang).await {
            merge_lang_entries(&mut exercises, lang, &entries);
        }
    }

    Ok(exercises)
}

/// Downloads a per-language exercise translation file (e.g. `exercises.fr.json`)
/// and returns the parsed entries.  Returns `Ok(vec![])` on HTTP 404 so the
/// caller can safely ignore missing languages.
async fn download_exercise_lang(lang: &str) -> Result<Vec<ExerciseLangEntry>, String> {
    let url = exercises_lang_json_url(lang);
    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("HTTP error fetching {lang} lang file: {e}"))?;

    // 404 means the language file simply does not exist yet – not an error.
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
    // Build a quick lookup map from ID → entry; O(m) to build, then O(1) per
    // exercise lookup, giving O(n+m) overall instead of O(n·m) for a naïve scan.
    let entry_map: HashMap<&str, &ExerciseLangEntry> =
        entries.iter().map(|e| (e.id.as_str(), e)).collect();

    for exercise in exercises.iter_mut() {
        if let Some(entry) = entry_map.get(exercise.id.as_str()) {
            // Only create the i18n map if there is something to add.
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
/// Returns an empty [`DbI18n`] map on any HTTP or parse error so the app
/// degrades gracefully to English labels.
pub(crate) async fn download_db_i18n() -> DbI18n {
    let url = db_i18n_url();
    let Ok(response) = reqwest::get(&url).await else {
        return DbI18n::default();
    };
    if !response.status().is_success() {
        return DbI18n::default();
    }
    response.json().await.unwrap_or_default()
}

// ─── Synchronous accessors for use in components ───

/// Normalises a string for error-tolerant search: lowercases and strips
/// hyphens, apostrophes, and spaces so that e.g. "push-ups", "Pushups", and
/// "Push Ups" all collapse to the same canonical form.
fn normalize_for_search(s: &str) -> String {
    s.chars()
        .filter(|c| !matches!(c, '-' | '\'' | ' ' | '.'))
        .flat_map(char::to_lowercase)
        .collect()
}

/// Returns true if any whitespace-separated word in the (already-lowercased)
/// `tag` starts with `query_lower`.  Used for muscle / category / etc. tag
/// matching so that partial-word substrings elsewhere in a word are not hit
/// (e.g. "ring" must not match "hamst**ring**s").
fn tag_word_prefix_matches(tag: &str, query_lower: &str) -> bool {
    tag.split_whitespace()
        .any(|word| word.starts_with(query_lower))
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
/// Higher = better match. Name matches always outrank attribute matches.
const SCORE_EXACT_NAME: u32 = 100;
const SCORE_NAME_STARTS: u32 = 90;
const SCORE_NAME_NORM_EXACT: u32 = 85;
const SCORE_NAME_CONTAINS: u32 = 80;
const SCORE_NAME_NORM_TOKEN_START: u32 = 75;
const SCORE_NAME_NORM_CONTAINS: u32 = 70;
const SCORE_NAME_ALL_TOKENS: u32 = 65;
const SCORE_NAME_REVERSE: u32 = 60;
const SCORE_I18N_NAME: u32 = 55;
const SCORE_PRIMARY_MUSCLE: u32 = 50;
const SCORE_SECONDARY_MUSCLE: u32 = 45;
const SCORE_CATEGORY: u32 = 40;
const SCORE_FORCE: u32 = 35;
const SCORE_EQUIPMENT: u32 = 30;
const SCORE_LEVEL: u32 = 25;
const SCORE_ID_TOKENS: u32 = 20;
const SCORE_I18N_TAG: u32 = 15;

/// Computes a relevance score for `exercise` against the pre-computed query
/// components.  Returns 0 if the exercise does not match the query at all.
/// Name matches always outrank attribute (muscle/category/…) matches so that
/// e.g. "Push-Up" appears before "Bench Press" (force=push) when searching
/// "push-up".
fn score_exercise(
    exercise: &Exercise,
    query_lower: &str,
    query_norm: &str,
    tokens: &[String],
    db_i18n: Option<&DbI18n>,
) -> u32 {
    // Resolve pre-computed lowercase name.
    let computed_name_lower;
    let name_lc: &str = if exercise.name_lower.is_empty() {
        computed_name_lower = exercise.name.to_lowercase();
        &computed_name_lower
    } else {
        &exercise.name_lower
    };
    let name_norm = normalize_for_search(name_lc);

    // ── Name-based scoring (highest priority) ───────────────────────────────
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
    // Token-start: first token matches the beginning of the normalised name.
    if !tokens.is_empty() && name_norm.starts_with(tokens[0].as_str()) {
        return SCORE_NAME_NORM_TOKEN_START;
    }
    // Normalised name contains the full normalised query.
    if !query_norm.is_empty() && name_norm.contains(query_norm) {
        return SCORE_NAME_NORM_CONTAINS;
    }
    // All tokens appear (in any order) in the normalised name.
    if !tokens.is_empty() && tokens.iter().all(|t| name_norm.contains(t.as_str())) {
        return SCORE_NAME_ALL_TOKENS;
    }
    // Reverse check: normalised name is a substring of the normalised query
    // (handles over-specified queries like "bench presses" → "bench press").
    if !query_norm.is_empty() && !name_norm.is_empty() && query_norm.contains(&name_norm) {
        return SCORE_NAME_REVERSE;
    }

    // ── Localised name scoring ───────────────────────────────────────────────
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

    // ── Attribute-based scoring (lower priority) ─────────────────────────────
    if exercise
        .primary_muscles
        .iter()
        .any(|m| tag_word_prefix_matches(m.as_ref(), query_lower))
    {
        return SCORE_PRIMARY_MUSCLE;
    }
    if exercise
        .secondary_muscles
        .iter()
        .any(|m| tag_word_prefix_matches(m.as_ref(), query_lower))
    {
        return SCORE_SECONDARY_MUSCLE;
    }
    if exercise.category.as_ref().contains(query_lower) {
        return SCORE_CATEGORY;
    }
    if exercise
        .force
        .is_some_and(|f| f.as_ref().contains(query_lower))
    {
        return SCORE_FORCE;
    }
    {
        // Schema2: no-equipment means body-only.
        let equipment_str = match exercise.equipment {
            Some(e) => e.as_ref().to_owned(),
            None => "body only".to_owned(),
        };
        if equipment_str.contains(query_lower) {
            return SCORE_EQUIPMENT;
        }
    }
    if exercise
        .level
        .is_some_and(|l| l.as_ref().contains(query_lower))
    {
        return SCORE_LEVEL;
    }
    {
        // Schema2 normalised IDs as extra search tokens.
        let id_words = exercise.id.to_lowercase().replace(['_', '-'], " ");
        if id_words.contains(query_lower)
            || (!tokens.is_empty()
                && tokens.iter().all(|t| id_words.contains(t.as_str())))
        {
            return SCORE_ID_TOKENS;
        }
    }

    // ── Translated tag search ────────────────────────────────────────────────
    if db_i18n.is_some_and(|i18n| {
        let check_tag = |field: &str, english_val: &str| -> bool {
            i18n.values().any(|lang| {
                let translated = match field {
                    "category" => lang.category.get(english_val),
                    "force" => lang.force.get(english_val),
                    "equipment" => lang.equipment.get(english_val),
                    "level" => lang.level.get(english_val),
                    "muscles" => lang.muscles.get(english_val),
                    _ => None,
                };
                translated.is_some_and(|v| v.to_lowercase().contains(query_lower))
            })
        };
        check_tag("category", exercise.category.as_ref())
            || exercise
                .force
                .is_some_and(|f| check_tag("force", f.as_ref()))
            || exercise
                .equipment
                .is_some_and(|e| check_tag("equipment", e.as_ref()))
            || exercise
                .level
                .is_some_and(|l| check_tag("level", l.as_ref()))
            || exercise
                .primary_muscles
                .iter()
                .any(|m| check_tag("muscles", m.as_ref()))
            || exercise
                .secondary_muscles
                .iter()
                .any(|m| check_tag("muscles", m.as_ref()))
    }) {
        return SCORE_I18N_TAG;
    }

    0 // No match
}

/// Search exercises by name (English and all available localized names), muscle
/// groups, category, force, equipment, level, and ID tokens.
///
/// Results are sorted by relevance: exact / near-exact name matches appear
/// first, followed by prefix / token matches, then attribute (muscle, category,
/// etc.) matches.  This ensures that e.g. "Push-Up" is the first result when
/// searching "push-up" even though many other exercises have `force = push`.
///
/// When `db_i18n` is provided translated tag values (category, force,
/// equipment, level, muscles) in every available language are also searched,
/// enabling queries like "musculation" to find strength exercises.
pub fn search_exercises<'a>(
    exercises: &'a [Exercise],
    query: &str,
    db_i18n: Option<&crate::models::DbI18n>,
) -> Vec<&'a Exercise> {
    let query_lower = query.to_lowercase();
    let query_norm = normalize_for_search(query);
    // Split query into individual tokens for multi-word search: each token must
    // independently appear in the normalised name.  Only tokens that contain at
    // least one alphanumeric character after normalisation are kept so that
    // punctuation-only tokens (e.g. "…") are silently ignored.
    let tokens: Vec<String> = query_lower
        .split_whitespace()
        .map(normalize_for_search)
        .filter(|t| t.chars().any(char::is_alphanumeric))
        .collect();

    let mut scored: Vec<(u32, &Exercise)> = exercises
        .iter()
        .filter_map(|exercise| {
            let score =
                score_exercise(exercise, &query_lower, &query_norm, &tokens, db_i18n);
            if score > 0 {
                Some((score, exercise))
            } else {
                None
            }
        })
        .collect();

    // Sort descending by score so the best matches appear first.
    // Use a stable sort to preserve the original order among equal-score items
    // (preserves the DB ordering, which is often alphabetical).
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.into_iter().map(|(_, ex)| ex).collect()
}

// ─── Hard-filter types and helpers ───────────────────────────────────────────

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
                exercise.primary_muscles.contains(m)
                    || exercise.secondary_muscles.contains(m)
            }
        }
    }
}

/// Returns true when `exercise` passes **all** active filters.
///
/// Filters of the same variant form an OR group; OR groups are ANDed together.
pub fn exercise_matches_filters(exercise: &Exercise, filters: &[SearchFilter]) -> bool {
    if filters.is_empty() {
        return true;
    }
    // Build groups by iterating once to find the first unseen kind, then checking
    // all filters of that kind as an OR group.  The list is always ≤ 4 elements.
    let mut handled: Vec<&SearchFilter> = Vec::new();
    for filter in filters {
        if handled.iter().any(|h| h.same_kind(filter)) {
            continue; // already processed this group
        }
        handled.push(filter);
        // OR: exercise must satisfy at least one filter of this kind.
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

pub fn get_exercise_by_id<'a>(exercises: &'a [Exercise], id: &str) -> Option<&'a Exercise> {
    exercises.iter().find(|e| e.id == id)
}

/// Resolves an exercise by ID: checks the main DB slice first, then falls back
/// to the custom-exercises slice.  Centralises the lookup logic used across
/// multiple components.
pub fn resolve_exercise<'a>(
    db: &'a [Exercise],
    custom: &'a [Exercise],
    id: &str,
) -> Option<&'a Exercise> {
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
        let results = search_exercises(&exercises, "bench", None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "bench_press");
    }

    #[test]
    fn search_by_muscle() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "lats", None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "pull_up");
    }

    #[test]
    fn search_by_category() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "cardio", None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "running");
    }

    #[test]
    fn search_by_force() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "push", None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "bench_press");
    }

    #[test]
    fn search_by_equipment() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "barbell", None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "bench_press");
    }

    #[test]
    fn search_by_level() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "beginner", None);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn search_case_insensitive() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "BENCH", None);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn search_no_match() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "zzz_no_match", None);
        assert!(results.is_empty());
    }

    // ── Error-tolerant (normalised) search ────────────────────────────────

    #[test]
    fn search_hyphenated_query_finds_unhyphenated_name() {
        // "pull-up" (with hyphen) should find the exercise named "Pull-Up"
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "pull-up", None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "pull_up");
    }

    #[test]
    fn search_plain_query_finds_hyphenated_name() {
        // "pullup" (no hyphen) should also find the exercise named "Pull-Up"
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "pullup", None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "pull_up");
    }

    #[test]
    fn search_pluralised_query_finds_exercise() {
        // "bench presses" normalises to "benchpresses"; "benchpress" is a
        // substring of it, so "Bench Press" should be found.
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "bench press", None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "bench_press");
    }

    #[test]
    fn search_multi_word_finds_interleaved_words() {
        // "wide grip bench" should find an exercise named "Wide-Grip Barbell Bench Press"
        // because each token ("wide", "grip", "bench") appears in the normalised name.
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
        let results = search_exercises(&exercises, "wide grip bench", None);
        assert_eq!(
            results.len(),
            1,
            "token-based search should find the exercise"
        );
        assert_eq!(results[0].id, "wide_grip_bench");
    }

    #[test]
    fn search_punctuation_only_token_is_ignored() {
        // A query like "… pushups" should still find "Pushups" because the "…"
        // token contains no alphanumeric characters and is silently ignored.
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
        let results = search_exercises(&exercises, "… pushups", None);
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
        let results = search_exercises(&exercises, "", None);
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
        // A custom entry with the same id as a DB exercise — DB wins.
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
        assert_eq!(found.unwrap().name, "Pull-Up"); // DB entry wins
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
        // barbell and body only (running has None equipment)
        assert_eq!(equipment.len(), 2);
    }

    #[test]
    fn get_muscle_groups_deduplicates() {
        let exercises = sample_exercises();
        let muscles = get_muscle_groups(&exercises);
        // chest, lats, quadriceps, hamstrings
        assert_eq!(muscles.len(), 4);
    }

    #[test]
    fn search_with_none_force_does_not_match_force_query() {
        let exercises = sample_exercises();
        // "running" has force: None, should not match when searching for "pull"
        let results = search_exercises(&exercises, "pull", None);
        for r in &results {
            assert_ne!(r.id, "running");
        }
    }

    #[test]
    fn search_with_body_only_equipment_matches_both_explicit_and_none() {
        // Schema2: no-equipment (None) means body-only.  A "body only" search
        // must therefore find both exercises with Equipment::BodyOnly AND those
        // with equipment: None (i.e. all three sample exercises).
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "body only", None);
        // pull_up has Equipment::BodyOnly, running has None equipment
        let ids: Vec<&str> = results.iter().map(|e| e.id.as_str()).collect();
        assert!(ids.contains(&"pull_up"), "BodyOnly exercise should match");
        assert!(
            ids.contains(&"running"),
            "None-equipment exercise should match 'body only'"
        );
    }

    #[test]
    fn search_by_normalized_id() {
        // ID-based search: an exercise with id "kettlebell_pistol_squat" should
        // be found by the query "kettlebell" even when the name is abbreviated.
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
        let results = search_exercises(&exercises, "kettlebell", None);
        assert_eq!(
            results.len(),
            1,
            "should find exercise by normalized ID token"
        );
        assert_eq!(results[0].id, "kettlebell_pistol_squat");
    }

    #[test]
    fn search_by_secondary_muscle() {
        let exercises = sample_exercises();
        // "triceps" is a secondary muscle of bench_press
        let results = search_exercises(&exercises, "triceps", None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "bench_press");
    }

    #[test]
    fn search_by_secondary_muscle_biceps() {
        let exercises = sample_exercises();
        // "biceps" is a secondary muscle of pull_up
        let results = search_exercises(&exercises, "biceps", None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "pull_up");
    }

    #[test]
    fn search_muscle_word_start_no_false_positive() {
        // "ring" is a substring of "hamstrings" but the word "hamstrings" does
        // not *start* with "ring".  "running" (which has Hamstrings as a primary
        // muscle) must therefore not be returned when searching for "ring".
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "ring", None);
        assert!(!results.iter().any(|e| e.id == "running"));
    }

    #[test]
    fn search_muscle_word_start_prefix_matches() {
        // "ham" is a word-start prefix of "hamstrings", so "running" should match.
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "ham", None);
        assert!(results.iter().any(|e| e.id == "running"));
    }

    #[test]
    fn exercises_json_url_uses_fork() {
        // The default JSON endpoint references the gfauredev GitHub Pages
        // static website, which serves files with CORS headers.
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
        let last_fetch = now - 60; // 1 minute ago
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
        // elapsed = now - last_fetch = interval → satisfies `>= interval`, so refresh IS due
        assert!(is_refresh_due_for(now, Some(last_fetch)));
    }

    // ── Unified search tests (covers the unified search for custom exercises) ──

    #[test]
    fn search_custom_exercise_by_muscle_unified() {
        // search_exercises is used for both custom and DB exercises; verify it
        // finds custom exercises by primary muscle.
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
        let results = search_exercises(&exercises, "quadriceps", None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "custom_squat");
    }

    #[test]
    fn search_custom_exercise_by_secondary_muscle_unified() {
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
        let results = search_exercises(&exercises, "glutes", None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "custom_squat");
    }

    #[test]
    fn search_custom_exercise_by_category_unified() {
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
        // Search by category should match custom exercises too
        let results = search_exercises(&exercises, "cardio", None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "custom_run");
    }

    // ── Localized search tests ──────────────────────────────────────────────

    #[test]
    fn search_by_i18n_name() {
        // Bench Press has a French name "Développé couché"; searching the French
        // name should find it even though the English name doesn't match.
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
        let results = search_exercises(&exercises, "développé", None);
        assert_eq!(results.len(), 1, "should find by French name");
        assert_eq!(results[0].id, "bench_press");
    }

    #[test]
    fn search_by_translated_tag() {
        // "musculation" is the French translation of category "strength".
        // With db_i18n provided, searching "musculation" should find strength exercises.
        use crate::models::{DbI18n, DbI18nLang};
        let mut lang = DbI18nLang::default();
        lang.category
            .insert("strength".to_string(), "musculation".to_string());
        let mut db_i18n = DbI18n::new();
        db_i18n.insert("fr".to_string(), lang);

        let exercises = sample_exercises();
        // bench_press and pull_up are Category::Strength
        let results = search_exercises(&exercises, "musculation", Some(&db_i18n));
        let ids: Vec<&str> = results.iter().map(|e| e.id.as_str()).collect();
        assert!(ids.contains(&"bench_press"), "bench_press should match");
        assert!(ids.contains(&"pull_up"), "pull_up should match");
        assert!(!ids.contains(&"running"), "cardio should not match");
    }

    #[test]
    fn search_by_translated_tag_without_db_i18n_does_not_match() {
        // Without db_i18n, "musculation" should NOT match English exercises.
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "musculation", None);
        assert_eq!(
            results.len(),
            0,
            "should not match translated tag without db_i18n"
        );
    }

    // ── get_equipment_types / get_muscle_groups (test-only utilities) ──

    #[test]
    fn get_equipment_types_only_returns_some_equipment() {
        let exercises = sample_exercises();
        // running has equipment: None, so only barbell and body only appear
        let equipment = get_equipment_types(&exercises);
        assert!(equipment.iter().all(|e| !e.as_ref().is_empty()));
    }

    #[test]
    fn get_muscle_groups_only_returns_primary_muscles() {
        let exercises = sample_exercises();
        // Only primary muscles are collected
        let muscles = get_muscle_groups(&exercises);
        // chest (bench_press), lats (pull_up), quadriceps+hamstrings (running)
        assert_eq!(muscles.len(), 4);
    }

    // ── Native-platform (non-wasm) integration tests ─────────────────────────
    // These tests exercise the filesystem-backed functions that the coverage
    // gate checks.  A single static mutex serialises ALL tests that touch the
    // shared config file (LAST_FETCH_KEY and EXERCISE_DB_URL_STORAGE_KEY both
    // live in the same JSON file, so one lock is sufficient).

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
                "config value should be removed after clear_fetch_cache"
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
            record_fetch_timestamp(); // writes "now" to config

            assert!(
                !is_refresh_due(),
                "refresh should not be due immediately after recording a fresh timestamp"
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
            // Bind to an ephemeral port then drop the listener; connections will
            // be immediately refused.
            let port = {
                let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
                l.local_addr().unwrap().port()
            };
            // RAII guard ensures the URL key is cleaned up even on panic.
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
                "error message should mention 'HTTP error'"
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
                "error should mention HTTP 404, got: {err}"
            );
        }

        #[test]
        fn download_exercises_returns_empty_vec_on_200_empty_json() {
            let _g = cfg_lock();
            let body = b"[]";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
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

            assert!(result.is_ok(), "expected Ok([]), got: {result:?}");
            assert!(result.unwrap().is_empty());
        }

        #[test]
        fn download_db_i18n_returns_default_on_connection_refused() {
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
                result.is_empty(),
                "download_db_i18n should return empty map on connection error"
            );
        }
    }

    // ── merge_lang_entries unit tests ────────────────────────────────────────

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
            "unmatched entry should not create an i18n map"
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
            "URL should end with exercises.fr.json, got: {url}"
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

    // ── Search ordering / scoring tests ─────────────────────────────────────

    #[test]
    fn search_name_ranks_above_force_attribute() {
        // An exercise named "Push-Up" must appear BEFORE exercises that only
        // match via force=Push when the query is "push-up".
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
        let results = search_exercises(&exercises, "push-up", None);
        assert!(
            !results.is_empty(),
            "search should find at least the Push-Up exercise"
        );
        assert_eq!(
            results[0].id, "push_up",
            "Push-Up should be the first result when searching 'push-up'"
        );
    }

    #[test]
    fn search_exact_name_ranks_first() {
        // "Bench Press" typed exactly should appear before any other match.
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "bench press", None);
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "bench_press");
    }

    #[test]
    fn search_name_prefix_ranks_above_substring() {
        // "pull" should rank exercises whose name starts with "pull" above
        // those that merely contain "pull" elsewhere.
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
        let results = search_exercises(&exercises, "pull", None);
        assert!(results.len() >= 2);
        assert_eq!(
            results[0].id, "pull_up",
            "Pull-Up (starts with 'pull') should rank above 'Supine Pull' (contains 'pull')"
        );
    }

    // ── SearchFilter / exercise_matches_filters tests ────────────────────────

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
        // Triceps is a *secondary* muscle of bench_press.
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
        // Cardio OR Strength should return exercises from both categories.
        let exercises = sample_exercises();
        let filters = vec![
            SearchFilter::Category(Category::Cardio),
            SearchFilter::Category(Category::Strength),
        ];
        let results: Vec<_> = exercises
            .iter()
            .filter(|e| exercise_matches_filters(e, &filters))
            .collect();
        // All 3 exercises match: bench_press + pull_up (Strength) + running (Cardio)
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn filter_different_kinds_intersect() {
        // Category::Strength AND Force::Push → only bench_press.
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

    // ── detect_filter_suggestions tests ─────────────────────────────────────

    #[test]
    fn detect_filter_suggests_category_for_cardio() {
        let suggestions = detect_filter_suggestions("cardio");
        assert!(
            suggestions
                .iter()
                .any(|f| f == &SearchFilter::Category(Category::Cardio)),
            "should suggest Cardio category filter"
        );
    }

    #[test]
    fn detect_filter_suggests_muscle_prefix() {
        let suggestions = detect_filter_suggestions("bicep");
        assert!(
            suggestions
                .iter()
                .any(|f| f == &SearchFilter::Muscle(Muscle::Biceps)),
            "should suggest Biceps muscle filter for prefix 'bicep'"
        );
    }

    #[test]
    fn detect_filter_short_query_returns_empty() {
        let suggestions = detect_filter_suggestions("a");
        assert!(
            suggestions.is_empty(),
            "single-character query should return no suggestions"
        );
    }

    #[test]
    fn detect_filter_suggests_level_beginner() {
        let suggestions = detect_filter_suggestions("beginner");
        assert!(
            suggestions
                .iter()
                .any(|f| f == &SearchFilter::Level(Level::Beginner)),
            "should suggest Beginner level filter"
        );
    }

    #[test]
    fn filter_label_is_human_readable() {
        assert_eq!(
            SearchFilter::Category(Category::Cardio).label(),
            "🏷 cardio"
        );
        assert_eq!(SearchFilter::Force(Force::Push).label(), "⚡ push");
        assert_eq!(
            SearchFilter::Equipment(Equipment::Barbell).label(),
            "🔧 barbell"
        );
        assert_eq!(
            SearchFilter::Level(Level::Beginner).label(),
            "📊 beginner"
        );
        assert_eq!(
            SearchFilter::Muscle(Muscle::Biceps).label(),
            "💪 biceps"
        );
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
