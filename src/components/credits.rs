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
                }
            }
            BottomNav { active_tab: ActiveTab::Credits }
        }
    }
}
