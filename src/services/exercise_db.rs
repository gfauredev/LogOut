use crate::models::{Equipment, Exercise, Muscle};
use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
const EXERCISES_JSON_URL: &str =
    "https://raw.githubusercontent.com/yuhonas/free-exercise-db/main/dist/exercises.json";

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
    // 1. Try IndexedDB
    #[cfg(target_arch = "wasm32")]
    {
        use crate::services::storage::idb_exercises;
        if let Ok(exercises) = idb_exercises::get_all_exercises().await {
            if !exercises.is_empty() {
                log::info!("Loaded {} exercises from IndexedDB", exercises.len());
                sig.set(exercises);
                return;
            }
        }

        // 2. Try downloading from API
        match download_exercises().await {
            Ok(exercises) if !exercises.is_empty() => {
                log::info!(
                    "Downloaded {} exercises, storing in IndexedDB",
                    exercises.len()
                );
                // Store all in IndexedDB for next time
                idb_exercises::store_all_exercises(&exercises).await;
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

#[cfg(target_arch = "wasm32")]
async fn download_exercises() -> Result<Vec<Exercise>, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, Response};

    let window = web_sys::window().ok_or("no window")?;
    let opts = RequestInit::new();
    opts.set_method("GET");

    let request = Request::new_with_str_and_init(EXERCISES_JSON_URL, &opts)
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
                    .any(|m| m.to_string().to_lowercase().contains(&query_lower))
                || exercise
                    .category
                    .to_string()
                    .to_lowercase()
                    .contains(&query_lower)
                || exercise
                    .force
                    .as_ref()
                    .map(|f| f.to_string().to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
                || exercise
                    .equipment
                    .as_ref()
                    .map(|e| e.to_string().to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
                || exercise
                    .level
                    .map(|l| l.to_string().to_lowercase().contains(&query_lower))
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
}
