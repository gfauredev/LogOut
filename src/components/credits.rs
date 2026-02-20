use crate::components::{ActiveTab, BottomNav};
use dioxus::prelude::*;

#[component]
pub fn CreditsPage() -> Element {
    // Current exercise DB URL (defaults to the compile-time constant)
    let mut url_input = use_signal(|| {
        #[cfg(target_arch = "wasm32")]
        {
            crate::utils::get_exercise_db_url()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            crate::utils::EXERCISE_DB_BASE_URL.to_string()
        }
    });

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
    };

    rsx! {
        div { class: "page-container",
            div { class: "page-content",
                section { class: "credits-section",
                    header { class: "credits-header",
                        h1 { class: "page-title", "ℹ️ Credits" }
                    }

                    article { class: "credits-card",
                        h2 { "LogOut" }
                        p { "Turn off your computer, Log your workOut." }
                        p { "A simple, efficient and cross-platform workout "
                            "logging application with "
                            a {
                                href: "https://github.com/yuhonas/free-exercise-db",
                                target: "_blank",
                                class: "credits-link",
                                "800+ exercises"
                            }
                            " built-in, by "
                            a {
                                href: "https://www.guilhemfau.re",
                                target: "_blank",
                                class: "credits-link",
                                "Guilhem Fauré."
                            }
                        }
                    }

                    article { class: "credits-card",
                        h3 { "Open Source & Licences" }
                        p { "This project is open-source under the GPL-3.0, "
                            "and uses other open-source projects. See its "
                            a {
                                href: "https://github.com/gfauredev/LogOut",
                                target: "_blank",
                                rel: "noopener noreferrer",
                                class: "credits-link",
                                "code repository"
                            }
                            " for details. We happily accept contributions, "
                            // a {
                            //     href: "https://github.com/gfauredev/LogOut",
                            //     target: "_blank",
                            //     rel: "noopener noreferrer",
                            //     class: "credits-link",
                            //     "on LogOut"
                            // }
                            " as well as on the "
                                a {
                                    href: "https://github.com/gfauredev/free-exercise-db",
                                    target: "_blank",
                                    class: "credits-link",
                                    "exercise database."
                                }
                        }
                    }

                    article { class: "credits-card",
                        h3 { "Built With" }
                        ul { class: "credits-list",
                            li {
                                a {
                                    href: "https://rust-lang.org",
                                    target: "_blank",
                                    class: "credits-link",
                                    "Rust"
                                }
                                " — Systems programming language"
                            }
                            li {
                                a {
                                    href: "https://dioxuslabs.com",
                                    target: "_blank",
                                    class: "credits-link",
                                    "Dioxus"
                                }
                                " — Rust framework for cross-platform apps"
                            }
                            li {
                                a {
                                    href: "https://github.com/yuhonas/free-exercise-db",
                                    target: "_blank",
                                    class: "credits-link",
                                    "Free Exercise DB"
                                }
                                " — Exercise data and images, by yuhonas"
                            }
                            li { "And many others …" }
                        }
                    }

                    article { class: "credits-card",
                        h3 { "⚙️ Exercise Database URL" }
                        p { class: "credits-card__hint",
                            "Override the exercise database source. "
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
                                class: "btn btn--primary",
                                "Save"
                            }
                        }
                    }
                }
            }
            BottomNav { active_tab: ActiveTab::Credits }
        }
    }
}
