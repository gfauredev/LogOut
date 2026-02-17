use dioxus::prelude::*;
use crate::Route;

#[component]
pub fn HomePage() -> Element {
    rsx! {
        div {
            class: "container",
            
            header {
                class: "home-header",
                h1 { class: "home-title", "💪 LogOut" }
                p { class: "home-tagline", "Turn off your computer, Log your workOut" }
            }
            
            nav {
                class: "home-nav",
                
                Link {
                    to: Route::ActiveSessionPage {},
                    class: "nav-link nav-link--primary",
                    "🏋️ Start New Workout"
                }
                
                Link {
                    to: Route::ExerciseListPage {},
                    class: "nav-link nav-link--secondary",
                    "📚 Browse Exercises"
                }
            }
            
            footer {
                class: "home-footer",
                p {
                    "Exercise database from "
                    a {
                        href: "https://github.com/yuhonas/free-exercise-db",
                        target: "_blank",
                        "free-exercise-db"
                    }
                }
            }
        }
    }
}
