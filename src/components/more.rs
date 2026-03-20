use crate::components::{ActiveTab, BottomNav};
use crate::models::Exercise;
use crate::services::{exercise_db, storage};
use crate::ToastSignal;
use dioxus::prelude::*;

#[component]
pub fn More() -> Element {
    // Current exercise DB URL (defaults to the compile-time constant)
    let mut url_input = use_signal(crate::utils::get_exercise_db_url);
    let toast = consume_context::<ToastSignal>().0;
    let exercises_sig = exercise_db::use_exercises();

    // State for import conflict confirmation: exercises that need user confirmation
    // to replace an existing custom exercise.
    let mut exercises_to_confirm: Signal<Vec<Exercise>> = use_signal(Vec::new);

    let sessions = storage::use_sessions();
    let custom_exercises = storage::use_custom_exercises();
    let all_exercises = exercise_db::use_exercises();

    let save_url = move |evt: Event<FormData>| {
        evt.prevent_default();
        #[allow(unused_variables)]
        let url = crate::utils::normalize_db_url(url_input.read().trim());
        // Keep the input in sync with what was actually stored
        url_input.set(url.clone());
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    if url.is_empty() || url == crate::utils::EXERCISE_DB_BASE_URL {
                        let _ = storage.remove_item(crate::utils::EXERCISE_DB_URL_STORAGE_KEY);
                    } else {
                        let _ = storage.set_item(crate::utils::EXERCISE_DB_URL_STORAGE_KEY, &url);
                    }
                }
            }
            // Clear cached fetch timestamp so reload_exercises downloads from the new URL
            crate::services::exercise_db::clear_fetch_cache();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::services::storage::native_storage;
            if url.is_empty() || url == crate::utils::EXERCISE_DB_BASE_URL {
                let _ =
                    native_storage::remove_config_value(crate::utils::EXERCISE_DB_URL_STORAGE_KEY);
            } else {
                let _ = native_storage::set_config_value(
                    crate::utils::EXERCISE_DB_URL_STORAGE_KEY,
                    &url,
                );
            }
            // Clear cached fetch timestamp so reload_exercises downloads from the new URL
            crate::services::exercise_db::clear_fetch_cache();
        }
        // Immediately reload exercises from the (new) URL, with toast feedback
        let sig = exercises_sig;
        spawn(async move {
            exercise_db::reload_exercises(sig, toast).await;
        });
    };

    // ── Export handlers ───────────────────────────────────────────────────────

    let export_exercises = move |_| {
        let exercises = custom_exercises.read().clone();
        match serde_json::to_string_pretty(&exercises) {
            Ok(json) => {
                #[cfg(target_arch = "wasm32")]
                trigger_download("custom_exercises.json", &json);
                #[cfg(not(target_arch = "wasm32"))]
                log::info!("Export exercises (native): {}", json.len());
            }
            Err(e) => {
                let mut t = toast;
                t.set(Some(format!("⚠️ Export failed: {e}")));
            }
        }
    };

    let export_sessions = move |_| {
        let data = sessions.read().clone();
        match serde_json::to_string_pretty(&data) {
            Ok(json) => {
                #[cfg(target_arch = "wasm32")]
                trigger_download("sessions.json", &json);
                #[cfg(not(target_arch = "wasm32"))]
                log::info!("Export sessions (native): {}", json.len());
            }
            Err(e) => {
                let mut t = toast;
                t.set(Some(format!("⚠️ Export failed: {e}")));
            }
        }
    };

    // ── Import handlers ───────────────────────────────────────────────────────

    // Called after a sessions JSON file has been read.
    let handle_sessions_json = move |json: String| {
        let mut t = toast;
        match serde_json::from_str::<Vec<crate::models::WorkoutSession>>(&json) {
            Err(e) => {
                t.set(Some(format!("⚠️ Invalid sessions JSON: {e}")));
            }
            Ok(imported) => {
                let existing_ids: Vec<String> =
                    sessions.read().iter().map(|s| s.id.clone()).collect();
                let mut refused = 0usize;
                for session in imported {
                    if existing_ids.contains(&session.id) {
                        refused += 1;
                    } else {
                        storage::save_session(session);
                    }
                }
                if refused > 0 {
                    t.set(Some(format!(
                        "⚠️ {refused} session(s) refused: ID already exists"
                    )));
                }
            }
        }
    };

    // Called after a custom-exercises JSON file has been read.
    let handle_exercises_json = move |json: String| {
        let mut t = toast;
        match serde_json::from_str::<Vec<Exercise>>(&json) {
            Err(e) => {
                t.set(Some(format!("⚠️ Invalid exercises JSON: {e}")));
            }
            Ok(imported) => {
                let db = all_exercises.read();
                let customs = custom_exercises.read();
                let mut refused = 0usize;
                let mut to_add: Vec<Exercise> = Vec::new();
                let mut to_confirm: Vec<Exercise> = Vec::new();
                for exercise in imported {
                    if db.iter().any(|e| e.id == exercise.id) {
                        // Matches a built-in exercise → refuse
                        refused += 1;
                    } else if customs.iter().any(|e| e.id == exercise.id) {
                        // Matches an existing custom exercise → needs confirmation
                        to_confirm.push(exercise);
                    } else {
                        // New exercise → add directly
                        to_add.push(exercise);
                    }
                }
                drop(db);
                drop(customs);
                for exercise in to_add {
                    storage::add_custom_exercise(exercise);
                }
                if refused > 0 {
                    t.set(Some(format!(
                        "⚠️ {refused} exercise(s) refused: built-in ID conflict"
                    )));
                }
                if !to_confirm.is_empty() {
                    exercises_to_confirm.set(to_confirm);
                }
            }
        }
    };

    // Button handlers that trigger the hidden file inputs
    let open_sessions_import = move |_| {
        #[cfg(target_arch = "wasm32")]
        click_file_input("import-sessions-input");
    };

    let open_exercises_import = move |_| {
        #[cfg(target_arch = "wasm32")]
        click_file_input("import-exercises-input");
    };

    // onchange handlers for the hidden file inputs
    let on_sessions_file_change = move |_| {
        #[cfg(target_arch = "wasm32")]
        spawn(async move {
            if let Some(json) = read_file_input("import-sessions-input").await {
                handle_sessions_json(json);
            }
        });
        #[cfg(not(target_arch = "wasm32"))]
        let _ = handle_sessions_json;
    };

    let on_exercises_file_change = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            let mut handler = handle_exercises_json;
            spawn(async move {
                if let Some(json) = read_file_input("import-exercises-input").await {
                    handler(json);
                }
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = handle_exercises_json;
    };

    // ── Confirmation modal helpers ────────────────────────────────────────────

    // Confirm replacing the first exercise in the queue
    let confirm_replace = move |_| {
        let queue = exercises_to_confirm.read();
        if let Some(exercise) = queue.first().cloned() {
            drop(queue);
            storage::update_custom_exercise(exercise);
            exercises_to_confirm.write().remove(0);
        }
    };

    // Skip (refuse) replacing the first exercise in the queue
    let skip_replace = move |_| {
        exercises_to_confirm.write().remove(0);
    };

    rsx! {
        header { h1 { "⚙️ More" } }
        main { class: "more",

            // ── Data management ───────────────────────────────────────────────
            article {
                h2 { "📤 Export" }
                div { class: "inputs",
                    button {
                        class: "label save",
                        onclick: export_exercises,
                        "💾 Custom Exercises"
                    }
                    button {
                        class: "label save",
                        onclick: export_sessions,
                        "💾 Sessions"
                    }
                }
            }
            article {
                h2 { "📥 Import" }
                div { class: "inputs",
                    button {
                        class: "label more",
                        onclick: open_exercises_import,
                        "📂 Custom Exercises"
                    }
                    button {
                        class: "label more",
                        onclick: open_sessions_import,
                        "📂 Sessions"
                    }
                }
                // Hidden file inputs
                input {
                    r#type: "file",
                    id: "import-exercises-input",
                    accept: ".json",
                    style: "display:none",
                    onchange: on_exercises_file_change,
                }
                input {
                    r#type: "file",
                    id: "import-sessions-input",
                    accept: ".json",
                    style: "display:none",
                    onchange: on_sessions_file_change,
                }
            }

            // ── App ───────────────────────────────────────────────────────────
            article {
                h2 { "LogOut" }
                p { "Turn off your computer, Log your workOut." }
                p { "A simple, efficient and cross-platform workout "
                    "logging application with "
                    a {
                        href: "https://github.com/yuhonas/free-exercise-db",
                        target: "_blank",
                        "800+ exercises"
                    }
                    " built-in, by "
                    a {
                        href: "https://www.guilhemfau.re",
                        target: "_blank",
                        "Guilhem Fauré."
                    }
                }
            }

            // ── Exercise database ─────────────────────────────────────────────
            article {
                h2 { "⚙️ Exercise Database URL" }
                p { "Override the exercise database source. "
                    "Save to trigger a re-download on next reload."
                }
                form {
                    onsubmit: save_url,
                    input { r#type: "url",
                        value: "{url_input}",
                        placeholder: "{crate::utils::EXERCISE_DB_BASE_URL}",
                        oninput: move |evt| url_input.set(evt.value()),
                    }
                    button {
                        r#type: "submit",
                        class: "icon save",
                        aria_label: "Save",
                        "💾"
                    }
                }
            }

            // ── Open Source & Licences ────────────────────────────────────────
            article {
                h2 { "Open Source & Licences" }
                p { "This project is open-source under the GPL-3.0, "
                    "and uses other open-source projects. See its "
                    a {
                        href: "https://github.com/gfauredev/LogOut",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "code repository"
                    }
                    " for details. We happily accept contributions, "
                    " including to the "
                    a {
                        href: "https://github.com/gfauredev/free-exercise-db",
                        target: "_blank",
                        "exercise database"
                    }
                    "."
                }
            }

            // ── Built With ────────────────────────────────────────────────────
            article {
                h2 { "Built With" }
                ul {
                    li {
                        a {
                            href: "https://rust-lang.org",
                            target: "_blank",
                            "Rust"
                        }
                        " — Systems programming language"
                    }
                    li {
                        a {
                            href: "https://dioxuslabs.com",
                            target: "_blank",
                            "Dioxus"
                        }
                        " — Rust framework for cross-platform apps"
                    }
                    li {
                        a {
                            href: "https://github.com/yuhonas/free-exercise-db",
                            target: "_blank",
                            "Free Exercise DB"
                        }
                        " — Exercise data and images, by yuhonas"
                    }
                    li { "And many others …" }
                }
            }
        }

        // ── Confirmation modal for replacing a custom exercise ────────────────
        if let Some(exercise) = exercises_to_confirm.read().first().cloned() {
            div {
                class: "backdrop",
                onclick: skip_replace,
            }
            dialog {
                open: true,
                onclick: move |evt| evt.stop_propagation(),
                p { "Replace custom exercise "{exercise.name}"?" }
                div {
                    button {
                        class: "no label",
                        onclick: confirm_replace,
                        "💾 Replace"
                    }
                    button {
                        class: "yes",
                        onclick: skip_replace,
                        "❌"
                    }
                }
            }
        }

        BottomNav { active_tab: ActiveTab::More }
    }
}

