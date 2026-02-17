use dioxus::prelude::*;
use crate::models::{CustomExercise, get_current_timestamp};
use crate::services::{exercise_db, storage};

#[component]
pub fn AddCustomExercisePage() -> Element {
    let mut name_input = use_signal(|| String::new());
    let mut category_input = use_signal(|| String::from("strength"));
    let mut force_input = use_signal(|| String::from(""));
    let mut equipment_input = use_signal(|| String::from(""));
    let mut muscle_input = use_signal(|| String::new());
    let mut muscles_list = use_signal(|| Vec::<String>::new());
    
    let categories = vec![
        "strength",
        "cardio",
        "stretching",
        "powerlifting",
        "strongman",
        "plyometrics",
        "olympic weightlifting",
    ];
    
    let force_types = vec!["", "pull", "push", "static"];
    let all_exercises = exercise_db::use_exercises();
    let all = all_exercises.read();
    let equipment_types = exercise_db::get_equipment_types(&all);
    let muscle_groups = exercise_db::get_muscle_groups(&all);
    
    let add_muscle = move |_| {
        let muscle = muscle_input.read().trim().to_string();
        if !muscle.is_empty() {
            let mut muscles = muscles_list.read().clone();
            if !muscles.contains(&muscle) {
                muscles.push(muscle);
                muscles_list.set(muscles);
                muscle_input.set(String::new());
            }
        }
    };
    
    let mut remove_muscle = move |muscle: String| {
        let mut muscles = muscles_list.read().clone();
        muscles.retain(|m| m != &muscle);
        muscles_list.set(muscles);
    };
    
    let save_exercise = move |_| {
        let name = name_input.read().trim().to_string();
        if name.is_empty() { return; }
        
        let timestamp = get_current_timestamp();
        let force = force_input.read().trim().to_string();
        let equipment = equipment_input.read().trim().to_string();
        
        let exercise = CustomExercise {
            id: format!("custom_{}", timestamp),
            name,
            category: category_input.read().clone(),
            force: if force.is_empty() { None } else { Some(force) },
            equipment: if equipment.is_empty() { None } else { Some(equipment) },
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
                        value: "{category_input}",
                        oninput: move |evt| category_input.set(evt.value()),
                        class: "form-select",
                        for category in categories {
                            option { value: "{category}", "{category}" }
                        }
                    }
                }
                
                // Force type
                div {
                    label { class: "form-label", "Force Type" }
                    select {
                        value: "{force_input}",
                        oninput: move |evt| force_input.set(evt.value()),
                        class: "form-select",
                        for force_type in force_types {
                            option {
                                value: "{force_type}",
                                if force_type.is_empty() { "None" } else { "{force_type}" }
                            }
                        }
                    }
                }
                
                // Equipment
                div {
                    label { class: "form-label", "Equipment" }
                    select {
                        value: "{equipment_input}",
                        oninput: move |evt| equipment_input.set(evt.value()),
                        class: "form-select",
                        option { value: "", "None" }
                        for equipment in equipment_types.iter() {
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
                            for muscle in muscle_groups.iter() {
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
                                            let m = muscle.clone();
                                            move |_| remove_muscle(m.clone())
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
