use crate::components::exercise_form_fields::ExerciseFormFields;
use crate::models::{get_current_timestamp, Category, CustomExercise, Equipment, Force, Muscle};
use crate::services::storage;
use dioxus::prelude::*;

#[component]
pub fn AddCustomExercisePage() -> Element {
    let name_input = use_signal(String::new);
    let category_input = use_signal(|| Category::Strength);
    let force_input: Signal<Option<Force>> = use_signal(|| None);
    let equipment_input: Signal<Option<Equipment>> = use_signal(|| None);
    let muscle_input = use_signal(String::new);
    let muscles_list = use_signal(Vec::<Muscle>::new);
    let secondary_muscle_input = use_signal(String::new);
    let secondary_muscles_list = use_signal(Vec::<Muscle>::new);
    let instructions_input = use_signal(String::new);
    let instructions_list = use_signal(Vec::<String>::new);

    let save_exercise = move |_: ()| {
        let name = name_input.read().trim().to_string();
        if name.is_empty() {
            return;
        }

        let timestamp = get_current_timestamp();

        let exercise = CustomExercise {
            id: format!("custom_{}", timestamp),
            name,
            category: *category_input.read(),
            force: *force_input.read(),
            equipment: *equipment_input.read(),
            primary_muscles: muscles_list.read().clone(),
            secondary_muscles: secondary_muscles_list.read().clone(),
            instructions: instructions_list.read().clone(),
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
                save_label: "Save Exercise".to_string(),
                on_save: save_exercise,
            }
        }
    }
}
