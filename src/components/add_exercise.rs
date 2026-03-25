use crate::components::exercise_form_fields::ExerciseFormFields;
use crate::models::{get_current_timestamp, Category, Equipment, Exercise, Force, Muscle};
use crate::services::storage;
use dioxus::prelude::*;
#[component]
pub fn AddExercise() -> Element {
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
    let image_url_input = use_signal(String::new);
    let images_list = use_signal(Vec::<String>::new);
    let sessions = storage::use_sessions();
    let save_exercise = move |()| {
        let name = name_input.read().trim().to_string();
        if name.is_empty() {
            return;
        }
        let timestamp = get_current_timestamp();
        let name_lower = name.to_lowercase();
        let exercise = Exercise {
            id: format!("custom_{timestamp}"),
            name,
            name_lower,
            category: *category_input.read(),
            force: *force_input.read(),
            level: None,
            mechanic: None,
            equipment: *equipment_input.read(),
            primary_muscles: muscles_list.read().clone(),
            secondary_muscles: secondary_muscles_list.read().clone(),
            instructions: instructions_list.read().clone(),
            images: images_list.read().clone(),
            i18n: None,
        };
        let exercise_id = exercise.id.clone();
        storage::add_custom_exercise(exercise);
        let active = sessions.read().iter().find(|s| s.is_active()).cloned();
        if let Some(active_session) = active {
            let mut active_session = (*active_session).clone();
            let start = get_current_timestamp();
            active_session.current_exercise_id = Some(exercise_id);
            active_session.current_exercise_start = Some(start);
            active_session.rest_start_time = None;
            storage::save_session(active_session);
            navigator().push(crate::Route::Home {});
        } else {
            navigator().go_back();
        }
    };
    rsx! {
        header {
            h1 { "Add Exercise" }
            button {
                class: "back",
                onclick: move |_| navigator().go_back(),
                title: "Cancel",
                "❌"
            }
        }
        main { class: "edit",
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
                save_label: "Save Exercise".to_string(),
                on_save: save_exercise,
            }
        }
    }
}
