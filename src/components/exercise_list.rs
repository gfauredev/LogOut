use std::collections::HashMap;
use dioxus::prelude::*;
use crate::models::{Exercise, Level};
use crate::services::{exercise_db, storage};
use crate::components::{BottomNav, ActiveTab, ExerciseCard};
use crate::Route;

#[component]
pub fn ExerciseListPage() -> Element {
    let all_exercises = exercise_db::use_exercises();
    let custom_exercises = storage::use_custom_exercises();
    let mut search_query = use_signal(|| String::new());
    let mut instructions_open = use_signal(|| HashMap::<String, bool>::new());
    let image_indices = use_signal(|| HashMap::<String, usize>::new());

    // Merge DB exercises and custom exercises into a unified list
    let exercises = use_memo(move || {
        let query = search_query.read();
        let all = all_exercises.read();
        let custom = custom_exercises.read();

        let mut results: Vec<Exercise> = Vec::new();

        // Add custom exercises (converted to Exercise for uniform display)
        for ce in custom.iter() {
            let matches = query.is_empty()
                || ce.name.to_lowercase().contains(&query.to_lowercase());
            if matches {
                results.push(Exercise {
                    id: ce.id.clone(),
                    name: ce.name.clone(),
                    force: ce.force,
                    level: Level::Beginner,
                    mechanic: None,
                    equipment: ce.equipment,
                    primary_muscles: ce.primary_muscles.clone(),
                    secondary_muscles: ce.secondary_muscles.clone(),
                    instructions: ce.instructions.clone(),
                    category: ce.category,
                    images: vec![], // Custom exercises have no images
                });
            }
        }

        // Add DB exercises
        if query.is_empty() {
            results.extend(all.iter().take(50).cloned());
        } else {
            results.extend(
                exercise_db::search_exercises(&all, &query)
                    .into_iter()
                    .take(50),
            );
        }

        results
    });

    let total = all_exercises.read().len() + custom_exercises.read().len();

    rsx! {
        div { class: "page-container",
            main { class: "page-content container container--narrow",
            
                header {
                    class: "page-header",
                    h1 { class: "page-title", "Exercise Database" }
                    p { class: "page-subtitle",
                        "Browse {total} exercises"
                    }
                }
                
                div {
                    class: "search-wrapper",
                    input {
                        r#type: "text",
                        placeholder: "Search exercises, muscles, or categories...",
                        value: "{search_query}",
                        oninput: move |evt| search_query.set(evt.value()),
                        class: "search-input",
                    }
                }
                
                section {
                    class: "exercise-list",
                    for exercise in exercises() {
                        {
                            let is_custom = exercise.id.starts_with("custom_");
                            rsx! {
                                if is_custom {
                                    article {
                                        key: "{exercise.id}",
                                        class: "exercise-card",
                                        div {
                                            class: "exercise-card__custom-header",
                                            div {
                                                class: "exercise-card__custom-badge",
                                                "Custom"
                                            }
                                            Link {
                                                to: Route::EditCustomExercisePage { id: exercise.id.clone() },
                                                class: "exercise-card__edit-btn",
                                                "✏️ Edit"
                                            }
                                        }
                                        h3 {
                                            class: "exercise-card__title",
                                            onclick: {
                                                let id = exercise.id.clone();
                                                move |_| {
                                                    let mut map = instructions_open.write();
                                                    let entry = map.entry(id.clone()).or_insert(false);
                                                    *entry = !*entry;
                                                }
                                            },
                                            "{exercise.name}"
                                        }

                                        if *instructions_open.read().get(&exercise.id).unwrap_or(&false) && !exercise.instructions.is_empty() {
                                            ol { class: "exercise-card__instructions",
                                                for instruction in &exercise.instructions {
                                                    li { "{instruction}" }
                                                }
                                            }
                                        }

                                        div {
                                            class: "tag-row",
                                            span { class: "tag tag--category", "{exercise.category}" }
                                            if let Some(force) = &exercise.force {
                                                span { class: "tag tag--force", "{force}" }
                                            }
                                            if let Some(equipment) = &exercise.equipment {
                                                span { class: "tag tag--equipment", "{equipment}" }
                                            }
                                        }
                                        if !exercise.primary_muscles.is_empty() {
                                            div {
                                                class: "tag-row",
                                                for muscle in &exercise.primary_muscles {
                                                    span { class: "tag tag--muscle-primary", "{muscle}" }
                                                }
                                            }
                                        }
                                        if !exercise.secondary_muscles.is_empty() {
                                            div {
                                                class: "tag-row",
                                                for muscle in &exercise.secondary_muscles {
                                                    span { class: "tag tag--muscle-secondary", "{muscle}" }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    ExerciseCard {
                                        exercise: exercise.clone(),
                                        instructions_open: instructions_open,
                                        image_indices: image_indices,
                                    }
                                }
                            }
                        }
                    }
                }
            }
            BottomNav { active_tab: ActiveTab::Exercises }
        }
    }
}
