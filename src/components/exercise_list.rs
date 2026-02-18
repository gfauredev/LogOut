use std::collections::HashMap;
use dioxus::prelude::*;
use crate::services::exercise_db;
use crate::components::{BottomNav, ActiveTab};

#[component]
pub fn ExerciseListPage() -> Element {
    let all_exercises = exercise_db::use_exercises();
    let mut search_query = use_signal(|| String::new());
    let mut instructions_open = use_signal(|| HashMap::<String, bool>::new());
    let mut image_indices = use_signal(|| HashMap::<String, usize>::new());
    let exercises = use_memo(move || {
        let query = search_query.read();
        let all = all_exercises.read();
        if query.is_empty() {
            all.iter().take(50).cloned().collect::<Vec<_>>()
        } else {
            exercise_db::search_exercises(&all, &query)
                .into_iter()
                .take(50)
                .collect::<Vec<_>>()
        }
    });

    let total = all_exercises.read().len();

    rsx! {
        div { class: "page-container",
            div { class: "page-content container container--narrow",
            
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
                
                div {
                    class: "exercise-list",
                    for exercise in exercises() {
                        {
                            let id = exercise.id.clone();
                            let id_for_img = exercise.id.clone();
                            let show_instructions = *instructions_open.read().get(&id).unwrap_or(&false);
                            let img_index = *image_indices.read().get(&id_for_img).unwrap_or(&0);
                            let image_count = exercise.images.len();
                            rsx! {
                                div {
                                    key: "{exercise.id}",
                                    class: "exercise-card",

                                    h3 {
                                        class: "exercise-card__title",
                                        onclick: move |_| {
                                            let mut map = instructions_open.write();
                                            let entry = map.entry(id.clone()).or_insert(false);
                                            *entry = !*entry;
                                        },
                                        "{exercise.name}"
                                    }

                                    if show_instructions && !exercise.instructions.is_empty() {
                                        ol { class: "exercise-card__instructions",
                                            for instruction in &exercise.instructions {
                                                li { "{instruction}" }
                                            }
                                        }
                                    }

                                    if let Some(image_url) = exercise.get_image_url(img_index) {
                                        img {
                                            src: "{image_url}",
                                            alt: "{exercise.name}",
                                            loading: "lazy",
                                            class: "exercise-card__image",
                                            onclick: move |_| {
                                                if image_count > 0 {
                                                    let mut map = image_indices.write();
                                                    let entry = map.entry(id_for_img.clone()).or_insert(0);
                                                    *entry = (*entry + 1) % image_count;
                                                }
                                            },
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
                                        span { class: "tag tag--level", "{exercise.level}" }
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
                            }
                        }
                    }
                }
            }
            BottomNav { active_tab: ActiveTab::Exercises }
        }
    }
}
