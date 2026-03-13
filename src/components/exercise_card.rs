use crate::models::{get_current_timestamp, Exercise};
use crate::services::storage;
use crate::Route;
use dioxus::prelude::*;
use dioxus_i18n::{prelude::i18n, t};

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

    let lang = i18n().language();
    let lang_str = lang.to_string();
    let display_name = exercise.name_for_lang(&lang_str).to_owned();
    let display_instructions: Vec<String> =
        exercise.instructions_for_lang(&lang_str).to_vec();

    rsx! {
        article { key: "{exercise.id}",
            header {
                h2 {
                    onclick: move |_| {
                        let current = *show_instructions.read();
                        show_instructions.set(!current);
                    },
                    "{display_name}"
                }
                if is_custom {
                    Link { class: "edit",
                        to: Route::EditExercise { id: exercise.id.clone() },
                        title: t!("exercise-edit"),
                        "✏️"
                    }
                } else {
                    button { class: "add",
                        onclick: {
                            let exercise = exercise.clone();
                            move |_| {
                                let timestamp = get_current_timestamp();
                                let clone = Exercise {
                                    id: format!("custom_{timestamp}"),
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
                                    i18n: None,
                                };
                                let clone_id = clone.id.clone();
                                storage::add_custom_exercise(clone);
                                navigator()
                                    .push(Route::EditExercise { id: clone_id });
                            }
                        },
                        title: t!("exercise-clone"),
                        "+"
                    }
                }
            }
            if *show_instructions.read() && !display_instructions.is_empty() {
                ol {
                    for instruction in &display_instructions {
                        li { "{instruction}" }
                    }
                }
            }
            if let Some(image_url) = exercise.get_image_url(*img_index.read()) {
                img {
                    src: "{image_url}",
                    alt: "{display_name}",
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
                // {
                //     let (tag_class, tag_label) = exercise.type_tag();
                //     rsx! { li { class: "{tag_class}", "{tag_label}" } }
                // }
            }
            if !exercise.primary_muscles.is_empty() {
                ul {
                    for muscle in &exercise.primary_muscles {
                        li { class: "primary-muscle", "{muscle}" }
                    }
                }
            }
            if !exercise.secondary_muscles.is_empty() {
                ul {
                    for muscle in &exercise.secondary_muscles {
                        li { class: "secondary-muscle", "{muscle}" }
                    }
                }
            }
        }
    }
}
