use crate::components::{ActiveTab, BottomNav};
use dioxus::prelude::*;

#[component]
pub fn CreditsPage() -> Element {
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
                        p { "An open source workout logging PWA built with Rust and Dioxus." }
                    }

                    article { class: "credits-card",
                        h3 { "Source Code" }
                        p {
                            a {
                                href: "https://github.com/gfauredev/LogOut",
                                target: "_blank",
                                rel: "noopener noreferrer",
                                class: "credits-link",
                                "github.com/gfauredev/LogOut"
                            }
                        }
                    }

                    article { class: "credits-card",
                        h3 { "License" }
                        p { "This project is open source. See the repository for license details." }
                    }

                    article { class: "credits-card",
                        h3 { "Built With" }
                        ul { class: "credits-list",
                            li {
                                a {
                                    href: "https://dioxuslabs.com",
                                    target: "_blank",
                                    rel: "noopener noreferrer",
                                    class: "credits-link",
                                    "Dioxus"
                                }
                                " — Rust framework for cross-platform apps"
                            }
                            li {
                                a {
                                    href: "https://www.rust-lang.org",
                                    target: "_blank",
                                    rel: "noopener noreferrer",
                                    class: "credits-link",
                                    "Rust"
                                }
                                " — Systems programming language"
                            }
                            li {
                                a {
                                    href: "https://github.com/nicholasgasior/golds-gym-exercises",
                                    target: "_blank",
                                    rel: "noopener noreferrer",
                                    class: "credits-link",
                                    "Golds Gym Exercises DB"
                                }
                                " — Exercise database"
                            }
                            li {
                                a {
                                    href: "https://github.com/nicholasgasior/free-exercise-db",
                                    target: "_blank",
                                    rel: "noopener noreferrer",
                                    class: "credits-link",
                                    "Free Exercise DB"
                                }
                                " — Exercise images"
                            }
                        }
                    }

                    article { class: "credits-card",
                        h3 { "Contributors" }
                        p {
                            "Contributions are welcome! Visit the "
                            a {
                                href: "https://github.com/gfauredev/LogOut",
                                target: "_blank",
                                rel: "noopener noreferrer",
                                class: "credits-link",
                                "GitHub repository"
                            }
                            " to get involved."
                        }
                    }
                }
            }
            BottomNav { active_tab: ActiveTab::Credits }
        }
    }
}
