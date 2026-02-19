use crate::components::custom_exercise_form::{CustomExerciseForm, ExerciseFormData};
use crate::models::CustomExercise;
use crate::services::storage;
use dioxus::prelude::*;

#[component]
pub fn EditCustomExercisePage(id: String) -> Element {
    let custom_exercises = storage::use_custom_exercises();

    // Load the exercise to edit
    let exercise = use_memo(move || custom_exercises.read().iter().find(|e| e.id == id).cloned());

    let ex = match exercise() {
        Some(e) => e,
        None => {
            return rsx! {
                div { class: "container container--form",
                    p { "Exercise not found." }
                    button {
                        onclick: move |_| navigator().go_back(),
                        class: "back-btn",
                        "← Back"
                    }
                }
            };
        }
    };

    let exercise_id = ex.id.clone();

    let save_exercise = move |(
        name,
        category,
        force,
        equipment,
        primary_muscles,
        secondary_muscles,
        instructions,
    ): ExerciseFormData| {
        let updated = CustomExercise {
            id: exercise_id.clone(),
            name,
            category,
            force,
            equipment,
            primary_muscles,
            secondary_muscles,
            instructions,
        };
        storage::update_custom_exercise(updated);
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
                h1 { class: "page-title", "Edit Custom Exercise" }
            }

            CustomExerciseForm {
                title: "Edit Custom Exercise".to_string(),
                save_label: "💾 Save Changes".to_string(),
                initial_name: ex.name.clone(),
                initial_category: ex.category,
                initial_force: ex.force,
                initial_equipment: ex.equipment,
                initial_primary_muscles: ex.primary_muscles.clone(),
                initial_secondary_muscles: ex.secondary_muscles.clone(),
                initial_instructions: ex.instructions.clone(),
                on_save: save_exercise,
            }
        }
    }
}
