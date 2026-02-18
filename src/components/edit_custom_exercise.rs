use dioxus::prelude::*;
use crate::models::{CustomExercise, Category, Force, Equipment, Muscle};
use crate::services::storage;

#[component]
pub fn EditCustomExercisePage(id: String) -> Element {
    let custom_exercises = storage::use_custom_exercises();

    // Load the exercise to edit
    let exercise = use_memo(move || {
        custom_exercises.read().iter().find(|e| e.id == id).cloned()
    });

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

    let mut name_input = use_signal(|| ex.name.clone());
    let mut category_input = use_signal(|| ex.category);
    let mut force_input: Signal<Option<Force>> = use_signal(|| ex.force);
    let mut equipment_input: Signal<Option<Equipment>> = use_signal(|| ex.equipment);
    let mut muscle_input = use_signal(|| String::new());
    let mut muscles_list = use_signal(|| ex.primary_muscles.clone());
    let mut secondary_muscle_input = use_signal(|| String::new());
    let mut secondary_muscles_list = use_signal(|| ex.secondary_muscles.clone());
    let mut instructions_input = use_signal(|| String::new());
    let mut instructions_list = use_signal(|| ex.instructions.clone());

    let exercise_id = ex.id.clone();

    let add_muscle = move |_| {
        let value = muscle_input.read().trim().to_string();
        if !value.is_empty() {
            if let Ok(muscle) = serde_json::from_value::<Muscle>(serde_json::Value::String(value)) {
                let mut muscles = muscles_list.read().clone();
                if !muscles.contains(&muscle) {
                    muscles.push(muscle);
                    muscles_list.set(muscles);
                    muscle_input.set(String::new());
                }
            }
        }
    };

    let mut remove_muscle = move |muscle: Muscle| {
        let mut muscles = muscles_list.read().clone();
        muscles.retain(|m| m != &muscle);
        muscles_list.set(muscles);
    };

    let add_secondary_muscle = move |_| {
        let value = secondary_muscle_input.read().trim().to_string();
        if !value.is_empty() {
            if let Ok(muscle) = serde_json::from_value::<Muscle>(serde_json::Value::String(value)) {
                let mut muscles = secondary_muscles_list.read().clone();
                if !muscles.contains(&muscle) {
                    muscles.push(muscle);
                    secondary_muscles_list.set(muscles);
                    secondary_muscle_input.set(String::new());
                }
            }
        }
    };

    let mut remove_secondary_muscle = move |muscle: Muscle| {
        let mut muscles = secondary_muscles_list.read().clone();
        muscles.retain(|m| m != &muscle);
        secondary_muscles_list.set(muscles);
    };

    let add_instruction = move |_| {
        let value = instructions_input.read().trim().to_string();
        if !value.is_empty() {
            let mut instructions = instructions_list.read().clone();
            instructions.push(value);
            instructions_list.set(instructions);
            instructions_input.set(String::new());
        }
    };

    let mut remove_instruction = move |idx: usize| {
        let mut instructions = instructions_list.read().clone();
        if idx < instructions.len() {
            instructions.remove(idx);
            instructions_list.set(instructions);
        }
    };

    let save_exercise = move |_| {
        let name = name_input.read().trim().to_string();
        if name.is_empty() { return; }

        let updated = CustomExercise {
            id: exercise_id.clone(),
            name,
            category: *category_input.read(),
            force: *force_input.read(),
            equipment: *equipment_input.read(),
            primary_muscles: muscles_list.read().clone(),
            secondary_muscles: secondary_muscles_list.read().clone(),
            instructions: instructions_list.read().clone(),
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

            div {
                class: "form-stack",

                // Name
                div {
                    label { class: "form-label", "Exercise Name *" }
                    input {
                        r#type: "text",
                        placeholder: "e.g., Farmer's Walk",
                        value: "{name_input}",
                        oninput: move |evt| name_input.set(evt.value()),
                        class: "form-input",
                    }
                }

                // Category
                div {
                    label { class: "form-label", "Category *" }
                    select {
                        value: "{category_input.read()}",
                        oninput: move |evt| {
                            if let Ok(cat) = serde_json::from_value::<Category>(serde_json::Value::String(evt.value())) {
                                category_input.set(cat);
                            }
                        },
                        class: "form-select",
                        for category in Category::ALL {
                            option { value: "{category}", "{category}" }
                        }
                    }
                }

                // Force type
                div {
                    label { class: "form-label", "Force Type" }
                    select {
                        value: if let Some(f) = *force_input.read() { f.to_string() } else { String::new() },
                        oninput: move |evt| {
                            let val = evt.value();
                            if val.is_empty() {
                                force_input.set(None);
                            } else if let Ok(f) = serde_json::from_value::<Force>(serde_json::Value::String(val)) {
                                force_input.set(Some(f));
                            }
                        },
                        class: "form-select",
                        option { value: "", "None" }
                        for force_type in Force::ALL {
                            option { value: "{force_type}", "{force_type}" }
                        }
                    }
                }

                // Equipment
                div {
                    label { class: "form-label", "Equipment" }
                    select {
                        value: if let Some(e) = *equipment_input.read() { e.to_string() } else { String::new() },
                        oninput: move |evt| {
                            let val = evt.value();
                            if val.is_empty() {
                                equipment_input.set(None);
                            } else if let Ok(e) = serde_json::from_value::<Equipment>(serde_json::Value::String(val)) {
                                equipment_input.set(Some(e));
                            }
                        },
                        class: "form-select",
                        option { value: "", "None" }
                        for equipment in Equipment::ALL {
                            option { value: "{equipment}", "{equipment}" }
                        }
                    }
                }

                // Primary muscles
                div {
                    label { class: "form-label", "Primary Muscles" }

                    div {
                        class: "muscle-row",
                        select {
                            value: "{muscle_input}",
                            oninput: move |evt| muscle_input.set(evt.value()),
                            class: "muscle-select",
                            option { value: "", "Select muscle..." }
                            for muscle in Muscle::ALL {
                                option { value: "{muscle}", "{muscle}" }
                            }
                        }
                        button {
                            onclick: add_muscle,
                            class: "btn btn--accent-lg",
                            "Add"
                        }
                    }

                    if !muscles_list.read().is_empty() {
                        div {
                            class: "muscle-tags",
                            for muscle in muscles_list.read().iter() {
                                div {
                                    key: "{muscle}",
                                    class: "muscle-tag",
                                    span { "{muscle}" }
                                    button {
                                        onclick: {
                                            let m = *muscle;
                                            move |_| remove_muscle(m)
                                        },
                                        class: "muscle-tag__remove",
                                        "×"
                                    }
                                }
                            }
                        }
                    }
                }

                // Secondary muscles
                div {
                    label { class: "form-label", "Secondary Muscles" }

                    div {
                        class: "muscle-row",
                        select {
                            value: "{secondary_muscle_input}",
                            oninput: move |evt| secondary_muscle_input.set(evt.value()),
                            class: "muscle-select",
                            option { value: "", "Select muscle..." }
                            for muscle in Muscle::ALL {
                                option { value: "{muscle}", "{muscle}" }
                            }
                        }
                        button {
                            onclick: add_secondary_muscle,
                            class: "btn btn--accent-lg",
                            "Add"
                        }
                    }

                    if !secondary_muscles_list.read().is_empty() {
                        div {
                            class: "muscle-tags",
                            for muscle in secondary_muscles_list.read().iter() {
                                div {
                                    key: "{muscle}",
                                    class: "muscle-tag muscle-tag--secondary",
                                    span { "{muscle}" }
                                    button {
                                        onclick: {
                                            let m = *muscle;
                                            move |_| remove_secondary_muscle(m)
                                        },
                                        class: "muscle-tag__remove",
                                        "×"
                                    }
                                }
                            }
                        }
                    }
                }

                // Instructions
                div {
                    label { class: "form-label", "Instructions" }

                    div {
                        class: "muscle-row",
                        input {
                            r#type: "text",
                            placeholder: "Add an instruction step...",
                            value: "{instructions_input}",
                            oninput: move |evt| instructions_input.set(evt.value()),
                            class: "form-input",
                            style: "flex: 1;",
                        }
                        button {
                            onclick: add_instruction,
                            class: "btn btn--accent-lg",
                            "Add"
                        }
                    }

                    if !instructions_list.read().is_empty() {
                        ol {
                            class: "instructions-list",
                            for (idx, instruction) in instructions_list.read().iter().enumerate() {
                                li {
                                    key: "{idx}",
                                    class: "instruction-item",
                                    span { "{instruction}" }
                                    button {
                                        onclick: move |_| remove_instruction(idx),
                                        class: "muscle-tag__remove",
                                        "×"
                                    }
                                }
                            }
                        }
                    }
                }

                // Save button
                button {
                    onclick: save_exercise,
                    disabled: name_input.read().trim().is_empty(),
                    class: "btn btn--primary",
                    "💾 Save Changes"
                }
            }
        }
    }
}
