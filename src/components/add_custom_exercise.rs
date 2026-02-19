use crate::components::custom_exercise_form::{CustomExerciseForm, ExerciseFormData};
use crate::models::{get_current_timestamp, Category, CustomExercise};
use crate::services::storage;
use dioxus::prelude::*;

#[component]
pub fn AddCustomExercisePage() -> Element {
    let save_exercise = move |(
        name,
        category,
        force,
        equipment,
        primary_muscles,
        secondary_muscles,
        instructions,
    ): ExerciseFormData| {
        let timestamp = get_current_timestamp();
        let exercise = CustomExercise {
            id: format!("custom_{}", timestamp),
            name,
            category,
            force,
            equipment,
            primary_muscles,
            secondary_muscles,
            instructions,
        };
        storage::add_custom_exercise(exercise);
        navigator().go_back();
    };

    rsx! {
        div {
            class: "container container--form",

            header {
                class: "page-header",
                button {
                    onclick: move |_| navigator().go_back(),
                    class: "back-btn",
                    "← Back"
                }
                h1 { class: "page-title", "Add Custom Exercise" }
            }

            CustomExerciseForm {
                title: "Add Custom Exercise".to_string(),
                save_label: "💾 Save Exercise".to_string(),
                initial_name: String::new(),
                initial_category: Category::Strength,
                initial_force: None,
                initial_equipment: None,
                initial_primary_muscles: vec![],
                initial_secondary_muscles: vec![],
                initial_instructions: vec![],
                on_save: save_exercise,
            }
        }
    }
}
