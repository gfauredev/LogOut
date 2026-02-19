use crate::components::{ActiveTab, AnalyticsPanel, BottomNav};
use dioxus::prelude::*;

#[component]
pub fn AnalyticsPage() -> Element {
    rsx! {
        div { class: "page-container",
            div { class: "page-content",
                AnalyticsPanel {}
            }
            BottomNav { active_tab: ActiveTab::Analytics }
        }
    }
}
