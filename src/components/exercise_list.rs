use dioxus::prelude::*;
use crate::services::exercise_db;
use crate::Route;

#[component]
pub fn ExerciseListPage() -> Element {
    let mut search_query = use_signal(|| String::new());
    let exercises = use_memo(move || {
        let query = search_query.read();
        if query.is_empty() {
            exercise_db::get_exercises().iter().take(50).cloned().collect::<Vec<_>>()
        } else {
            exercise_db::search_exercises(&query)
                .into_iter()
                .take(50)
                .cloned()
                .collect::<Vec<_>>()
        }
    });

    rsx! {
        div {
            class: "container",
            style: "padding: 20px; font-family: system-ui, -apple-system, sans-serif; max-width: 800px; margin: 0 auto;",
            
            header {
                style: "margin-bottom: 20px;",
                Link {
                    to: Route::HomePage {},
                    style: "text-decoration: none; color: #667eea; font-size: 1.1em;",
                    "← Back"
                }
                h1 { 
                    style: "margin: 15px 0;",
                    "Exercise Database" 
                }
                p { 
                    style: "color: #666;",
                    "Browse {exercise_db::get_exercises().len()} exercises"
                }
            }
            
            div {
                style: "margin-bottom: 20px;",
                input {
                    r#type: "text",
                    placeholder: "Search exercises, muscles, or categories...",
                    value: "{search_query}",
                    oninput: move |evt| search_query.set(evt.value()),
                    style: "
                        width: 100%;
                        padding: 12px;
                        font-size: 16px;
                        border: 2px solid #e0e0e0;
                        border-radius: 8px;
                        box-sizing: border-box;
                    ",
                }
            }
            
            div {
                style: "display: flex; flex-direction: column; gap: 10px;",
                for exercise in exercises() {
                    div {
                        key: "{exercise.id}",
                        style: "
                            padding: 15px;
                            border: 1px solid #e0e0e0;
                            border-radius: 8px;
                            background: white;
                            box-shadow: 0 2px 4px rgba(0,0,0,0.05);
                        ",
                        
                        h3 { 
                            style: "margin: 0 0 8px 0; font-size: 1.2em;",
                            "{exercise.name}" 
                        }
                        
                        div {
                            style: "display: flex; gap: 10px; flex-wrap: wrap; margin-bottom: 8px;",
                            
                            span {
                                style: "
                                    padding: 4px 10px;
                                    background: #667eea;
                                    color: white;
                                    border-radius: 12px;
                                    font-size: 0.85em;
                                ",
                                "{exercise.category}"
                            }
                            
                            span {
                                style: "
                                    padding: 4px 10px;
                                    background: #f5576c;
                                    color: white;
                                    border-radius: 12px;
                                    font-size: 0.85em;
                                ",
                                "{exercise.level}"
                            }
                            
                            if let Some(equipment) = &exercise.equipment {
                                span {
                                    style: "
                                        padding: 4px 10px;
                                        background: #4facfe;
                                        color: white;
                                        border-radius: 12px;
                                        font-size: 0.85em;
                                    ",
                                    "{equipment}"
                                }
                            }
                        }
                        
                        div {
                            style: "color: #666; font-size: 0.9em;",
                            "Target: {exercise.primary_muscles.join(\", \")}"
                        }
                    }
                }
            }
        }
    }
}
