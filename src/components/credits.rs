use crate::components::{ActiveTab, BottomNav};
use dioxus::prelude::*;

#[component]
pub fn CreditsPage() -> Element {
    // Current exercise DB URL (defaults to the compile-time constant)
    let mut url_input = use_signal(crate::utils::get_exercise_db_url);

    let save_url = move |evt: Event<FormData>| {
        evt.prevent_default();
        #[allow(unused_variables)]
        let url = url_input.read().trim().to_string();
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
            // Clear cached fetch timestamp so exercises refresh from the new URL
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
            // Clear cached fetch timestamp so exercises refresh from the new URL
            crate::services::exercise_db::clear_fetch_cache();
        }
    };

    rsx! {
        header { h1 { "ℹ️ Credits" } }
        main { class: "credits",
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
            article {
                h2 { "⚙️ Exercise Database URL" }
                p { "Override the exercise database source. "
                    "Save to trigger a re-download on next reload."
                }
                form {
                    class: "db-url-form",
                    onsubmit: save_url,
                    input {
                        r#type: "url",
                        value: "{url_input}",
                        placeholder: "{crate::utils::EXERCISE_DB_BASE_URL}",
                        oninput: move |evt| url_input.set(evt.value()),
                        class: "form-input db-url-input",
                    }
                    button {
                        r#type: "submit",
                        class: "btn btn--accent",
                        aria_label: "Save",
                        "💾"
                    }
                }
            }
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
                    // a {
                    //     href: "https://github.com/gfauredev/LogOut",
                    //     target: "_blank",
                    //     rel: "noopener noreferrer",
                    //     "on LogOut"
                    // }
                    " including to the "
                    a {
                        href: "https://github.com/gfauredev/free-exercise-db",
                        target: "_blank",
                        "exercise database"
                    }
                    "."
                }
            }
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
                    li { "And many others …" }
                }
            }
        }
        BottomNav { active_tab: ActiveTab::Credits }
    }
}
