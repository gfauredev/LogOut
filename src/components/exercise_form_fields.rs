use crate::models::{Category, Equipment, Force, Muscle};
use dioxus::prelude::*;

/// Shared form fields used by both AddCustomExercisePage and EditCustomExercisePage.
#[component]
pub fn ExerciseFormFields(
    name_input: Signal<String>,
    category_input: Signal<Category>,
    force_input: Signal<Option<Force>>,
    equipment_input: Signal<Option<Equipment>>,
    muscle_input: Signal<String>,
    muscles_list: Signal<Vec<Muscle>>,
    secondary_muscle_input: Signal<String>,
    secondary_muscles_list: Signal<Vec<Muscle>>,
    instructions_input: Signal<String>,
    instructions_list: Signal<Vec<String>>,
    image_url_input: Signal<String>,
    images_list: Signal<Vec<String>>,
    save_label: String,
    on_save: EventHandler<()>,
) -> Element {
    let mut name_input = name_input;
    let mut category_input = category_input;
    let mut force_input = force_input;
    let mut equipment_input = equipment_input;
    let mut muscle_input = muscle_input;
    let mut muscles_list = muscles_list;
    let mut secondary_muscle_input = secondary_muscle_input;
    let mut secondary_muscles_list = secondary_muscles_list;
    let mut instructions_input = instructions_input;
    let mut instructions_list = instructions_list;
    let mut image_url_input = image_url_input;
    let mut images_list = images_list;

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

    let add_image = move |_| {
        let url = image_url_input.read().trim().to_string();
        if !url.is_empty() {
            let mut imgs = images_list.read().clone();
            if !imgs.contains(&url) {
                imgs.push(url);
                images_list.set(imgs);
                image_url_input.set(String::new());
            }
        }
    };

    let mut remove_image = move |idx: usize| {
        let mut imgs = images_list.read().clone();
        if idx < imgs.len() {
            imgs.remove(idx);
            images_list.set(imgs);
        }
    };

    rsx! {
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
                        class: "form-input form-input--flex",
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

            // Images
            div {
                label { class: "form-label", "Images (URLs)" }

                div {
                    class: "muscle-row",
                    input {
                        r#type: "url",
                        placeholder: "https://example.com/image.jpg",
                        value: "{image_url_input}",
                        oninput: move |evt| image_url_input.set(evt.value()),
                        class: "form-input form-input--flex",
                    }
                    button {
                        onclick: add_image,
                        class: "btn btn--accent-lg",
                        "Add"
                    }
                }

                if !images_list.read().is_empty() {
                    div {
                        class: "muscle-tags",
                        for (idx, url) in images_list.read().iter().enumerate() {
                            div {
                                key: "{idx}",
                                class: "muscle-tag",
                                span { class: "image-url-tag", "{url}" }
                                button {
                                    onclick: move |_| remove_image(idx),
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
                onclick: move |_| on_save.call(()),
                disabled: name_input.read().trim().is_empty(),
                class: "btn btn--primary",
                "💾 {save_label}"
            }
        }
    }
}
