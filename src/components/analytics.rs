use crate::components::{ActiveTab, AnalyticsPanel, BottomNav};
use dioxus::prelude::*;

#[component]
pub fn AnalyticsPage() -> Element {
    rsx! {
        main { AnalyticsPanel {} }
        BottomNav { active_tab: ActiveTab::Analytics }
    }
}
