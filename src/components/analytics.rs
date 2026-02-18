use dioxus::prelude::*;
use crate::components::{AnalyticsPanel, BottomNav, ActiveTab};

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
