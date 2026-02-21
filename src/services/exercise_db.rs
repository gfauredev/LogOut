use crate::models::Exercise;
#[cfg(test)]
use crate::models::{Equipment, Muscle};
use dioxus::prelude::*;

/// Newtype wrapper for the exercise-database signal so its `TypeId` is distinct
/// from the `Signal<Vec<Exercise>>` used by `storage::provide_app_state` for
/// custom exercises.  Without this wrapper both `use_context_provider` calls in
/// `App` would share the same context slot, causing both signals to point at the
/// same `Signal<Vec<Exercise>>` and leading to doubled counts, missing DB
/// exercises, and all exercises being treated as custom.
#[derive(Clone, Copy)]
struct AllExercisesSignal(Signal<Vec<Exercise>>);

/// Number of seconds between automatic exercise database refreshes (7 days).
#[cfg(target_arch = "wasm32")]
const EXERCISE_DB_REFRESH_INTERVAL_SECS: f64 = 7.0 * 24.0 * 60.0 * 60.0;

/// Number of seconds between automatic exercise database refreshes (7 days).
#[cfg(not(target_arch = "wasm32"))]
const EXERCISE_DB_REFRESH_INTERVAL_SECS: u64 = 7 * 24 * 60 * 60;

/// Storage key used to track when exercises were last downloaded
/// (localStorage on WASM, config file on native).
const LAST_FETCH_KEY: &str = "exercise_db_last_fetch";

/// Milliseconds per second – used when converting `Date.now()` to Unix seconds.
#[cfg(target_arch = "wasm32")]
const MILLIS_PER_SECOND: f64 = 1000.0;

/// Returns the URL for the exercises JSON file.
/// Available on all platforms; `get_exercise_db_url()` handles per-platform config.
fn exercises_json_url() -> String {
    format!("{}dist/exercises.json", crate::utils::get_exercise_db_url())
}

/// Provide the exercises signal in the Dioxus context.
/// On first launch, downloads exercises from the API and stores them in IndexedDB
/// (web) or a local file (native).  On subsequent launches, loads from cache.
pub fn provide_exercises() {
    let wrapper = use_context_provider(|| AllExercisesSignal(Signal::new(Vec::new())));
    let sig = wrapper.0;

    spawn(async move {
        load_exercises(sig).await;
    });
}

pub fn use_exercises() -> Signal<Vec<Exercise>> {
    use_context::<AllExercisesSignal>().0
}

#[allow(unused_mut, unused_variables)]
async fn load_exercises(mut sig: Signal<Vec<Exercise>>) {
    // ── Web platform (wasm32 + IndexedDB) ────────────────────────────────────
    #[cfg(target_arch = "wasm32")]
    {
        use crate::services::storage::idb_exercises;

        let cached = idb_exercises::get_all_exercises().await.unwrap_or_default();
        let needs_refresh = !cached.is_empty() && is_refresh_due();

        if !cached.is_empty() {
            log::info!("Loaded {} exercises from IndexedDB", cached.len());
            sig.set(cached);

            if !needs_refresh {
                return;
            }

            // Re-fetch in the background to keep exercises up to date
            log::info!("Exercise database is stale – refreshing in background");
        }

        // Download from the network (first run or periodic refresh)
        match download_exercises().await {
            Ok(exercises) if !exercises.is_empty() => {
                log::info!(
                    "Downloaded {} exercises, storing in IndexedDB",
                    exercises.len()
                );
                idb_exercises::store_all_exercises(&exercises).await;
                record_fetch_timestamp();
                sig.set(exercises);
                return;
            }
            Ok(_) => log::warn!("Downloaded exercises file was empty"),
            Err(e) => log::warn!("Failed to download exercises: {:?}", e),
        }
    }

    // ── Native platform (Android / desktop) ──────────────────────────────────
    #[cfg(not(target_arch = "wasm32"))]
    {
        use crate::services::storage::native_exercises;

        let cached = native_exercises::get_all_exercises();
        let needs_refresh = !cached.is_empty() && is_refresh_due();

        if !cached.is_empty() {
            log::info!("Loaded {} exercises from local file", cached.len());
            sig.set(cached);

            if !needs_refresh {
                return;
            }

            log::info!("Exercise database is stale – refreshing in background");
        }

        match download_exercises().await {
            Ok(exercises) if !exercises.is_empty() => {
                log::info!(
                    "Downloaded {} exercises, storing in local file",
                    exercises.len()
                );
                native_exercises::store_all_exercises(&exercises);
                record_fetch_timestamp();
                sig.set(exercises);
                return;
            }
            Ok(_) => log::warn!("Downloaded exercises file was empty"),
            Err(e) => log::warn!("Failed to download exercises: {:?}", e),
        }
    }

    // No exercises available: database will remain empty until next launch or network becomes available
    log::warn!("No exercises available: failed to load from cache and download from API");
}

