use crate::Route;
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub enum ActiveTab {
    Exercises,
    Sessions,
    Analytics,
    Credits,
}

#[component]
pub fn BottomNav(active_tab: ActiveTab) -> Element {
    rsx! {
        nav {
            Link {
                class: if active_tab == ActiveTab::Exercises { "active" } else { "" },
                to: Route::Exercises {},
                "📚"
            }
            Link {
                class: if active_tab == ActiveTab::Sessions { "active" } else { "" },
                to: Route::Home {},
                "💪"
            }
            Link {
                class: if active_tab == ActiveTab::Analytics { "active" } else { "" },
                to: Route::Analytics {},
                "📊"
            }
            Link {
                class: if active_tab == ActiveTab::Credits { "active" } else { "" },
                to: Route::Credits {},
                "ℹ️"
            }
        }
    }
}
