use std::collections::HashMap;
use dioxus::prelude::*;
use crate::models::Exercise;

#[component]
pub fn ExerciseCard(
    exercise: Exercise,
    instructions_open: Signal<HashMap<String, bool>>,
    image_indices: Signal<HashMap<String, usize>>,
) -> Element {
    let id = exercise.id.clone();
    let id_for_img = exercise.id.clone();
    let show_instructions = *instructions_open.read().get(&id).unwrap_or(&false);
    let img_index = *image_indices.read().get(&id_for_img).unwrap_or(&0);
    let image_count = exercise.images.len();

    rsx! {
        article {
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
                        if image_count > 1 {
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
