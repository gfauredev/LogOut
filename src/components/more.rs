use crate::components::{ActiveTab, BottomNav};
use crate::models::Exercise;
use crate::services::{exercise_db, storage};
use crate::{ImageDownloadProgressSignal, ToastSignal};
use dioxus::prelude::*;
use dioxus_i18n::t;
#[component]
pub fn More() -> Element {
    let mut url_input = use_signal(crate::utils::get_exercise_db_url);
    let mut toast = consume_context::<ToastSignal>().0;
    let exercises_sig = exercise_db::use_exercises();
    let mut exercises_to_confirm: Signal<Vec<Exercise>> = use_signal(Vec::new);
    let sessions = storage::use_sessions();
    let custom_exercises = storage::use_custom_exercises();
    let all_exercises = exercise_db::use_exercises();
    let img_progress = consume_context::<ImageDownloadProgressSignal>().0;
    // Count of cached images on native (computed asynchronously from the image directory).
    #[cfg(not(target_arch = "wasm32"))]
    let image_count_resource = use_resource(move || {
        let exercises = exercises_sig.read().clone();
        async move {
            use crate::services::storage::native_storage;
            let images_dir = native_storage::data_dir().join("images");
            exercises
                .iter()
                .flat_map(|e| e.images.iter())
                .filter(|key| {
                    !key.contains("://") && !key.starts_with("idb:") && !key.starts_with("local:")
                })
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .filter(|key| images_dir.join(key).exists())
                .count()
        }
    });
    // Flatten the platform-specific resource into a simple Option<usize> for use in rsx!.
    #[cfg(not(target_arch = "wasm32"))]
    let image_count_opt: Option<usize> = {
        let guard = image_count_resource.read();
        *guard
    };
    #[cfg(target_arch = "wasm32")]
    let image_count_opt: Option<usize> = None;
    // Pre-compute translated toast message prefixes at render time.
    // Export-failed strings are used in closures that clone before capture, so String is OK.
    let msg_export_failed = t!("toast-export-failed");
    let msg_export_sessions_failed = t!("toast-export-sessions-failed");
    // Invalid-JSON strings are used in closures that must remain FnMut (captured by async move).
    // use_memo returns Memo<String> which is Copy, so these closures stay FnMut on WASM.
    let msg_sessions_invalid = use_memo(|| t!("toast-sessions-invalid"));
    let msg_exercises_invalid = use_memo(|| t!("toast-exercises-invalid"));
    let msg_sessions_refused = use_memo(|| t!("more-sessions-refused"));
    let msg_exercises_refused = use_memo(|| t!("more-exercises-refused"));
    let save_url = move |evt: Event<FormData>| {
        evt.prevent_default();
        let url = crate::utils::normalize_db_url(url_input.read().trim());
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
            crate::services::exercise_db::clear_fetch_cache();
        }
        let sig = exercises_sig;
        spawn(async move {
            exercise_db::reload_exercises(sig, toast, img_progress).await;
        });
    };
    let export_exercises = {
        let msg_export_failed = msg_export_failed.clone();
        move |_| {
            let exercises = custom_exercises.read().clone();
            match serde_json::to_string_pretty(&exercises) {
                Ok(json) => {
                    if let Some(msg) = trigger_download("custom_exercises.json", &json) {
                        toast.write().push_back(msg);
                    }
                }
                Err(e) => {
                    toast.write().push_back(format!("{msg_export_failed}: {e}"));
                }
            }
        }
    };
    let export_sessions = move |_| {
        let msg_export_sessions_failed = msg_export_sessions_failed.clone();
        let msg_export_failed = msg_export_failed.clone();
        let mut t = toast;
        spawn(async move {
            let active = sessions.peek().clone();
            let mut all = active;
            let mut offset = 0usize;
            let page_size = 500usize;
            loop {
                match storage::load_completed_sessions_page(page_size, offset).await {
                    Ok(page) => {
                        let fetched = page.len();
                        all.extend(page);
                        if fetched < page_size {
                            break;
                        }
                        offset += fetched;
                    }
                    Err(e) => {
                        t.write()
                            .push_back(format!("{msg_export_sessions_failed}: {e}"));
                        return;
                    }
                }
            }
            all.sort_by(|a, b| a.start_time.cmp(&b.start_time));
            match serde_json::to_string_pretty(&all) {
                Ok(json) => {
                    if let Some(msg) = trigger_download("sessions.json", &json) {
                        t.write().push_back(msg);
                    }
                }
                Err(e) => {
                    t.write().push_back(format!("{msg_export_failed}: {e}"));
                }
            }
        });
    };
    let handle_sessions_json = move |json: String| {
        let mut t = toast;
        match serde_json::from_str::<Vec<crate::models::WorkoutSession>>(&json) {
            Err(e) => {
                t.write()
                    .push_back(format!("{}: {e}", msg_sessions_invalid()));
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
                    t.write()
                        .push_back(format!("⚠️ {refused} {}", msg_sessions_refused()));
                }
            }
        }
    };
    let handle_exercises_json = move |json: String| {
        let mut t = toast;
        match serde_json::from_str::<Vec<Exercise>>(&json) {
            Err(e) => {
                t.write()
                    .push_back(format!("{}: {e}", msg_exercises_invalid()));
            }
            Ok(imported) => {
                let db = all_exercises.read();
                let customs = custom_exercises.read();
                let mut refused = 0usize;
                let mut to_add: Vec<Exercise> = Vec::new();
                let mut to_confirm: Vec<Exercise> = Vec::new();
                for exercise in imported {
                    if db.iter().any(|e| e.id == exercise.id) {
                        refused += 1;
                    } else if customs.iter().any(|e| e.id == exercise.id) {
                        to_confirm.push(exercise);
                    } else {
                        to_add.push(exercise);
                    }
                }
                drop(db);
                drop(customs);
                for exercise in to_add {
                    storage::add_custom_exercise(exercise);
                }
                if refused > 0 {
                    t.write()
                        .push_back(format!("⚠️ {refused} {}", msg_exercises_refused()));
                }
                if !to_confirm.is_empty() {
                    exercises_to_confirm.set(to_confirm);
                }
            }
        }
    };
    let on_sessions_file_change = move |_| {
        spawn(async move {
            if let Some(json) = read_file_input("import-sessions-input").await {
                handle_sessions_json(json);
            }
        });
    };
    let on_exercises_file_change = move |_| {
        let mut handler = handle_exercises_json;
        spawn(async move {
            if let Some(json) = read_file_input("import-exercises-input").await {
                handler(json);
            }
        });
    };
    let confirm_replace = move |_| {
        let queue = exercises_to_confirm.read();
        if let Some(exercise) = queue.first().cloned() {
            drop(queue);
            storage::update_custom_exercise(exercise);
            exercises_to_confirm.write().remove(0);
        }
    };
    let skip_replace = move |_| {
        exercises_to_confirm.write().remove(0);
    };
    rsx! {
        Stylesheet { href: asset!("/assets/more.scss") }
        header {
            h1 { {t!("more-title")} }
        }
        main { class: "more",
            article {
                h2 { {t!("more-export-section")} }
                div { class: "inputs",
                    button { class: "label save", onclick: export_exercises,
                        {t!("more-export-exercises-btn", count : custom_exercises.read().len())}
                    }
                    button { class: "label save", onclick: export_sessions,
                        {t!("more-export-sessions-btn", count : sessions.read().len())}
                    }
                }
            }
            article {
                h2 { {t!("more-import-section")} }
                div { class: "inputs",
                    label { class: "label more", r#for: "import-exercises-input",
                        {t!("more-import-exercises-btn")}
                    }
                    label { class: "label more", r#for: "import-sessions-input",
                        {t!("more-import-sessions-btn")}
                    }
                }
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
            article {
                h2 { {t!("more-about-section")} }
                p { {t!("app-subtitle")} }
                p {
                    {t!("more-about-desc-a")}
                    " "
                    a {
                        href: "https://github.com/yuhonas/free-exercise-db",
                        target: "_blank",
                        {t!("more-about-exercises-link")}
                    }
                    " "
                    {t!("more-about-desc-b")}
                    " "
                    a { href: "https://www.guilhemfau.re", target: "_blank", "Guilhem Fauré." }
                }
            }
            article {
                h2 { {t!("more-db-url-section")} }
                p { {t!("more-db-url-desc")} }
                p { {t!("more-db-exercises-count", count : exercises_sig.read().len())} }
                if let Some(img_count) = image_count_opt {
                    p { {t!("more-db-images-count", count : img_count)} }
                }
                form { onsubmit: save_url,
                    input {
                        r#type: "url",
                        value: "{url_input}",
                        placeholder: "{crate::utils::EXERCISE_DB_BASE_URL}",
                        oninput: move |evt| url_input.set(evt.value()),
                    }
                    button {
                        r#type: "submit",
                        class: "icon save",
                        aria_label: t!("more-db-url-save-aria"),
                        "💾"
                    }
                }
            }
            article {
                h2 { {t!("more-oss-section")} }
                p {
                    {t!("more-oss-desc-a")}
                    " "
                    a {
                        href: "https://github.com/gfauredev/LogOut",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        {t!("more-oss-repo-link")}
                    }
                    " "
                    {t!("more-oss-desc-b")}
                    " "
                    a {
                        href: "https://github.com/gfauredev/free-exercise-db",
                        target: "_blank",
                        {t!("more-oss-db-link")}
                    }
                    "."
                }
            }
            article {
                h2 { {t!("more-built-with-section")} }
                ul {
                    li {
                        a { href: "https://rust-lang.org", target: "_blank", "Rust" }
                        " — "
                        {t!("more-built-with-rust")}
                    }
                    li {
                        a { href: "https://dioxuslabs.com", target: "_blank", "Dioxus" }
                        " — "
                        {t!("more-built-with-dioxus")}
                    }
                    li {
                        a {
                            href: "https://github.com/yuhonas/free-exercise-db",
                            target: "_blank",
                            "Free Exercise DB"
                        }
                        " — "
                        {t!("more-built-with-freeexdb")}
                    }
                    li { {t!("more-built-with-others")} }
                }
            }
            article {
                h2 { {t!("more-privacy-section")} }
                p { {t!("more-privacy-desc")} }
            }
        }
        if let Some(exercise) = exercises_to_confirm.read().first().cloned() {
            div { class: "backdrop", onclick: skip_replace }
            dialog { open: true, onclick: move |evt| evt.stop_propagation(),
                p { {t!("more-replace-confirm", name : exercise.name.clone())} }
                div {
                    button { class: "no label", onclick: confirm_replace, {t!("more-replace-btn")} }
                    button { class: "yes", onclick: skip_replace, "❌" }
                }
            }
        }
        BottomNav { active_tab: ActiveTab::More }
    }
}
/// Trigger a file download.
///
/// On WASM the `web_sys` DOM APIs are used directly for efficiency.
/// On Android, the file is written to the app's exports directory and
/// `Some(message)` is returned so the caller can show a toast with the path.
/// `<a download>` does not work reliably in Android WebView, so native I/O
/// is used instead.
/// On other native targets (desktop) the same Blob/anchor download is driven
/// through `document::eval` so the Dioxus `WebView` executes JavaScript.
///
/// Returns `Some(message)` when there is something worth reporting to the user
/// (Android: the path the file was saved to), `None` otherwise.
fn trigger_download(filename: &str, content: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        let Some(window) = web_sys::window() else {
            return None;
        };
        let Some(document) = window.document() else {
            return None;
        };
        let Ok(blob_parts) = js_sys::Array::new().dyn_into::<js_sys::Array>() else {
            return None;
        };
        blob_parts.push(&wasm_bindgen::JsValue::from_str(content));
        let props = web_sys::BlobPropertyBag::new();
        props.set_type("application/json");
        let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&blob_parts, &props) else {
            return None;
        };
        let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) else {
            return None;
        };
        let Ok(anchor): Result<web_sys::HtmlAnchorElement, _> =
            document.create_element("a").and_then(|el| {
                el.dyn_into::<web_sys::HtmlAnchorElement>()
                    .map_err(|_| wasm_bindgen::JsValue::NULL)
            })
        else {
            let _ = web_sys::Url::revoke_object_url(&url);
            return None;
        };
        anchor.set_href(&url);
        anchor.set_download(filename);
        if let Some(body) = document.body() {
            let _ = body.append_child(&anchor);
            anchor.click();
            let _ = body.remove_child(&anchor);
        }
        let _ = web_sys::Url::revoke_object_url(&url);
        None
    }
    #[cfg(target_os = "android")]
    {
        // `<a download>` is not handled by Android WebView without a custom
        // DownloadListener.  Write the file directly to the app's exports
        // directory instead and return a message for the caller to toast.
        use crate::services::storage::native_storage;
        let exports_dir = native_storage::data_dir().join("exports");
        if let Err(e) = std::fs::create_dir_all(&exports_dir) {
            log::warn!("Failed to create exports dir: {e}");
            return None;
        }
        let path = exports_dir.join(filename);
        match std::fs::write(&path, content.as_bytes()) {
            Ok(()) => {
                log::info!("Exported {} to {}", filename, path.display());
                Some(format!("💾 {}", path.display()))
            }
            Err(e) => {
                log::warn!("Failed to write export {filename}: {e}");
                None
            }
        }
    }
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
    {
        // Encode content and filename as JSON strings so they are safely embedded
        // in the JavaScript snippet without any injection risk.
        let content_js = serde_json::to_string(content).unwrap_or_default();
        let filename_js = serde_json::to_string(filename).unwrap_or_default();
        document::eval(&format!(
            r"(function(){{
  var b=new Blob([{content_js}],{{type:'application/json'}});
  var u=URL.createObjectURL(b);
  var a=document.createElement('a');
  a.href=u; a.download={filename_js};
  document.body.appendChild(a); a.click(); document.body.removeChild(a);
  setTimeout(function(){{URL.revokeObjectURL(u);}},100);
}})();"
        ));
        None
    }
}
/// Read the text content of the first selected file from a file input element.
///
/// Returns `None` if no file is selected or an error occurs.
///
/// On WASM the `web_sys` `FileReader` API is used.  On native the read is
/// performed inside the `WebView` via `document::eval` and the result is
/// returned through `dioxus.send()`.
async fn read_file_input(id: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        let document = web_sys::window()?.document()?;
        let input: web_sys::HtmlInputElement = document.get_element_by_id(id)?.dyn_into().ok()?;
        let files = input.files()?;
        let file = files.get(0)?;
        let promise = js_sys::Promise::new(&mut |resolve, reject| {
            let Ok(reader) = web_sys::FileReader::new() else {
                let _ = reject.call0(&wasm_bindgen::JsValue::NULL);
                return;
            };
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
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Use the WebView's FileReader API via eval; send the text (or null) back.
        let js = format!(
            r"(function(){{
  var input=document.getElementById('{id}');
  var file=input&&input.files&&input.files[0];
  if(!file){{dioxus.send(null);return;}}
  var r=new FileReader();
  r.onload=function(e){{dioxus.send(e.target.result);}};
  r.onerror=function(){{dioxus.send(null);}};
  r.readAsText(file);
}})();"
        );
        let mut eval = document::eval(&js);
        eval.recv::<serde_json::Value>().await.ok().and_then(|v| {
            if v.is_null() {
                None
            } else {
                v.as_str().map(str::to_owned)
            }
        })
    }
}
