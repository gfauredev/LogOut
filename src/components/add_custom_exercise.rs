use dioxus::prelude::*;
use crate::models::{CustomExercise, Category, Force, Equipment, Muscle, get_current_timestamp};
use crate::services::storage;

#[component]
pub fn AddCustomExercisePage() -> Element {
    let mut name_input = use_signal(|| String::new());
    let mut category_input = use_signal(|| Category::Strength);
    let mut force_input: Signal<Option<Force>> = use_signal(|| None);
    let mut equipment_input: Signal<Option<Equipment>> = use_signal(|| None);
    let mut muscle_input = use_signal(|| String::new());
    let mut muscles_list = use_signal(|| Vec::<Muscle>::new());
    
    let add_muscle = move |_| {
        let value = muscle_input.read().trim().to_string();
        if !value.is_empty() {
            if let Ok(muscle) = serde_json::from_str::<Muscle>(&format!("\"{}\"", value)) {
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
    
    let save_exercise = move |_| {
        let name = name_input.read().trim().to_string();
        if name.is_empty() { return; }
        
        let timestamp = get_current_timestamp();
        
        let exercise = CustomExercise {
            id: format!("custom_{}", timestamp),
            name,
            category: *category_input.read(),
            force: *force_input.read(),
            equipment: *equipment_input.read(),
            primary_muscles: muscles_list.read().clone(),
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
                            if let Ok(cat) = serde_json::from_str::<Category>(&format!("\"{}\"", evt.value())) {
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
                            } else if let Ok(f) = serde_json::from_str::<Force>(&format!("\"{}\"", val)) {
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
                            } else if let Ok(e) = serde_json::from_str::<Equipment>(&format!("\"{}\"", val)) {
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
                
                // Save button
                button {
                    onclick: save_exercise,
                    disabled: name_input.read().trim().is_empty(),
                    class: "btn btn--primary",
                    "💾 Save Exercise"
                }
            }
        }
    }
}
