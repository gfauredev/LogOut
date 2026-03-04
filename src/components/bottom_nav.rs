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
                to: Route::Exercises {},
                class: if active_tab == ActiveTab::Exercises { "active" } else { "" },
                "📚"
            }
            Link {
                to: Route::Home {},
                class: if active_tab == ActiveTab::Sessions { "active" } else { "" },
                "💪"
            }
            Link {
                to: Route::Analytics {},
                class: if active_tab == ActiveTab::Analytics { "active" } else { "" },
                "📊"
            }
            Link {
                to: Route::Credits {},
                class: if active_tab == ActiveTab::Credits { "active" } else { "" },
                "ℹ️"
            }
        }
    }
}