// ── Web-only helpers ──────────────────────────────────────────────────────────

/// Trigger a file download in the browser by creating a temporary anchor element.
#[cfg(target_arch = "wasm32")]
fn trigger_download(filename: &str, content: &str) {
    use wasm_bindgen::JsCast;
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Ok(blob_parts) = js_sys::Array::new().dyn_into::<js_sys::Array>() else {
        return;
    };
    blob_parts.push(&wasm_bindgen::JsValue::from_str(content));
    let props = web_sys::BlobPropertyBag::new();
    props.set_type("application/json");
    let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&blob_parts, &props) else {
        return;
    };
    let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) else {
        return;
    };
    let Ok(anchor): Result<web_sys::HtmlAnchorElement, _> =
        document.create_element("a").and_then(|el| {
            el.dyn_into::<web_sys::HtmlAnchorElement>()
                .map_err(|_| wasm_bindgen::JsValue::NULL)
        })
    else {
        let _ = web_sys::Url::revoke_object_url(&url);
        return;
    };
    anchor.set_href(&url);
    anchor.set_download(filename);
    if let Some(body) = document.body() {
        let _ = body.append_child(&anchor);
        anchor.click();
        let _ = body.remove_child(&anchor);
    }
    let _ = web_sys::Url::revoke_object_url(&url);
}

