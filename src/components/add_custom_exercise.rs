use dioxus::prelude::*;
use crate::models::CustomExercise;
use crate::services::{exercise_db, storage};
use std::time::{SystemTime, UNIX_EPOCH};

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
    
    // Get equipment types from database
    let equipment_types = exercise_db::get_equipment_types();
    
    // Get muscle groups from database
    let muscle_groups = exercise_db::get_muscle_groups();
    
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
        if name.is_empty() {
            return;
        }
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
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
        
        // Navigate back
        navigator().go_back();
    };

    rsx! {
        div {
            class: "container",
            style: "padding: 20px; font-family: system-ui, -apple-system, sans-serif; max-width: 600px; margin: 0 auto;",
            
            header {
                style: "margin-bottom: 20px;",
                button {
                    onclick: move |_| navigator().go_back(),
                    style: "
                        background: none;
                        border: none;
                        color: #667eea;
                        font-size: 1.1em;
                        cursor: pointer;
                        padding: 0;
                    ",
                    "← Back"
                }
                h1 { 
                    style: "margin: 15px 0;",
                    "Add Custom Exercise" 
                }
            }
            
            div {
                style: "display: flex; flex-direction: column; gap: 20px;",
                
                // Name
                div {
                    label {
                        style: "display: block; margin-bottom: 5px; font-weight: bold;",
                        "Exercise Name *"
                    }
                    input {
                        r#type: "text",
                        placeholder: "e.g., Farmer's Walk",
                        value: "{name_input}",
                        oninput: move |evt| name_input.set(evt.value()),
                        style: "
                            width: 100%;
                            padding: 10px;
                            border: 1px solid #e0e0e0;
                            border-radius: 6px;
                            font-size: 16px;
                            box-sizing: border-box;
                        ",
                    }
                }
                
                // Category
                div {
                    label {
                        style: "display: block; margin-bottom: 5px; font-weight: bold;",
                        "Category *"
                    }
                    select {
                        value: "{category_input}",
                        oninput: move |evt| category_input.set(evt.value()),
                        style: "
                            width: 100%;
                            padding: 10px;
                            border: 1px solid #e0e0e0;
                            border-radius: 6px;
                            font-size: 16px;
                            box-sizing: border-box;
                        ",
                        for category in categories {
                            option {
                                value: "{category}",
                                "{category}"
                            }
                        }
                    }
                }
                
                // Force type
                div {
                    label {
                        style: "display: block; margin-bottom: 5px; font-weight: bold;",
                        "Force Type"
                    }
                    select {
                        value: "{force_input}",
                        oninput: move |evt| force_input.set(evt.value()),
                        style: "
                            width: 100%;
                            padding: 10px;
                            border: 1px solid #e0e0e0;
                            border-radius: 6px;
                            font-size: 16px;
                            box-sizing: border-box;
                        ",
                        for force_type in force_types {
                            option {
                                value: "{force_type}",
                                if force_type.is_empty() {
                                    "None"
                                } else {
                                    "{force_type}"
                                }
                            }
                        }
                    }
                }
                
                // Equipment
                div {
                    label {
                        style: "display: block; margin-bottom: 5px; font-weight: bold;",
                        "Equipment"
                    }
                    select {
                        value: "{equipment_input}",
                        oninput: move |evt| equipment_input.set(evt.value()),
                        style: "
                            width: 100%;
                            padding: 10px;
                            border: 1px solid #e0e0e0;
                            border-radius: 6px;
                            font-size: 16px;
                            box-sizing: border-box;
                        ",
                        option { value: "", "None" }
                        for equipment in equipment_types.iter() {
                            option {
                                value: "{equipment}",
                                "{equipment}"
                            }
                        }
                    }
                }
                
                // Primary muscles
                div {
                    label {
                        style: "display: block; margin-bottom: 5px; font-weight: bold;",
                        "Primary Muscles"
                    }
                    
                    div {
                        style: "display: flex; gap: 10px; margin-bottom: 10px;",
                        select {
                            value: "{muscle_input}",
                            oninput: move |evt| muscle_input.set(evt.value()),
                            style: "
                                flex: 1;
                                padding: 10px;
                                border: 1px solid #e0e0e0;
                                border-radius: 6px;
                                font-size: 16px;
                            ",
                            option { value: "", "Select muscle..." }
                            for muscle in muscle_groups.iter() {
                                option {
                                    value: "{muscle}",
                                    "{muscle}"
                                }
                            }
                        }
                        button {
                            onclick: add_muscle,
                            style: "
                                padding: 10px 20px;
                                background: #4facfe;
                                color: white;
                                border: none;
                                border-radius: 6px;
                                font-weight: bold;
                                cursor: pointer;
                            ",
                            "Add"
                        }
                    }
                    
                    if !muscles_list.read().is_empty() {
                        div {
                            style: "display: flex; flex-wrap: wrap; gap: 8px;",
                            for muscle in muscles_list.read().iter() {
                                div {
                                    key: "{muscle}",
                                    style: "
                                        padding: 6px 12px;
                                        background: #667eea;
                                        color: white;
                                        border-radius: 16px;
                                        display: flex;
                                        align-items: center;
                                        gap: 8px;
                                    ",
                                    span { "{muscle}" }
                                    button {
                                        onclick: {
                                            let m = muscle.clone();
                                            move |_| remove_muscle(m.clone())
                                        },
                                        style: "
                                            background: none;
                                            border: none;
                                            color: white;
                                            cursor: pointer;
                                            font-size: 1.2em;
                                            padding: 0;
                                            line-height: 1;
                                        ",
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
                    style: "
                        width: 100%;
                        padding: 15px;
                        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                        color: white;
                        border: none;
                        border-radius: 8px;
                        font-size: 1.2em;
                        font-weight: bold;
                        cursor: pointer;
                        margin-top: 10px;
                    ",
                    "💾 Save Exercise"
                }
            }
        }
    }
}