/// Returns true when the locally-cached exercise list is older than
/// [`EXERCISE_DB_REFRESH_INTERVAL_SECS`] or has never been fetched.
#[cfg(target_arch = "wasm32")]
fn is_refresh_due() -> bool {
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
    let now = js_sys::Date::now() / MILLIS_PER_SECOND;
    (now - last_fetch) >= EXERCISE_DB_REFRESH_INTERVAL_SECS
}

/// Returns true when the locally-cached exercise list is older than
/// [`EXERCISE_DB_REFRESH_INTERVAL_SECS`] or has never been fetched.
#[cfg(not(target_arch = "wasm32"))]
fn is_refresh_due() -> bool {
    use crate::services::storage::native_storage;
    let last_fetch =
        native_storage::get_config_value(LAST_FETCH_KEY).and_then(|s| s.parse::<u64>().ok());
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    is_refresh_due_for(now, last_fetch)
}

/// Pure helper: returns true when a refresh is due given the current time and the
/// last-fetch timestamp (both as Unix seconds).  Extracted for unit-testability.
fn is_refresh_due_for(now_secs: u64, last_fetch_secs: Option<u64>) -> bool {
    match last_fetch_secs {
        None => true,
        Some(last) => now_secs.saturating_sub(last) >= EXERCISE_DB_REFRESH_INTERVAL_SECS,
    }
}

/// Stores the current timestamp as the last exercise-fetch time.
#[cfg(target_arch = "wasm32")]
fn record_fetch_timestamp() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return;
    };
    let now = (js_sys::Date::now() / MILLIS_PER_SECOND).to_string();
    let _ = storage.set_item(LAST_FETCH_KEY, &now);
}

/// Stores the current timestamp as the last exercise-fetch time.
#[cfg(not(target_arch = "wasm32"))]
fn record_fetch_timestamp() {
    use crate::services::storage::native_storage;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string();
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

/// Downloads the exercises JSON from the configured URL using `reqwest`.
/// Works on all platforms: reqwest uses the browser's `fetch` on WASM and
/// native TLS on Android / desktop.
async fn download_exercises() -> Result<Vec<Exercise>, String> {
    let url = exercises_json_url();
    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("HTTP error: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    response
        .json::<Vec<Exercise>>()
        .await
        .map_err(|e| format!("JSON parse error: {e}"))
}

// ─── Synchronous accessors for use in components ───

pub fn search_exercises(exercises: &[Exercise], query: &str) -> Vec<Exercise> {
    let query_lower = query.to_lowercase();
    exercises
        .iter()
        .filter(|exercise| {
            exercise.name.to_lowercase().contains(&query_lower)
                || exercise
                    .primary_muscles
                    .iter()
                    .any(|m| m.as_str().contains(&query_lower))
                || exercise
                    .secondary_muscles
                    .iter()
                    .any(|m| m.as_str().contains(&query_lower))
                || exercise.category.as_str().contains(&query_lower)
                || exercise
                    .force
                    .map(|f| f.as_str().contains(&query_lower))
                    .unwrap_or(false)
                || exercise
                    .equipment
                    .map(|e| e.as_str().contains(&query_lower))
                    .unwrap_or(false)
                || exercise
                    .level
                    .map(|l| l.as_str().contains(&query_lower))
                    .unwrap_or(false)
        })
        .cloned()
        .collect()
}

pub fn get_exercise_by_id<'a>(exercises: &'a [Exercise], id: &str) -> Option<&'a Exercise> {
    exercises.iter().find(|e| e.id == id)
}

#[cfg(test)]
pub fn get_equipment_types(exercises: &[Exercise]) -> Vec<Equipment> {
    let mut equipment: Vec<Equipment> = exercises.iter().filter_map(|e| e.equipment).collect();
    equipment.sort_by_key(|a| a.to_string());
    equipment.dedup();
    equipment
}