/// Programmatically click the file input element with the given id.
#[cfg(target_arch = "wasm32")]
fn click_file_input(id: &str) {
    use wasm_bindgen::JsCast;
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    if let Some(element) = document.get_element_by_id(id) {
        if let Ok(input) = element.dyn_into::<web_sys::HtmlInputElement>() {
            input.click();
        }
    }
}

/// Read the text content of the first selected file from a file input element.
///
/// Returns `None` if no file is selected or an error occurs.
#[cfg(target_arch = "wasm32")]
async fn read_file_input(id: &str) -> Option<String> {
    use wasm_bindgen::JsCast;

    let document = web_sys::window()?.document()?;
    let input: web_sys::HtmlInputElement = document.get_element_by_id(id)?.dyn_into().ok()?;
    let files = input.files()?;
    let file = files.get(0)?;

    // Wrap FileReader in a Promise so we can await it.
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let reader = web_sys::FileReader::new().expect("FileReader");
        let reader_clone = reader.clone();
        let onload = wasm_bindgen::closure::Closure::once(move |_: web_sys::ProgressEvent| {
            let result = reader_clone.result().unwrap_or(wasm_bindgen::JsValue::NULL);
            let _ = resolve.call1(&wasm_bindgen::JsValue::NULL, &result);
        });
        let onerror = wasm_bindgen::closure::Closure::once(move |_: wasm_bindgen::JsValue| {
            let _ = reject.call0(&wasm_bindgen::JsValue::NULL);
        });
        reader.set_onload(Some(onload.as_ref().unchecked_ref()));
        reader.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        onload.forget();
        onerror.forget();
        let _ = reader.read_as_text(&file);
    });

    let result = wasm_bindgen_futures::JsFuture::from(promise).await.ok()?;
    result.as_string()
}
