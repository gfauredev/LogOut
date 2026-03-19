use crate::Route;
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub enum ActiveTab {
    Exercises,
    Sessions,
    Analytics,
    More,
}

#[component]
pub fn BottomNav(active_tab: ActiveTab) -> Element {
    rsx! {
        nav {
            Link {
                class: if active_tab == ActiveTab::Exercises { "exercises active" } else { "exercises" },
                to: Route::Exercises {},
                "📚"
            }
            Link {
                class: if active_tab == ActiveTab::Sessions { "home active" } else { "home" },
                to: Route::Home {},
                "💪"
            }
            Link {
                class: if active_tab == ActiveTab::Analytics { "analytics active" } else { "analytics" },
                to: Route::Analytics {},
                "📊"
            }
            Link {
                class: if active_tab == ActiveTab::More { "more active" } else { "more" },
                to: Route::More {},
                "⚙️"
            }
        }
    }
}
