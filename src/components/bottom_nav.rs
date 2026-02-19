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
            class: "bottom-nav",
            Link {
                to: Route::ExerciseListPage {},
                class: if active_tab == ActiveTab::Exercises { "bottom-nav__tab bottom-nav__tab--active" } else { "bottom-nav__tab" },
                span { class: "bottom-nav__icon", "📚" }
                span { class: "bottom-nav__label", "Exercises" }
            }
            Link {
                to: Route::HomePage {},
                class: if active_tab == ActiveTab::Sessions { "bottom-nav__tab bottom-nav__tab--active" } else { "bottom-nav__tab" },
                span { class: "bottom-nav__icon", "💪" }
                span { class: "bottom-nav__label", "Sessions" }
            }
            Link {
                to: Route::AnalyticsPage {},
                class: if active_tab == ActiveTab::Analytics { "bottom-nav__tab bottom-nav__tab--active" } else { "bottom-nav__tab" },
                span { class: "bottom-nav__icon", "📊" }
                span { class: "bottom-nav__label", "Analytics" }
            }
            Link {
                to: Route::CreditsPage {},
                class: if active_tab == ActiveTab::Credits { "bottom-nav__tab bottom-nav__tab--active" } else { "bottom-nav__tab" },
                span { class: "bottom-nav__icon", "ℹ️" }
                span { class: "bottom-nav__label", "Credits" }
            }
        }
    }
}
