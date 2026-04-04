use crate::models::{get_current_timestamp, DbI18n, Exercise};
use crate::services::storage;
use crate::{DbI18nSignal, Route};
use dioxus::prelude::*;
use dioxus_i18n::{prelude::i18n, t};
use std::sync::Arc;

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

/// Renders a single exercise image, handling both regular URLs and `idb:`-prefixed
/// keys that require async loading from `IndexedDB` on web.  Clicking cycles through
/// multiple images when more than one is available.
#[component]
fn ExerciseImage(exercise: Arc<Exercise>, display_name: String) -> Element {
    let mut img_index = use_signal(|| 0usize);
    let image_count = exercise.images.len();

    // On native platforms, subscribe to image-download progress so that the
    // URL is re-evaluated each time a new image is saved to disk.  This lets
    // images appear progressively as they are downloaded.
    #[cfg(not(target_arch = "wasm32"))]
    let img_progress = use_context::<crate::ImageDownloadProgressSignal>().0;

    // Synchronous URL via the shared model method (covers all non-idb: keys).
    let sync_url = {
        let ex = exercise.clone();
        use_memo(move || {
            // Track download progress as a reactive dependency so the URL is
            // recomputed whenever a new image finishes downloading.
            #[cfg(not(target_arch = "wasm32"))]
            let _ = img_progress.read();
            ex.get_image_url(*img_index.read())
        })
    };

    // Async blob URL for `idb:`-prefixed keys (web only).
    #[cfg(target_arch = "wasm32")]
    let idb_url = {
        let ex = exercise.clone();
        use_resource(move || {
            let ex = ex.clone();
            async move {
                let key = ex.images.get(*img_index.read())?.clone();
                let image_key = key.strip_prefix("idb:")?;
                crate::services::storage::idb_images::get_image_blob_url(image_key).await
            }
        })
    };

    // Revoke stale `blob:` URLs when the resource produces a new value or the
    // component is unmounted, to avoid leaking object-URL memory.
    #[cfg(target_arch = "wasm32")]
    let prev_blob_url: Signal<Option<String>> = use_signal(|| None);

    #[cfg(target_arch = "wasm32")]
    {
        let mut slot = prev_blob_url;
        use_effect(move || {
            let new_url: Option<String> = idb_url.read().as_ref().and_then(|r| r.clone());
            let mut s = slot.write();
            if let Some(old) = s.as_deref() {
                if Some(old) != new_url.as_deref() {
                    let _ = web_sys::Url::revoke_object_url(old);
                }
            }
            *s = new_url;
        });
        use_drop(move || {
            if let Some(url) = slot.peek().as_deref() {
                let _ = web_sys::Url::revoke_object_url(url);
            }
        });
    }

    let display_url: Option<String> = {
        #[cfg(target_arch = "wasm32")]
        {
            let is_idb = exercise
                .images
                .get(*img_index.read())
                .map_or(false, |k| k.starts_with("idb:"));
            if is_idb {
                idb_url.read().as_ref().and_then(|r| r.clone())
            } else {
                sync_url.read().clone()
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            sync_url.read().clone()
        }
    };

    if let Some(url) = display_url {
        rsx! {
            img {
                src: "{url}",
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
    } else {
        rsx! {
            span { class: "img-loading", "⬇️" }
        }
    }
}

#[component]
pub fn ExerciseCard(
    exercise: Arc<Exercise>,
    is_custom: bool,
    show_instructions_initial: Option<bool>,
) -> Element {
    let initial = show_instructions_initial.unwrap_or(false);
    let mut show_instructions = use_signal(move || initial);
    let db_i18n_sig = use_context::<DbI18nSignal>().0;

    // Resolve the locale string once per language change.  All three memos
    // below read this shared value so the BCP-47 lookup and prefix fallback
    // run only once per locale update, not three times.
    let lang_str = use_memo(move || i18n().language().to_string());

    let display_name = {
        let ex = exercise.clone();
        use_memo(move || ex.name_for_lang(&lang_str.read()).to_owned())
    };

    let display_instructions = {
        let ex = exercise.clone();
        use_memo(move || ex.instructions_for_lang(&lang_str.read()).to_vec())
    };

    let enum_labels = {
        let ex = exercise.clone();
        use_memo(move || {
            let lang = lang_str.read();
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
                    Link {
                        class: "edit",
                        to: Route::EditExercise {
                            id: exercise.id.clone(),
                        },
                        title: t!("exercise-edit"),
                        "✏️"
                    }
                } else {
                    button {
                        class: "more",
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
                                    .push(Route::EditExercise {
                                        id: clone_id,
                                    });
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
            if !exercise.images.is_empty() {
                ExerciseImage {
                    exercise: exercise.clone(),
                    display_name: display_name.read().clone(),
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
            "musculation",
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
        assert_eq!(
            translate_enum(&db_i18n, "fr-FR", "category", "strength"),
            "musculation",
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
            "strength",
        );
    }
}
