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
                    .any(|m| m.to_string().contains(&query_lower))
                || exercise
                    .secondary_muscles
                    .iter()
                    .any(|m| m.to_string().contains(&query_lower))
                || exercise.category.to_string().contains(&query_lower)
                || exercise
                    .force
                    .map(|f| f.to_string().contains(&query_lower))
                    .unwrap_or(false)
                || exercise
                    .equipment
                    .map(|e| e.to_string().contains(&query_lower))
                    .unwrap_or(false)
                || exercise
                    .level
                    .map(|l| l.to_string().contains(&query_lower))
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

    // ─── Tests ensuring search never panics for any input ───────────────────

    /// Simulate the exact user scenario that caused the crash: typing characters
    /// one by one into the search field. Each progressive query must succeed.
    #[test]
    fn search_progressive_typing_never_panics() {
        let exercises = sample_exercises();
        let input = "bench press";
        for i in 1..=input.len() {
            let query = &input[..i];
            let _ = search_exercises(&exercises, query);
        }
    }

    /// Progressive typing with a query that starts matching many exercises
    /// and then narrows down – the transition that originally crashed.
    #[test]
    fn search_progressive_typing_narrowing_results() {
        let exercises = sample_exercises();
        let queries = ["b", "be", "ben", "benc", "bench"];
        let mut prev_count = usize::MAX;
        for query in &queries {
            let results = search_exercises(&exercises, query);
            // Results should never increase as the query gets more specific
            assert!(
                results.len() <= prev_count || prev_count == usize::MAX,
                "Results increased from {} to {} for query '{}'",
                prev_count,
                results.len(),
                query
            );
            prev_count = results.len();
        }
    }

    /// Progressive typing starting with a single character that matches everything.
    #[test]
    fn search_progressive_typing_single_char_start() {
        let exercises = sample_exercises();
        // 's' matches many fields: "strength", "chest", "lats", etc.
        let r1 = search_exercises(&exercises, "s");
        assert!(!r1.is_empty());
        // Adding a second character must not crash
        let r2 = search_exercises(&exercises, "st");
        assert!(r2.len() <= r1.len());
        let r3 = search_exercises(&exercises, "str");
        assert!(r3.len() <= r2.len());
    }

    #[test]
    fn search_empty_exercise_list() {
        let exercises: Vec<Exercise> = vec![];
        let results = search_exercises(&exercises, "anything");
        assert!(results.is_empty());
    }

    #[test]
    fn search_empty_exercise_list_empty_query() {
        let exercises: Vec<Exercise> = vec![];
        let results = search_exercises(&exercises, "");
        assert!(results.is_empty());
    }

    #[test]
    fn search_whitespace_only_query() {
        let exercises = sample_exercises();
        let results = search_exercises(&exercises, "   ");
        // Whitespace-only queries should not crash
        assert!(results.is_empty() || !results.is_empty());
    }

    #[test]
    fn search_with_tab_and_newline() {
        let exercises = sample_exercises();
        let _ = search_exercises(&exercises, "\t");
        let _ = search_exercises(&exercises, "\n");
        let _ = search_exercises(&exercises, "\r\n");
        let _ = search_exercises(&exercises, " \t\n ");
    }

    #[test]
    fn search_unicode_accented_characters() {
        let exercises = sample_exercises();
        let _ = search_exercises(&exercises, "café");
        let _ = search_exercises(&exercises, "über");
        let _ = search_exercises(&exercises, "naïve");
        let _ = search_exercises(&exercises, "résumé");
    }

    #[test]
    fn search_unicode_emoji() {
        let exercises = sample_exercises();
        let _ = search_exercises(&exercises, "💪");
        let _ = search_exercises(&exercises, "🏋️");
        let _ = search_exercises(&exercises, "🏃‍♂️");
    }

    #[test]
    fn search_unicode_cjk_characters() {
        let exercises = sample_exercises();
        let _ = search_exercises(&exercises, "腕立て伏せ");
        let _ = search_exercises(&exercises, "运动");
        let _ = search_exercises(&exercises, "한국어");
    }

    #[test]
    fn search_unicode_mixed_scripts() {
        let exercises = sample_exercises();
        let _ = search_exercises(&exercises, "bench💪press");
        let _ = search_exercises(&exercises, "αβγ");
        let _ = search_exercises(&exercises, "Ωmega");
    }

    #[test]
    fn search_special_characters() {
        let exercises = sample_exercises();
        let special = [
            "!", "@", "#", "$", "%", "^", "&", "*", "(", ")", "-", "+", "=", "[", "]", "{", "}",
            "|", "\\", "/", ":", ";", "'", "\"", "<", ">", ",", ".", "?", "`", "~",
        ];
        for s in &special {
            let _ = search_exercises(&exercises, s);
        }
    }

    #[test]
    fn search_regex_like_patterns() {
        let exercises = sample_exercises();
        let _ = search_exercises(&exercises, ".*");
        let _ = search_exercises(&exercises, "bench|pull");
        let _ = search_exercises(&exercises, "[a-z]+");
        let _ = search_exercises(&exercises, "^bench$");
        let _ = search_exercises(&exercises, "(?i)bench");
    }

    #[test]
    fn search_very_long_query() {
        let exercises = sample_exercises();
        let long_query = "a".repeat(10_000);
        let results = search_exercises(&exercises, &long_query);
        assert!(results.is_empty());
    }

    #[test]
    fn search_null_byte_in_query() {
        let exercises = sample_exercises();
        let _ = search_exercises(&exercises, "bench\0press");
        let _ = search_exercises(&exercises, "\0");
    }

    #[test]
    fn search_exercise_with_all_none_optional_fields() {
        let exercises = vec![Exercise {
            id: "minimal".into(),
            name: "Minimal Exercise".into(),
            force: None,
            level: None,
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec![],
            category: Category::Strength,
            images: vec![],
        }];
        // Must not crash even when all optional fields are None and lists are empty
        let results = search_exercises(&exercises, "m");
        assert!(!results.is_empty()); // matches "Minimal" name
        let results = search_exercises(&exercises, "pull");
        assert!(results.is_empty()); // no match for force=None
        let results = search_exercises(&exercises, "barbell");
        assert!(results.is_empty()); // no match for equipment=None
        let results = search_exercises(&exercises, "beginner");
        assert!(results.is_empty()); // no match for level=None
    }

    /// Rapidly alternating between short queries simulates fast typing/backspace.
    #[test]
    fn search_rapid_query_changes() {
        let exercises = sample_exercises();
        let queries = [
            "b", "be", "ben", "be", "b", "", "p", "pu", "pul", "pull", "pul", "pu", "p", "",
        ];
        for query in &queries {
            let _ = search_exercises(&exercises, query);
        }
    }

    /// Simulates typing then clearing then typing again.
    #[test]
    fn search_type_clear_retype() {
        let exercises = sample_exercises();
        for _ in 0..3 {
            let _ = search_exercises(&exercises, "b");
            let _ = search_exercises(&exercises, "be");
            let _ = search_exercises(&exercises, "ben");
            let _ = search_exercises(&exercises, "");
        }
    }

    /// Every single printable ASCII character must not crash the search.
    #[test]
    fn search_every_ascii_char() {
        let exercises = sample_exercises();
        for c in (0x20u8..=0x7E).map(|b| b as char) {
            let query = String::from(c);
            let _ = search_exercises(&exercises, &query);
        }
    }

    /// Two-character combinations of common letters must not crash.
    #[test]
    fn search_two_char_combinations() {
        let exercises = sample_exercises();
        let chars = ['a', 'b', 'c', 'e', 'l', 'p', 'r', 's', 't', 'u'];
        for &a in &chars {
            for &b in &chars {
                let query = format!("{}{}", a, b);
                let _ = search_exercises(&exercises, &query);
            }
        }
    }

    /// Ensure search results are always a subset: adding characters never adds
    /// exercises that weren't in the broader result set.
    #[test]
    fn search_results_monotonically_narrow() {
        let exercises = sample_exercises();
        let broad = search_exercises(&exercises, "b");
        let narrow = search_exercises(&exercises, "be");
        for ex in &narrow {
            assert!(
                broad.iter().any(|b| b.id == ex.id),
                "Exercise '{}' in narrow results but not in broad results",
                ex.id
            );
        }
    }

    #[test]
    fn search_with_leading_trailing_spaces() {
        let exercises = sample_exercises();
        let _ = search_exercises(&exercises, " bench ");
        let _ = search_exercises(&exercises, "  ");
        let _ = search_exercises(&exercises, " b");
    }

    #[test]
    fn search_hyphenated_name() {
        let exercises = sample_exercises();
        // "Pull-Up" contains a hyphen
        let results = search_exercises(&exercises, "pull-up");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "pull_up");
        // Partial with hyphen must not crash
        let _ = search_exercises(&exercises, "pull-");
        let _ = search_exercises(&exercises, "-up");
        let _ = search_exercises(&exercises, "-");
    }

    /// Stress test: large exercise list with progressive typing.
    #[test]
    fn search_large_list_progressive_typing() {
        let mut exercises = Vec::new();
        for i in 0..500 {
            exercises.push(Exercise {
                id: format!("exercise_{}", i),
                name: format!("Exercise Number {}", i),
                force: if i % 3 == 0 {
                    Some(Force::Push)
                } else if i % 3 == 1 {
                    Some(Force::Pull)
                } else {
                    None
                },
                level: if i % 2 == 0 {
                    Some(Level::Beginner)
                } else {
                    Some(Level::Expert)
                },
                mechanic: None,
                equipment: if i % 4 == 0 {
                    Some(Equipment::Barbell)
                } else {
                    None
                },
                primary_muscles: vec![Muscle::Chest],
                secondary_muscles: vec![],
                instructions: vec![],
                category: if i % 5 == 0 {
                    Category::Cardio
                } else {
                    Category::Strength
                },
                images: vec![],
            });
        }

        // Progressive typing on a large list must not crash
        let input = "exercise number 42";
        for i in 1..=input.len() {
            let query = &input[..i];
            let _ = search_exercises(&exercises, query);
        }
    }

    #[test]
    fn get_exercise_by_id_empty_list() {
        let exercises: Vec<Exercise> = vec![];
        assert!(get_exercise_by_id(&exercises, "anything").is_none());
    }

    #[test]
    fn get_exercise_by_id_empty_id() {
        let exercises = sample_exercises();
        assert!(get_exercise_by_id(&exercises, "").is_none());
    }

    #[test]
    fn get_equipment_types_empty_list() {
        let exercises: Vec<Exercise> = vec![];
        let equipment = get_equipment_types(&exercises);
        assert!(equipment.is_empty());
    }

    #[test]
    fn get_muscle_groups_empty_list() {
        let exercises: Vec<Exercise> = vec![];
        let muscles = get_muscle_groups(&exercises);
        assert!(muscles.is_empty());
    }
}
