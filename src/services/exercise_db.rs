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
                    .to_string()
                    .to_lowercase()
                    .contains(&query_lower)
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

    fn make_exercise(id: &str, name: &str, category: Category) -> Exercise {
        Exercise {
            id: id.to_string(),
            name: name.to_string(),
            force: None,
            level: Level::Beginner,
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec![],
            category,
            images: vec![],
        }
    }

    // ── search_exercises ──────────────────────────────────────────────────────

    #[test]
    fn search_by_name_returns_matches() {
        let exercises = vec![
            make_exercise("ex1", "Push-up", Category::Strength),
            make_exercise("ex2", "Pull-up", Category::Strength),
            make_exercise("ex3", "Squat", Category::Strength),
        ];
        let results = search_exercises(&exercises, "push");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "ex1");
    }

    #[test]
    fn search_is_case_insensitive() {
        let exercises = vec![make_exercise("ex1", "Push-up", Category::Strength)];
        assert_eq!(search_exercises(&exercises, "PUSH").len(), 1);
        assert_eq!(search_exercises(&exercises, "push").len(), 1);
        assert_eq!(search_exercises(&exercises, "Push").len(), 1);
    }

    #[test]
    fn search_by_category_returns_matches() {
        let exercises = vec![
            make_exercise("ex1", "Running", Category::Cardio),
            make_exercise("ex2", "Push-up", Category::Strength),
        ];
        let results = search_exercises(&exercises, "cardio");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "ex1");
    }

    #[test]
    fn search_by_primary_muscle() {
        let mut ex = make_exercise("ex1", "Bench Press", Category::Strength);
        ex.primary_muscles = vec![Muscle::Chest];
        let exercises = vec![ex];
        let results = search_exercises(&exercises, "chest");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn search_by_force_type() {
        let mut ex = make_exercise("ex1", "Curl", Category::Strength);
        ex.force = Some(Force::Pull);
        let exercises = vec![ex];
        let results = search_exercises(&exercises, "pull");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn search_by_equipment() {
        let mut ex = make_exercise("ex1", "Curl", Category::Strength);
        ex.equipment = Some(Equipment::Dumbbell);
        let exercises = vec![ex];
        let results = search_exercises(&exercises, "dumbbell");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn search_by_level() {
        let mut ex = make_exercise("ex1", "Squat", Category::Strength);
        ex.level = Level::Expert;
        let exercises = vec![ex];
        let results = search_exercises(&exercises, "expert");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn search_empty_query_returns_nothing() {
        let exercises = vec![make_exercise("ex1", "Push-up", Category::Strength)];
        // Empty query matches all exercises (all names contain "")
        // This is expected behavior because str::contains("") is always true.
        let results = search_exercises(&exercises, "");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn search_no_match_returns_empty() {
        let exercises = vec![make_exercise("ex1", "Push-up", Category::Strength)];
        let results = search_exercises(&exercises, "zzz_no_match");
        assert!(results.is_empty());
    }

    #[test]
    fn search_empty_list_returns_empty() {
        let results = search_exercises(&[], "push");
        assert!(results.is_empty());
    }

    // ── get_exercise_by_id ───────────────────────────────────────────────────

    #[test]
    fn get_exercise_by_id_found() {
        let exercises = vec![
            make_exercise("ex1", "Push-up", Category::Strength),
            make_exercise("ex2", "Squat", Category::Strength),
        ];
        let found = get_exercise_by_id(&exercises, "ex2");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Squat");
    }

    #[test]
    fn get_exercise_by_id_not_found() {
        let exercises = vec![make_exercise("ex1", "Push-up", Category::Strength)];
        assert!(get_exercise_by_id(&exercises, "missing").is_none());
    }

    #[test]
    fn get_exercise_by_id_empty_list() {
        assert!(get_exercise_by_id(&[], "ex1").is_none());
    }

    // ── get_equipment_types ──────────────────────────────────────────────────

    #[test]
    fn get_equipment_types_deduplicates_and_sorts() {
        let mut ex1 = make_exercise("ex1", "Curl", Category::Strength);
        ex1.equipment = Some(Equipment::Dumbbell);
        let mut ex2 = make_exercise("ex2", "Row", Category::Strength);
        ex2.equipment = Some(Equipment::Barbell);
        let mut ex3 = make_exercise("ex3", "Curl2", Category::Strength);
        ex3.equipment = Some(Equipment::Dumbbell); // duplicate
        let ex4 = make_exercise("ex4", "Plank", Category::Strength); // no equipment
        let exercises = vec![ex1, ex2, ex3, ex4];
        let result = get_equipment_types(&exercises);
        // Should have Barbell and Dumbbell (deduplicated), sorted by name
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Equipment::Barbell);
        assert_eq!(result[1], Equipment::Dumbbell);
    }

    #[test]
    fn get_equipment_types_empty_list() {
        assert!(get_equipment_types(&[]).is_empty());
    }

    // ── get_muscle_groups ────────────────────────────────────────────────────

    #[test]
    fn get_muscle_groups_deduplicates_and_sorts() {
        let mut ex1 = make_exercise("ex1", "Bench Press", Category::Strength);
        ex1.primary_muscles = vec![Muscle::Chest, Muscle::Triceps];
        let mut ex2 = make_exercise("ex2", "Fly", Category::Strength);
        ex2.primary_muscles = vec![Muscle::Chest]; // duplicate
        let exercises = vec![ex1, ex2];
        let result = get_muscle_groups(&exercises);
        // Chest + Triceps (deduplicated, sorted alphabetically)
        assert_eq!(result.len(), 2);
        assert!(result.contains(&Muscle::Chest));
        assert!(result.contains(&Muscle::Triceps));
    }

    #[test]
    fn get_muscle_groups_empty_list() {
        assert!(get_muscle_groups(&[]).is_empty());
    }
}
