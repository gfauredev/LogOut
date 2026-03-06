use crate::models::{get_current_timestamp, Exercise};
use crate::services::storage;
use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn ExerciseCard(
    exercise: Exercise,
    is_custom: bool,
    show_instructions_initial: Option<bool>,
) -> Element {
    let initial = show_instructions_initial.unwrap_or(false);
    let mut show_instructions = use_signal(move || initial);
    let mut img_index = use_signal(|| 0usize);
    let image_count = exercise.images.len();

    rsx! {
        article { key: "{exercise.id}",
            header {
                h2 {
                    onclick: move |_| {
                        let current = *show_instructions.read();
                        show_instructions.set(!current);
                    },
                    "{exercise.name}"
                }
                if is_custom {
                    Link { class: "edit",
                        to: Route::EditExercise { id: exercise.id.clone() },
                        title: "Edit",
                        "✏️"
                    }
                } else {
                    button { class: "add",
                        onclick: {
                            let exercise = exercise.clone();
                            move |_| {
                                let timestamp = get_current_timestamp();
                                let clone = Exercise {
                                    id: format!("custom_{}", timestamp),
                                    name: exercise.name.clone(),
                                    name_lower: exercise.name_lower.clone(),
                                    category: exercise.category,
                                    force: exercise.force,
                                    level: exercise.level,
                                    mechanic: exercise.mechanic,
                                    equipment: exercise.equipment,
                                    primary_muscles: exercise.primary_muscles.clone(),
                                    secondary_muscles: exercise.secondary_muscles.clone(),
                                    instructions: exercise.instructions.clone(),
                                    images: exercise.images.clone(),
                                };
                                let clone_id = clone.id.clone();
                                storage::add_custom_exercise(clone);
                                navigator()
                                    .push(Route::EditExercise { id: clone_id });
                            }
                        },
                        title: "Clone then edit",
                        "+"
                    }
                }
            }
            if *show_instructions.read() && !exercise.instructions.is_empty() {
                ol {
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
                    onclick: move |_| {
                        if image_count > 1 {
                            let next = (*img_index.read() + 1) % image_count;
                            img_index.set(next);
                        }
                    },
                }
            }
            ul {
                li { class: "category", "{exercise.category}" }
                if let Some(force) = &exercise.force {
                    li { class: "force", "{force}" }
                }
                if let Some(equipment) = &exercise.equipment {
                    li { class: "equipment", "{equipment}" }
                }
                if let Some(level) = &exercise.level {
                    li { class: "level", "{level}" }
                }
            }
            if !exercise.primary_muscles.is_empty() {
                ul {
                    for muscle in &exercise.primary_muscles {
                        li { class: "primary", "{muscle}" }
                    }
                }
            }
            if !exercise.secondary_muscles.is_empty() {
                ul {
                    for muscle in &exercise.secondary_muscles {
                        li { class: "secondary", "{muscle}" }
                    }
                }
            }
        }
    }
}
