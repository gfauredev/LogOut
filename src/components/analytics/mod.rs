use crate::components::{ActiveTab, BottomNav};
use crate::models::analytics::Metric;
use crate::services::storage;
use dioxus::prelude::*;

mod chart;
mod selector;

pub use chart::{ChartView, SeriesData};
pub use selector::MetricSelector;

const COLORS: [&str; 8] = [
    "#3498db", "#e74c3c", "#2ecc71", "#9b59b6", "#e67e22", "#f1c40f", "#16a085", "#e91e63",
];

#[component]
pub fn Analytics() -> Element {
    let selected_pairs: Signal<Vec<(Metric, Option<String>)>> =
        use_signal(|| vec![(Metric::Weight, None); 8]);

    let sessions_resource = use_resource(move || async move {
        let mut all: Vec<crate::models::WorkoutSession> = Vec::new();
        let mut offset = 0usize;
        let page_size = 500usize;
        loop {
            match storage::load_completed_sessions_page(page_size, offset).await {
                Ok(page) => {
                    let fetched = page.len();
                    all.extend(page);
                    if fetched < page_size {
                        break;
                    }
                    offset += fetched;
                }
                Err(e) => {
                    log::error!("Failed to load sessions page for analytics: {e}");
                    break;
                }
            }
        }
        all
    });

    let sessions: Vec<crate::models::WorkoutSession> =
        sessions_resource.read().as_deref().unwrap_or(&[]).to_vec();

    let available_by_metric = use_memo(move || {
        let res = sessions_resource.read();
        let sessions = res.as_deref().unwrap_or(&[]);
        let mut maps: [std::collections::HashMap<String, String>; 4] =
            std::array::from_fn(|_| std::collections::HashMap::new());
        for session in sessions {
            for log in &session.exercise_logs {
                if log.weight_hg.is_some() {
                    maps[0].insert(log.exercise_id.clone(), log.exercise_name.clone());
                }
                if log.reps.is_some() {
                    maps[1].insert(log.exercise_id.clone(), log.exercise_name.clone());
                }
                if log.distance_m.is_some() {
                    maps[2].insert(log.exercise_id.clone(), log.exercise_name.clone());
                }
                maps[3].insert(log.exercise_id.clone(), log.exercise_name.clone());
            }
        }
        maps.map(|m| {
            let mut v: Vec<_> = m.into_iter().collect();
            v.sort_by(|a, b| a.1.cmp(&b.1));
            v
        })
    });

    let chart_data: SeriesData = {
        selected_pairs
            .read()
            .iter()
            .enumerate()
            .filter_map(|(i, (metric, opt_id))| opt_id.as_ref().map(|id| (i, *metric, id.clone())))
            .map(|(i, metric, exercise_id)| {
                let mut points = Vec::new();
                for session in &sessions {
                    for log in &session.exercise_logs {
                        if log.exercise_id == exercise_id {
                            if let Some(value) = metric.extract_value(log) {
                                #[allow(clippy::cast_precision_loss)]
                                points.push((log.start_time as f64, value));
                            }
                        }
                    }
                }
                points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
                let metric_idx = metric.to_index();
                let exercise_name = available_by_metric
                    .read()
                    .get(metric_idx)
                    .and_then(|list| list.iter().find(|(id, _)| id == &exercise_id))
                    .map_or_else(|| exercise_id.clone(), |(_, name)| name.clone());
                (i, exercise_name, metric, points)
            })
            .collect()
    };

    rsx! {
        header {
            h1 { "📊 Analytics" }
            p { "Track your progress over time" }
            label { "Metric–Exercise Pairs (⩽ 8)" }
            for i in 0..8 {
                MetricSelector {
                    i,
                    color: COLORS[i],
                    selected_pairs,
                    available_by_metric,
                }
            }
        }
        main { class: "analytics",
            if chart_data.is_empty()
                || chart_data.iter().all(|(_, _, _, points)| points.is_empty())
            {
                p { "Select exercises to view analytics" }
            } else {
                ChartView { data: chart_data, colors: COLORS.to_vec() }
            }
        }
        BottomNav { active_tab: ActiveTab::Analytics }
    }
}
