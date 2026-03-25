use super::enums::{Category, Equipment, Force, Level, Mechanic, Muscle};
use super::exercise_type_tag;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
/// Sub-path for exercise images within the exercise database repository.
pub const EXERCISES_IMAGE_SUB_PATH: &str = "exercises/";
/// Per-language overrides for an exercise's display text, as defined in schema2.
///
/// Stored as a map from language code (e.g. `"fr"`) to this struct inside the
/// [`Exercise::i18n`] field.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExerciseI18n {
    /// Translated exercise name; falls back to [`Exercise::name`] when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Translated step-by-step instructions; falls back to [`Exercise::instructions`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<Vec<String>>,
}
/// Translations for enum display values for a single language, as loaded from
/// `i18n.json` in the exercise database release assets.
///
/// Each field maps English enum values (lowercased) to their translation.
/// For example, `category["strength"]` → `"musculation"` in French.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct DbI18nLang {
    #[serde(default)]
    pub force: HashMap<String, String>,
    #[serde(default)]
    pub level: HashMap<String, String>,
    #[serde(default)]
    pub mechanic: HashMap<String, String>,
    #[serde(default)]
    pub equipment: HashMap<String, String>,
    #[serde(default)]
    pub category: HashMap<String, String>,
    #[serde(default)]
    pub muscles: HashMap<String, String>,
}
/// Full enum-translation map keyed by BCP-47 language tag (e.g. `"fr"`, `"es"`).
///
/// Loaded once from `i18n.json` and stored in the Dioxus context as
/// [`DbI18nSignal`].
pub type DbI18n = HashMap<String, DbI18nLang>;
/// One entry from a per-language exercise translation file (e.g.
/// `exercises.fr.json`).  Only `id` is required; `name` and `instructions` are
/// optional so partial translations are handled gracefully.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExerciseLangEntry {
    /// Must match [`Exercise::id`] to locate the exercise to update.
    pub id: String,
    /// Translated exercise name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Translated step-by-step instructions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<Vec<String>>,
}
/// An exercise definition from the exercise database or created by the user.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Exercise {
    /// Unique identifier (slug from the exercise database, or `custom_<timestamp>` for user-created).
    pub id: String,
    /// Human-readable exercise name.
    pub name: String,
    /// Pre-computed lowercase name for efficient search filtering; not serialised.
    #[serde(skip)]
    pub name_lower: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Direction of muscular force (push / pull / static).
    pub force: Option<Force>,
    #[serde(default)]
    /// Difficulty level.
    pub level: Option<Level>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Whether the exercise is compound or isolation.
    pub mechanic: Option<Mechanic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Equipment required.
    pub equipment: Option<Equipment>,
    #[serde(rename = "primaryMuscles")]
    /// Primary muscle groups targeted.
    pub primary_muscles: Vec<Muscle>,
    #[serde(rename = "secondaryMuscles")]
    #[serde(default)]
    /// Secondary / synergist muscle groups.
    pub secondary_muscles: Vec<Muscle>,
    #[serde(default)]
    /// Step-by-step instructions for the exercise.
    pub instructions: Vec<String>,
    /// Exercise category (e.g. strength, cardio).
    pub category: Category,
    #[serde(default)]
    /// Relative or absolute image paths / URLs.
    pub images: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Per-language translations of [`name`] and [`instructions`] (schema2 `i18n` field).
    pub i18n: Option<HashMap<String, ExerciseI18n>>,
}
impl Exercise {
    /// Populate `name_lower` from `name`.
    /// Call this after deserialisation or after creating a new exercise to enable
    /// allocation-free search matching.
    pub fn with_lowercase(mut self) -> Self {
        self.name_lower = self.name.to_lowercase();
        self
    }
    /// Return the exercise name for the given BCP-47 language tag, falling back
    /// to the default English name.  Checks the `i18n` map for an exact match,
    /// then for a prefix match (e.g. `"fr"` from `"fr-FR"`).
    pub fn name_for_lang<'a>(&'a self, lang: &str) -> &'a str {
        if let Some(map) = &self.i18n {
            if let Some(t) = map.get(lang).and_then(|t| t.name.as_deref()) {
                return t;
            }
            if let Some(base) = lang.split('-').next() {
                if base != lang {
                    if let Some(t) = map.get(base).and_then(|t| t.name.as_deref()) {
                        return t;
                    }
                }
            }
        }
        &self.name
    }
    /// Return the exercise instructions for the given BCP-47 language tag,
    /// falling back to the default instructions.  Same prefix-matching logic as
    /// [`name_for_lang`].
    pub fn instructions_for_lang<'a>(&'a self, lang: &str) -> &'a [String] {
        if let Some(map) = &self.i18n {
            if let Some(t) = map.get(lang).and_then(|t| t.instructions.as_deref()) {
                return t;
            }
            if let Some(base) = lang.split('-').next() {
                if base != lang {
                    if let Some(t) = map.get(base).and_then(|t| t.instructions.as_deref()) {
                        return t;
                    }
                }
            }
        }
        &self.instructions
    }
    /// Get the URL for a specific image by index.
    ///
    /// Images with a known URL scheme (`http://`, `https://`, `blob:`, `data:`,
    /// `file://`) or an absolute filesystem path (starting with `/`) are returned
    /// unchanged.
    ///
    /// Images with the `local:` prefix are user-uploaded files copied into the
    /// app's `data_dir/images/` folder on native platforms.  The prefix is
    /// stripped and the filename is resolved to the full path.
    ///
    /// Images with the `idb:` prefix are stored as binary blobs in `IndexedDB` on
    /// the web platform.  This method returns `None` for them; use
    /// `storage::idb_images::get_image_blob_url` to obtain a `blob:` URL
    /// asynchronously when the image is actually rendered.
    ///
    /// Relative paths from the exercise database (e.g. `Squat/0.jpg`) are
    /// prefixed with the configured `EXERCISES_IMAGE_BASE_URL`.
    #[allow(dead_code)]
    pub fn get_image_url(&self, index: usize) -> Option<String> {
        let img = self.images.get(index)?;
        if img.starts_with("http://")
            || img.starts_with("https://")
            || img.starts_with("blob:")
            || img.starts_with("data:")
            || img.starts_with("file://")
            || img.starts_with('/')
        {
            return Some(img.clone());
        }
        if img.starts_with("idb:") {
            // Blob is stored in IndexedDB; must be loaded asynchronously.
            return None;
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(filename) = img.strip_prefix("local:") {
            let path = crate::services::storage::native_storage::data_dir()
                .join("images")
                .join(filename);
            return Some(format!("file://{}", path.display()));
        }
        let base_url = crate::utils::get_exercise_images_base_url();
        Some(format!("{base_url}{EXERCISES_IMAGE_SUB_PATH}{img}"))
    }
    /// Get the first image URL if available
    #[cfg(test)]
    pub fn get_first_image_url(&self) -> Option<String> {
        self.get_image_url(0)
    }
    /// Returns the CSS class and icon for the exercise type tag.
    ///
    /// The tag reflects what metrics are logged for this exercise:
    /// - `"tag-cardio"` / `"🏃"` — distance-based (`Category::Cardio`)
    /// - `"tag-strength"` / `"💪"` — repetition-based (`Force::Pull` / `Force::Push`)
    /// - `"tag-static"` / `"⏱️"` — time-only (static hold, stretch, etc.)
    #[allow(dead_code)]
    pub fn type_tag(&self) -> (&'static str, &'static str) {
        exercise_type_tag(self.category, self.force)
    }
}
impl AsRef<Exercise> for Exercise {
    fn as_ref(&self) -> &Exercise {
        self
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exercise_get_first_image_url_some() {
        #[cfg(not(target_arch = "wasm32"))]
        let _g = crate::services::storage::native_storage::test_lock();
        let ex = Exercise {
            id: "ex1".into(),
            name: "Squat".into(),
            name_lower: String::new(),
            force: None,
            level: Some(Level::Beginner),
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec![],
            category: Category::Strength,
            images: vec!["Squat/0.jpg".into()],
            i18n: None,
        };
        assert_eq!(
            ex.get_first_image_url(),
            Some(
                "https://raw.githubusercontent.com/gfauredev/free-exercise-db/main/exercises/Squat/0.jpg"
                    .into(),
            ),
        );
    }
    #[test]
    fn exercise_get_first_image_url_none() {
        let ex = Exercise {
            id: "ex1".into(),
            name: "Squat".into(),
            name_lower: String::new(),
            force: None,
            level: Some(Level::Beginner),
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec![],
            category: Category::Strength,
            images: vec![],
            i18n: None,
        };
        assert_eq!(ex.get_first_image_url(), None);
    }
    fn make_exercise_with_image(image: &str) -> Exercise {
        Exercise {
            id: "ex1".into(),
            name: "Test".into(),
            name_lower: String::new(),
            force: None,
            level: None,
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec![],
            category: Category::Strength,
            images: vec![image.into()],
            i18n: None,
        }
    }
    #[test]
    fn get_image_url_passes_through_blob_url() {
        let ex = make_exercise_with_image("blob:https://example.com/abc-123");
        assert_eq!(
            ex.get_first_image_url(),
            Some("blob:https://example.com/abc-123".into()),
        );
    }
    #[test]
    fn get_image_url_passes_through_data_url() {
        let ex = make_exercise_with_image("data:image/jpeg;base64,/9j/4AAQ");
        assert_eq!(
            ex.get_first_image_url(),
            Some("data:image/jpeg;base64,/9j/4AAQ".into()),
        );
    }
    #[test]
    fn get_image_url_passes_through_file_url() {
        let ex = make_exercise_with_image("file:///home/user/images/my.jpg");
        assert_eq!(
            ex.get_first_image_url(),
            Some("file:///home/user/images/my.jpg".into()),
        );
    }
    #[test]
    fn get_image_url_passes_through_absolute_path() {
        let ex = make_exercise_with_image("/data/user/0/dev.log_out/images/my.jpg");
        assert_eq!(
            ex.get_first_image_url(),
            Some("/data/user/0/dev.log_out/images/my.jpg".into()),
        );
    }
    #[test]
    fn get_image_url_prefixes_relative_exercise_db_path() {
        #[cfg(not(target_arch = "wasm32"))]
        let _g = crate::services::storage::native_storage::test_lock();
        let ex = make_exercise_with_image("Squat/0.jpg");
        let url = ex.get_first_image_url().unwrap();
        assert!(
            url.contains("exercises/Squat/0.jpg"),
            "relative path must be prefixed; got: {url}",
        );
    }
    #[test]
    fn user_exercise_serialization_with_all_fields() {
        let exercise = Exercise {
            id: "custom_123".into(),
            name: "Test Exercise".into(),
            name_lower: String::new(),
            category: Category::Strength,
            force: Some(Force::Push),
            level: None,
            mechanic: None,
            equipment: Some(Equipment::Barbell),
            primary_muscles: vec![Muscle::Chest],
            secondary_muscles: vec![Muscle::Triceps, Muscle::Shoulders],
            instructions: vec!["Step 1".into(), "Step 2".into()],
            images: vec!["https://example.com/img.jpg".into()],
            i18n: None,
        };
        let json = serde_json::to_string(&exercise).unwrap();
        let deserialized: Exercise = serde_json::from_str(&json).unwrap();
        assert_eq!(exercise, deserialized);
        assert_eq!(deserialized.secondary_muscles.len(), 2);
        assert_eq!(deserialized.instructions.len(), 2);
        assert_eq!(deserialized.images.len(), 1);
        assert!(!json.contains("name_lower"), "name_lower should be skipped");
    }
    #[test]
    fn exercise_with_lowercase_sets_name_lower_and_is_excluded_from_json() {
        let exercise = Exercise {
            id: "ex1".into(),
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
            i18n: None,
        }
        .with_lowercase();
        assert_eq!(exercise.name_lower, "bench press");
        let json = serde_json::to_string(&exercise).unwrap();
        assert!(
            !json.contains("name_lower"),
            "name_lower should be skipped from JSON"
        );
        let deserialized: Exercise = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.name_lower, "",
            "name_lower should default to empty after deserialization",
        );
    }
    #[test]
    fn exercise_backward_compat_missing_optional_fields() {
        let json = r#"{"id":"custom_1","name":"Old Exercise","category":"strength","force":"push","equipment":"barbell","primaryMuscles":["chest"]}"#;
        let exercise: Exercise = serde_json::from_str(json).unwrap();
        assert_eq!(exercise.secondary_muscles, Vec::<Muscle>::new());
        assert_eq!(exercise.instructions, Vec::<String>::new());
        assert_eq!(exercise.images, Vec::<String>::new());
        assert_eq!(exercise.level, None);
        assert_eq!(exercise.i18n, None);
    }
    fn make_i18n_exercise() -> Exercise {
        let mut map = HashMap::new();
        map.insert(
            "fr".into(),
            ExerciseI18n {
                name: Some("Traction".into()),
                instructions: Some(vec!["Saisissez la barre.".into()]),
            },
        );
        Exercise {
            id: "pull_up".into(),
            name: "Pull-Up".into(),
            name_lower: "pull-up".into(),
            force: Some(Force::Pull),
            level: Some(Level::Beginner),
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec!["Grab the bar.".into()],
            category: Category::Strength,
            images: vec![],
            i18n: Some(map),
        }
    }
    #[test]
    fn name_for_lang_returns_translation() {
        let ex = make_i18n_exercise();
        assert_eq!(ex.name_for_lang("fr"), "Traction");
    }
    #[test]
    fn name_for_lang_prefix_match() {
        let ex = make_i18n_exercise();
        assert_eq!(ex.name_for_lang("fr-FR"), "Traction");
    }
    #[test]
    fn name_for_lang_fallback_to_default() {
        let ex = make_i18n_exercise();
        assert_eq!(ex.name_for_lang("de"), "Pull-Up");
        assert_eq!(ex.name_for_lang("en"), "Pull-Up");
    }
    #[test]
    fn name_for_lang_no_i18n_returns_name() {
        let ex = Exercise {
            id: "bench".into(),
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
            i18n: None,
        };
        assert_eq!(ex.name_for_lang("fr"), "Bench Press");
    }
    #[test]
    fn instructions_for_lang_returns_translation() {
        let ex = make_i18n_exercise();
        assert_eq!(
            ex.instructions_for_lang("fr"),
            &["Saisissez la barre.".to_string()]
        );
    }
    #[test]
    fn instructions_for_lang_fallback_to_default() {
        let ex = make_i18n_exercise();
        assert_eq!(
            ex.instructions_for_lang("en"),
            &["Grab the bar.".to_string()]
        );
    }
    #[test]
    fn exercise_i18n_round_trip() {
        let ex = make_i18n_exercise();
        let json = serde_json::to_string(&ex).unwrap();
        assert!(
            json.contains("\"i18n\""),
            "i18n should be present when Some"
        );
        let back: Exercise = serde_json::from_str(&json).unwrap();
        assert_eq!(back.i18n, ex.i18n);
    }
    #[test]
    fn exercise_i18n_absent_from_json_when_none() {
        let ex = Exercise {
            id: "bench".into(),
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
            i18n: None,
        };
        let json = serde_json::to_string(&ex).unwrap();
        assert!(
            !json.contains("\"i18n\""),
            "i18n should be absent when None"
        );
    }
    #[test]
    fn db_i18n_lang_round_trip() {
        use crate::models::DbI18nLang;
        let json = r#"{
            "force": {"push": "poussée", "pull": "traction"},
            "level": {"beginner": "débutant"},
            "mechanic": {},
            "equipment": {"barbell": "barre"},
            "category": {"strength": "musculation"},
            "muscles": {"chest": "pectoraux"}
        }"#;
        let lang: DbI18nLang = serde_json::from_str(json).unwrap();
        assert_eq!(lang.force.get("push").map(String::as_str), Some("poussée"));
        assert_eq!(
            lang.level.get("beginner").map(String::as_str),
            Some("débutant")
        );
        assert_eq!(
            lang.category.get("strength").map(String::as_str),
            Some("musculation"),
        );
        assert_eq!(
            lang.muscles.get("chest").map(String::as_str),
            Some("pectoraux")
        );
    }
    #[test]
    fn db_i18n_lang_defaults_to_empty_maps() {
        use crate::models::DbI18nLang;
        let lang: DbI18nLang = serde_json::from_str("{}").unwrap();
        assert!(lang.force.is_empty());
        assert!(lang.level.is_empty());
        assert!(lang.mechanic.is_empty());
        assert!(lang.equipment.is_empty());
        assert!(lang.category.is_empty());
        assert!(lang.muscles.is_empty());
    }
    #[test]
    fn exercise_lang_entry_deserialize_partial() {
        use crate::models::ExerciseLangEntry;
        let json = r#"{"id": "bench_press", "name": "Développé Couché"}"#;
        let entry: ExerciseLangEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.id, "bench_press");
        assert_eq!(entry.name.as_deref(), Some("Développé Couché"));
        assert!(
            entry.instructions.is_none(),
            "instructions should be None when absent"
        );
    }
    #[test]
    fn exercise_lang_entry_deserialize_full() {
        use crate::models::ExerciseLangEntry;
        let json = r#"{"id":"squat","name":"Squat","instructions":["Étape 1","Étape 2"]}"#;
        let entry: ExerciseLangEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.id, "squat");
        assert_eq!(
            entry.instructions.as_deref(),
            Some(&["Étape 1".to_owned(), "Étape 2".to_owned()][..]),
        );
    }
    #[test]
    fn exercise_get_image_url_by_index() {
        #[cfg(not(target_arch = "wasm32"))]
        let _g = crate::services::storage::native_storage::test_lock();
        let ex = Exercise {
            id: "ex1".into(),
            name: "Squat".into(),
            name_lower: String::new(),
            force: None,
            level: Some(Level::Beginner),
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec![],
            category: Category::Strength,
            images: vec!["Squat/0.jpg".into(), "Squat/1.jpg".into()],
            i18n: None,
        };
        assert_eq!(
            ex.get_image_url(0),
            Some(
                "https://raw.githubusercontent.com/gfauredev/free-exercise-db/main/exercises/Squat/0.jpg"
                    .into(),
            ),
        );
        assert_eq!(
            ex.get_image_url(1),
            Some(
                "https://raw.githubusercontent.com/gfauredev/free-exercise-db/main/exercises/Squat/1.jpg"
                    .into(),
            ),
        );
        assert_eq!(ex.get_image_url(2), None);
    }
    #[test]
    fn exercise_get_image_url_full_url_passthrough() {
        let ex = Exercise {
            id: "ex1".into(),
            name: "Custom".into(),
            name_lower: String::new(),
            force: None,
            level: None,
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec![],
            category: Category::Strength,
            images: vec!["https://example.com/image.jpg".into()],
            i18n: None,
        };
        assert_eq!(
            ex.get_image_url(0),
            Some("https://example.com/image.jpg".into())
        );
    }
    #[test]
    fn exercise_level_none_when_missing_from_json() {
        let json = r#"{"id":"ex1","name":"Test","category":"strength","primaryMuscles":[]}"#;
        let ex: Exercise = serde_json::from_str(json).unwrap();
        assert_eq!(ex.level, None);
    }
    #[test]
    fn exercise_level_some_when_present_in_json() {
        let json = r#"{"id":"ex1","name":"Test","level":"expert","category":"strength","primaryMuscles":[]}"#;
        let ex: Exercise = serde_json::from_str(json).unwrap();
        assert_eq!(ex.level, Some(Level::Expert));
    }
    #[test]
    fn exercise_full_json_deserialization() {
        let json = r#"{
            "id": "bench_press",
            "name": "Bench Press",
            "force": "push",
            "level": "intermediate",
            "mechanic": "compound",
            "equipment": "barbell",
            "primaryMuscles": ["chest"],
            "secondaryMuscles": ["triceps", "shoulders"],
            "instructions": ["Lie down", "Push up"],
            "category": "strength",
            "images": ["BenchPress/0.jpg"]
        }"#;
        let ex: Exercise = serde_json::from_str(json).unwrap();
        assert_eq!(ex.id, "bench_press");
        assert_eq!(ex.name, "Bench Press");
        assert_eq!(ex.force, Some(Force::Push));
        assert_eq!(ex.level, Some(Level::Intermediate));
        assert_eq!(ex.mechanic, Some(Mechanic::Compound));
        assert_eq!(ex.equipment, Some(Equipment::Barbell));
        assert_eq!(ex.primary_muscles, vec![Muscle::Chest]);
        assert_eq!(
            ex.secondary_muscles,
            vec![Muscle::Triceps, Muscle::Shoulders]
        );
        assert_eq!(ex.instructions.len(), 2);
        assert_eq!(ex.category, Category::Strength);
        assert_eq!(ex.images, vec!["BenchPress/0.jpg"]);
    }
    #[test]
    fn exercise_optional_fields_none() {
        let json = r#"{
            "id": "stretch1",
            "name": "Hamstring Stretch",
            "level": "beginner",
            "primaryMuscles": ["hamstrings"],
            "secondaryMuscles": [],
            "instructions": [],
            "category": "stretching",
            "images": []
        }"#;
        let ex: Exercise = serde_json::from_str(json).unwrap();
        assert_eq!(ex.force, None);
        assert_eq!(ex.mechanic, None);
        assert_eq!(ex.equipment, None);
        assert!(ex.images.is_empty());
    }
    #[test]
    fn exercise_minimal() {
        let ex = Exercise {
            id: "custom_1".into(),
            name: "My Exercise".into(),
            name_lower: String::new(),
            category: Category::Strength,
            force: None,
            level: None,
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec![],
            images: vec![],
            i18n: None,
        };
        let json = serde_json::to_string(&ex).unwrap();
        let back: Exercise = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ex);
    }
    #[test]
    fn exercise_get_image_url_http_passthrough() {
        let ex = Exercise {
            id: "ex1".into(),
            name: "Custom".into(),
            name_lower: String::new(),
            force: None,
            level: None,
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec![],
            category: Category::Strength,
            images: vec!["http://example.com/image.jpg".into()],
            i18n: None,
        };
        assert_eq!(
            ex.get_image_url(0),
            Some("http://example.com/image.jpg".into())
        );
    }
    #[test]
    fn exercise_type_tag_cardio() {
        let ex = Exercise {
            id: "run1".into(),
            name: "Running".into(),
            name_lower: String::new(),
            category: Category::Cardio,
            force: None,
            level: None,
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec![],
            images: vec![],
            i18n: None,
        };
        assert_eq!(ex.type_tag(), ("tag-cardio", "🏃"));
    }
    #[test]
    fn exercise_type_tag_strength() {
        let ex = Exercise {
            id: "bench1".into(),
            name: "Bench Press".into(),
            name_lower: String::new(),
            category: Category::Strength,
            force: Some(Force::Push),
            level: None,
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec![],
            images: vec![],
            i18n: None,
        };
        assert_eq!(ex.type_tag(), ("tag-strength", "💪"));
    }
    #[test]
    fn exercise_type_tag_static() {
        let ex = Exercise {
            id: "plank1".into(),
            name: "Plank".into(),
            name_lower: String::new(),
            category: Category::Strength,
            force: Some(Force::Static),
            level: None,
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec![],
            images: vec![],
            i18n: None,
        };
        assert_eq!(ex.type_tag(), ("tag-static", "⏱️"));
    }
}
