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
                to: Route::ExerciseListPage {},
                class: if active_tab == ActiveTab::Exercises { "active" } else { "" },
                "📚"
            }
            Link {
                to: Route::HomePage {},
                class: if active_tab == ActiveTab::Sessions { "active" } else { "" },
                "💪"
            }
            Link {
                to: Route::AnalyticsPage {},
                class: if active_tab == ActiveTab::Analytics { "active" } else { "" },
                "📊"
            }
            Link {
                to: Route::CreditsPage {},
                class: if active_tab == ActiveTab::Credits { "active" } else { "" },
                "ℹ️"
            }
        }
    }
}
