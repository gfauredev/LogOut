use crate::models::{get_current_timestamp, DbI18n, Exercise};
use crate::services::storage;
use crate::{DbI18nSignal, Route};
use dioxus::prelude::*;
use dioxus_i18n::{prelude::i18n, t};

/// Looks up the translation for a single enum value in the `i18n.json` data.
///
/// Falls back to the English `value` string when:
/// - the language has no entry in the map, or
/// - the field has no entry for the given value.
///
/// `lang` is a BCP-47 tag (e.g. `"fr"` or `"fr-FR"`).  Prefix matching
/// (e.g. `"fr-FR"` → `"fr"`) is attempted automatically.
fn translate_enum<'a>(db_i18n: &'a DbI18n, lang: &str, field: &str, value: &'a str) -> &'a str {
    let lookup = |l: &str| -> Option<&'a str> {
        let lang_data = db_i18n.get(l)?;
        let map = match field {
            "force" => &lang_data.force,
            "level" => &lang_data.level,
            "mechanic" => &lang_data.mechanic,
            "equipment" => &lang_data.equipment,
            "category" => &lang_data.category,
            "muscles" => &lang_data.muscles,
            _ => return None,
        };
        map.get(value).map(String::as_str)
    };
    lookup(lang)
        .or_else(|| lang.split('-').next().and_then(lookup))
        .unwrap_or(value)
}

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

    // Consume the enum-translation context provided by exercise_loader.
    let db_i18n_sig = use_context::<DbI18nSignal>().0;

    // Memoize translated name and instructions so they are only recomputed when
    // the i18n language context changes (rare) rather than on every render.
    let display_name = {
        let ex = exercise.clone();
        use_memo(move || {
            let lang = i18n().language();
            ex.name_for_lang(&lang.to_string()).to_owned()
        })
    };
    let display_instructions = {
        let ex = exercise.clone();
        use_memo(move || {
            let lang = i18n().language();
            ex.instructions_for_lang(&lang.to_string()).to_vec()
        })
    };

    // Translated enum labels (category, force, equipment, level, muscles).
    // Memoised so they are only recomputed when the language or the i18n data signal changes.
    let enum_labels = {
        let ex = exercise.clone();
        use_memo(move || {
            let lang = i18n().language().to_string();
            let db_i18n = db_i18n_sig.read();
            let category =
                translate_enum(&db_i18n, &lang, "category", ex.category.as_ref()).to_owned();
            let force = ex
                .force
                .map(|f| translate_enum(&db_i18n, &lang, "force", f.as_ref()).to_owned());
            let equipment = ex
                .equipment
                .map(|e| translate_enum(&db_i18n, &lang, "equipment", e.as_ref()).to_owned());
            let level = ex
                .level
                .map(|l| translate_enum(&db_i18n, &lang, "level", l.as_ref()).to_owned());
            let primary_muscles: Vec<String> = ex
                .primary_muscles
                .iter()
                .map(|m| translate_enum(&db_i18n, &lang, "muscles", m.as_ref()).to_owned())
                .collect();
            let secondary_muscles: Vec<String> = ex
                .secondary_muscles
                .iter()
                .map(|m| translate_enum(&db_i18n, &lang, "muscles", m.as_ref()).to_owned())
                .collect();
            (
                category,
                force,
                equipment,
                level,
                primary_muscles,
                secondary_muscles,
            )
        })
    };

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
            if *show_instructions.read() && !display_instructions.read().is_empty() {
                ol {
                    for instruction in display_instructions.read().iter() {
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
                li { class: "category", "{enum_labels.read().0}" }
                if let Some(label) = &enum_labels.read().1 {
                    li { class: "force", "{label}" }
                }
                if let Some(label) = &enum_labels.read().2 {
                    li { class: "equipment", "{label}" }
                }
                if let Some(label) = &enum_labels.read().3 {
                    li { class: "level", "{label}" }
                }
                // {
                //     let (tag_class, tag_label) = exercise.type_tag();
                //     rsx! { li { class: "{tag_class}", "{tag_label}" } }
                // }
            }
            if !exercise.primary_muscles.is_empty() {
                ul {
                    for label in enum_labels.read().4.iter() {
                        li { class: "primary-muscle", "{label}" }
                    }
                }
            }
            if !exercise.secondary_muscles.is_empty() {
                ul {
                    for label in enum_labels.read().5.iter() {
                        li { class: "secondary-muscle", "{label}" }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DbI18nLang;

    fn sample_db_i18n() -> DbI18n {
        let mut lang = DbI18nLang::default();
        lang.category
            .insert("strength".into(), "musculation".into());
        lang.force.insert("push".into(), "poussée".into());
        lang.equipment.insert("barbell".into(), "barre".into());
        lang.level.insert("beginner".into(), "débutant".into());
        lang.muscles.insert("chest".into(), "pectoraux".into());
        let mut map = DbI18n::new();
        map.insert("fr".into(), lang);
        map
    }

    #[test]
    fn translate_enum_exact_match() {
        let db_i18n = sample_db_i18n();
        assert_eq!(
            translate_enum(&db_i18n, "fr", "category", "strength"),
            "musculation"
        );
        assert_eq!(translate_enum(&db_i18n, "fr", "force", "push"), "poussée");
        assert_eq!(
            translate_enum(&db_i18n, "fr", "equipment", "barbell"),
            "barre"
        );
        assert_eq!(
            translate_enum(&db_i18n, "fr", "level", "beginner"),
            "débutant"
        );
        assert_eq!(
            translate_enum(&db_i18n, "fr", "muscles", "chest"),
            "pectoraux"
        );
    }

    #[test]
    fn translate_enum_prefix_match() {
        let db_i18n = sample_db_i18n();
        // "fr-FR" should fall back to "fr" key
        assert_eq!(
            translate_enum(&db_i18n, "fr-FR", "category", "strength"),
            "musculation"
        );
    }

    #[test]
    fn translate_enum_missing_lang_returns_original() {
        let db_i18n = sample_db_i18n();
        assert_eq!(
            translate_enum(&db_i18n, "de", "category", "strength"),
            "strength"
        );
    }

    #[test]
    fn translate_enum_missing_key_returns_original() {
        let db_i18n = sample_db_i18n();
        // "cardio" is not in the sample French map
        assert_eq!(
            translate_enum(&db_i18n, "fr", "category", "cardio"),
            "cardio"
        );
    }

    #[test]
    fn translate_enum_unknown_field_returns_original() {
        let db_i18n = sample_db_i18n();
        assert_eq!(
            translate_enum(&db_i18n, "fr", "unknown_field", "strength"),
            "strength"
        );
    }
}
