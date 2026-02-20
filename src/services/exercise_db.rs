use crate::models::{Equipment, Exercise, Muscle};
use dioxus::prelude::*;

/// Number of seconds between automatic exercise database refreshes (7 days).
#[cfg(target_arch = "wasm32")]
const EXERCISE_DB_REFRESH_INTERVAL_SECS: f64 = 7.0 * 24.0 * 60.0 * 60.0;

/// localStorage key used to track when exercises were last downloaded.
#[cfg(target_arch = "wasm32")]
const LAST_FETCH_KEY: &str = "exercise_db_last_fetch";

/// Milliseconds per second – used when converting `Date.now()` to Unix seconds.
#[cfg(target_arch = "wasm32")]
const MILLIS_PER_SECOND: f64 = 1000.0;

#[cfg(target_arch = "wasm32")]
fn exercises_json_url() -> String {
    format!("{}dist/exercises.json", crate::utils::EXERCISE_DB_BASE_URL)
}

/// Provide the exercises signal in the Dioxus context.
/// On first launch, downloads exercises from the API and stores them in IndexedDB.
/// On subsequent launches, loads from IndexedDB.
pub fn provide_exercises() {
    let sig: Signal<Vec<Exercise>> = use_context_provider(|| Signal::new(Vec::new()));

    spawn(async move {
        load_exercises(sig).await;
    });
}

pub fn use_exercises() -> Signal<Vec<Exercise>> {
    use_context::<Signal<Vec<Exercise>>>()
}

#[allow(unused_mut, unused_variables)]
async fn load_exercises(mut sig: Signal<Vec<Exercise>>) {
    // 1. Try IndexedDB for immediate display
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

        // 2. Download from the network (first run or periodic refresh)
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

    // No exercises available: database will remain empty until next launch or network becomes available
    log::warn!("No exercises available: failed to load from IndexedDB and download from API");
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

#[cfg(target_arch = "wasm32")]
async fn download_exercises() -> Result<Vec<Exercise>, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, Response};

    let window = web_sys::window().ok_or("no window")?;
    let opts = RequestInit::new();
    opts.set_method("GET");

    let request = Request::new_with_str_and_init(&exercises_json_url(), &opts)
        .map_err(|e| format!("{:?}", e))?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("{:?}", e))?;

    let resp: Response = resp_value.dyn_into().map_err(|_| "not a Response")?;

    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let text = JsFuture::from(resp.text().map_err(|e| format!("{:?}", e))?)
        .await
        .map_err(|e| format!("{:?}", e))?;

    let text_str = text.as_string().ok_or("response not a string")?;

    serde_json::from_str::<Vec<Exercise>>(&text_str).map_err(|e| format!("JSON parse error: {}", e))
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

#[allow(dead_code)]
pub fn get_equipment_types(exercises: &[Exercise]) -> Vec<Equipment> {
    let mut equipment: Vec<Equipment> = exercises.iter().filter_map(|e| e.equipment).collect();
    equipment.sort_by_key(|a| a.to_string());
    equipment.dedup();
    equipment
}

#[allow(dead_code)]
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
        #[cfg(target_arch = "wasm32")]
        {
            let url = exercises_json_url();
            assert!(
                url.contains("gfauredev"),
                "Expected gfauredev fork URL, got: {url}"
            );
            assert!(url.ends_with("dist/exercises.json"));
        }
        // On non-wasm we verify the base URL constant instead.
        #[cfg(not(target_arch = "wasm32"))]
        {
            assert!(crate::utils::EXERCISE_DB_BASE_URL.contains("gfauredev"));
        }
    }
}
