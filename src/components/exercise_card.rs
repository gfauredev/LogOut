use crate::models::Exercise;
use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn ExerciseCard(exercise: Exercise) -> Element {
    let mut show_instructions = use_signal(|| false);
    let mut img_index = use_signal(|| 0usize);
    let image_count = exercise.images.len();

    rsx! {
        article {
            key: "{exercise.id}",
            class: "exercise-card",

            div {
                class: "exercise-card__custom-header",
                Link {
                    to: Route::EditCustomExercisePage { id: exercise.id.clone() },
                    class: "exercise-card__edit-btn",
                    "✏️ Edit"
                }
            }

            h3 {
                class: "exercise-card__title",
                onclick: move |_| {
                    let current = *show_instructions.read();
                    show_instructions.set(!current);
                },
                "{exercise.name}"
            }

            if *show_instructions.read() && !exercise.instructions.is_empty() {
                ol { class: "exercise-card__instructions",
                    for instruction in &exercise.instructions {
                        li { "{instruction}" }
                    }
                }
            }

            if let Some(image_url) = exercise.get_image_url(*img_index.read()) {
                img {
                    src: "{image_url}",
                    alt: "{exercise.name}",
                    loading: "lazy",
                    class: "exercise-card__image",
                    onclick: move |_| {
                        if image_count > 1 {
                            let next = (*img_index.read() + 1) % image_count;
                            img_index.set(next);
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
                if let Some(level) = &exercise.level {
                    span { class: "tag tag--level", "{level}" }
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
    }
}
