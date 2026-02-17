use dioxus::prelude::*;
use crate::services::exercise_db;
use crate::Route;

#[component]
pub fn ExerciseListPage() -> Element {
    let all_exercises = exercise_db::use_exercises();
    let mut search_query = use_signal(|| String::new());
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
        div {
            class: "container container--narrow",
            
            header {
                class: "page-header",
                Link {
                    to: Route::HomePage {},
                    class: "back-link",
                    "← Back"
                }
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
                    div {
                        key: "{exercise.id}",
                        class: "exercise-card",
                        
                        h3 { class: "exercise-card__title", "{exercise.name}" }
                        
                        if let Some(image_url) = exercise.get_first_image_url() {
                            img {
                                src: "{image_url}",
                                alt: "{exercise.name}",
                                loading: "lazy",
                                class: "exercise-card__image",
                            }
                        }
                        
                        div {
                            class: "tag-row",
                            span { class: "tag tag--category", "{exercise.category}" }
                            span { class: "tag tag--level", "{exercise.level}" }
                            if let Some(equipment) = &exercise.equipment {
                                span { class: "tag tag--equipment", "{equipment}" }
                            }
                        }
                        
                        div {
                            class: "exercise-card__muscles",
                            "Target: {exercise.primary_muscles.join(\", \")}"
                        }
                    }
                }
            }
        }
    }
}