#[cfg(test)]
pub fn get_muscle_groups(exercises: &[Exercise]) -> Vec<Muscle> {
    let mut muscles: Vec<Muscle> = exercises
        .iter()
        .flat_map(|e| e.primary_muscles.iter().copied())
        .collect();
    muscles.sort_by_key(|a| a.to_string());
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
                force: Some(Force::Push),
                level: Some(Level::Intermediate),
                mechanic: None,
                equipment: Some(Equipment::Barbell),
                primary_muscles: vec![Muscle::Chest],
                secondary_muscles: vec![Muscle::Triceps],
                instructions: vec![],
                category: Category::Strength,
                images: vec![],
            },
            Exercise {
                id: "pull_up".into(),
                name: "Pull-Up".into(),
                force: Some(Force::Pull),
                level: Some(Level::Beginner),
                mechanic: None,
                equipment: Some(Equipment::BodyOnly),
                primary_muscles: vec![Muscle::Lats],
                secondary_muscles: vec![Muscle::Biceps],
                instructions: vec![],
                category: Category::Strength,
                images: vec![],
            },
            Exercise {
                id: "running".into(),
                name: "Running".into(),
                force: None,
                level: Some(Level::Beginner),
                mechanic: None,
                equipment: None,
                primary_muscles: vec![Muscle::Quadriceps, Muscle::Hamstrings],
                secondary_muscles: vec![],
                instructions: vec![],
                category: Category::Cardio,
                images: vec![],
            },
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
    fn search_by_muscle() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "lats");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "pull_up");
    }

    #[test]
    fn search_by_category() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "cardio");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "running");
    }

    #[test]
    fn search_by_force() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "push");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "bench_press");
    }

    #[test]
    fn search_by_equipment() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "barbell");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "bench_press");
    }

    #[test]
    fn search_by_level() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "beginner");
        assert_eq!(results.len(), 2);
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
        let results = search_exercises(&exercises, "pull");
        for r in &results {
            assert_ne!(r.id, "running");
        }
    }

    #[test]
    fn search_with_none_equipment_does_not_match_equipment_query() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "body only");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "pull_up");
    }

    #[test]
    fn search_by_secondary_muscle() {
        let exercises = sample_exercises();
        // "triceps" is a secondary muscle of bench_press
        let results = search_exercises(&exercises, "triceps");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "bench_press");
    }

    #[test]
    fn search_by_secondary_muscle_biceps() {
        let exercises = sample_exercises();
        // "biceps" is a secondary muscle of pull_up
        let results = search_exercises(&exercises, "biceps");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "pull_up");
    }

    #[test]
    fn exercises_json_url_uses_fork() {
        // The JSON endpoint must reference the gfauredev fork (SSOT).
        // exercises_json_url() is now cross-platform; test it on all targets.
        let url = exercises_json_url();
        assert!(
            url.contains("gfauredev"),
            "Expected gfauredev fork URL, got: {url}"
        );
        assert!(url.ends_with("dist/exercises.json"));
    }

    #[test]
    fn is_refresh_due_true_when_no_timestamp() {
        assert!(is_refresh_due_for(1_000_000, None));
    }

    #[test]
    fn is_refresh_due_false_when_recent() {
        let now = 1_000_000u64;
        let last_fetch = now - 60; // 1 minute ago
        assert!(!is_refresh_due_for(now, Some(last_fetch)));
    }

    #[test]
    fn is_refresh_due_true_when_stale() {
        let interval = EXERCISE_DB_REFRESH_INTERVAL_SECS;
        let now = interval + 1_000_000;
        let last_fetch = 1_000_000u64;
        assert!(is_refresh_due_for(now, Some(last_fetch)));
    }

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
            force: Some(Force::Push),
            level: Some(Level::Beginner),
            mechanic: None,
            equipment: None,
            primary_muscles: vec![Muscle::Quadriceps],
            secondary_muscles: vec![Muscle::Glutes],
            instructions: vec![],
            category: Category::Strength,
            images: vec![],
        }];
        let results = search_exercises(&exercises, "quadriceps");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "custom_squat");
    }

    #[test]
    fn search_custom_exercise_by_secondary_muscle_unified() {
        let exercises = vec![Exercise {
            id: "custom_squat".into(),
            name: "Custom Squat".into(),
            force: Some(Force::Push),
            level: Some(Level::Beginner),
            mechanic: None,
            equipment: None,
            primary_muscles: vec![Muscle::Quadriceps],
            secondary_muscles: vec![Muscle::Glutes],
            instructions: vec![],
            category: Category::Strength,
            images: vec![],
        }];
        let results = search_exercises(&exercises, "glutes");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "custom_squat");
    }

    #[test]
    fn search_custom_exercise_by_category_unified() {
        let exercises = vec![Exercise {
            id: "custom_run".into(),
            name: "My Run".into(),
            force: None,
            level: None,
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec![],
            category: Category::Cardio,
            images: vec![],
        }];
        // Search by category should match custom exercises too
        let results = search_exercises(&exercises, "cardio");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "custom_run");
    }

    // ── get_equipment_types / get_muscle_groups (test-only utilities) ──

    #[test]
    fn get_equipment_types_only_returns_some_equipment() {
        let exercises = sample_exercises();
        // running has equipment: None, so only barbell and body only appear
        let equipment = get_equipment_types(&exercises);
        assert!(equipment.iter().all(|e| e.as_str().len() > 0));
    }

    #[test]
    fn get_muscle_groups_only_returns_primary_muscles() {
        let exercises = sample_exercises();
        // Only primary muscles are collected
        let muscles = get_muscle_groups(&exercises);
        // chest (bench_press), lats (pull_up), quadriceps+hamstrings (running)
        assert_eq!(muscles.len(), 4);
    }
}
