use dioxus::prelude::*;
use crate::models::Exercise;

// Include the generated exercises data from build.rs as a fallback
include!(concat!(env!("OUT_DIR"), "/exercises_data.rs"));

const EXERCISES_JSON_URL: &str = "https://raw.githubusercontent.com/yuhonas/free-exercise-db/main/dist/exercises.json";

/// Provide the exercises signal in the Dioxus context.
/// On first launch, downloads exercises from the API and stores them in IndexedDB.
/// On subsequent launches, loads from IndexedDB.
/// Falls back to the embedded copy if both fail.
pub fn provide_exercises() {
    let sig: Signal<Vec<Exercise>> = use_context_provider(|| Signal::new(Vec::new()));

    spawn(async move {
        load_exercises(sig).await;
    });
}

pub fn use_exercises() -> Signal<Vec<Exercise>> {
    use_context::<Signal<Vec<Exercise>>>()
}

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
                log::info!("Downloaded {} exercises, storing in IndexedDB", exercises.len());
                // Store all in IndexedDB for next time
                idb_exercises::store_all_exercises(&exercises).await;
                sig.set(exercises);
                return;
            }
            Ok(_) => log::warn!("Downloaded exercises file was empty"),
            Err(e) => log::warn!("Failed to download exercises: {:?}", e),
        }
    }

    // 3. Fallback to embedded data
    log::info!("Using embedded exercises as fallback");
    let exercises = parse_embedded_exercises();

    #[cfg(target_arch = "wasm32")]
    {
        use crate::services::storage::idb_exercises;
        // Store embedded exercises so next load is from IDB
        idb_exercises::store_all_exercises(&exercises).await;
    }

    sig.set(exercises);
}

fn parse_embedded_exercises() -> Vec<Exercise> {
    serde_json::from_str(EXERCISES_JSON).unwrap_or_else(|e| {
        log::error!("Failed to parse embedded exercises: {}", e);
        vec![]
    })
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

    serde_json::from_str::<Vec<Exercise>>(&text_str)
        .map_err(|e| format!("JSON parse error: {}", e))
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
                    .any(|m| m.to_lowercase().contains(&query_lower))
                || exercise.category.to_lowercase().contains(&query_lower)
        })
        .cloned()
        .collect()
}

pub fn get_exercise_by_id<'a>(exercises: &'a [Exercise], id: &str) -> Option<&'a Exercise> {
    exercises.iter().find(|e| e.id == id)
}

pub fn get_equipment_types(exercises: &[Exercise]) -> Vec<String> {
    let mut equipment: Vec<String> = exercises
        .iter()
        .filter_map(|e| e.equipment.clone())
        .collect();
    equipment.sort();
    equipment.dedup();
    equipment
}

pub fn get_muscle_groups(exercises: &[Exercise]) -> Vec<String> {
    let mut muscles: Vec<String> = exercises
        .iter()
        .flat_map(|e| e.primary_muscles.clone())
        .collect();
    muscles.sort();
    muscles.dedup();
    muscles
}
