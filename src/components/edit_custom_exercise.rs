use crate::components::exercise_form_fields::ExerciseFormFields;
use crate::models::{Equipment, Exercise, Force};
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
                main { class: "container--form",
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

    let name_input = use_signal(|| ex.name.clone());
    let category_input = use_signal(|| ex.category);
    let force_input: Signal<Option<Force>> = use_signal(|| ex.force);
    let equipment_input: Signal<Option<Equipment>> = use_signal(|| ex.equipment);
    let muscle_input = use_signal(String::new);
    let muscles_list = use_signal(|| ex.primary_muscles.clone());
    let secondary_muscle_input = use_signal(String::new);
    let secondary_muscles_list = use_signal(|| ex.secondary_muscles.clone());
    let instructions_input = use_signal(String::new);
    let instructions_list = use_signal(|| ex.instructions.clone());
    let image_url_input = use_signal(String::new);
    let images_list = use_signal(|| ex.images.clone());

    let exercise_id = ex.id.clone();
    let exercise_level = ex.level;
    let exercise_mechanic = ex.mechanic;

    let save_exercise = move |_: ()| {
        let name = name_input.read().trim().to_string();
        if name.is_empty() {
            return;
        }

        let updated = Exercise {
            id: exercise_id.clone(),
            name,
            category: *category_input.read(),
            force: *force_input.read(),
            level: exercise_level,
            mechanic: exercise_mechanic,
            equipment: *equipment_input.read(),
            primary_muscles: muscles_list.read().clone(),
            secondary_muscles: secondary_muscles_list.read().clone(),
            instructions: instructions_list.read().clone(),
            images: images_list.read().clone(),
        };

        storage::update_custom_exercise(updated);
        navigator().go_back();
    };

    rsx! {
        header {
            button {
                onclick: move |_| navigator().go_back(),
                class: "back-btn",
                "← Back"
            }
            h1 { "Edit Exercise" }
        }
        main { class: "container--form",
            ExerciseFormFields {
                name_input,
                category_input,
                force_input,
                equipment_input,
                muscle_input,
                muscles_list,
                secondary_muscle_input,
                secondary_muscles_list,
                instructions_input,
                instructions_list,
                image_url_input,
                images_list,
                save_label: "Save Changes".to_string(),
                on_save: save_exercise,
            }
        }
    }
}
