use crate::components::{ActiveTab, BottomNav};
use crate::models::analytics::{AnalyticsMode, Metric};
use crate::models::HG_PER_KG;
use crate::services::{exercise_db, storage};
use dioxus::prelude::*;
use dioxus_i18n::prelude::i18n;
use dioxus_i18n::t;

mod chart;
mod selector;

pub use chart::{ChartView, SeriesData};
pub use selector::MetricSelector;

/// Slot index reserved for the Volume series (beyond the 8 exercise pairs).
const VOLUME_SLOT: usize = 8;

const COLORS: [&str; 9] = [
    "#3498db", "#e74c3c", "#2ecc71", "#9b59b6", "#e67e22", "#f1c40f", "#16a085", "#e91e63",
    "#1abc9c", // volume series (chart 3)
];

#[component]
pub fn Analytics() -> Element {
    let selected_pairs: Signal<Vec<(Metric, Option<String>)>> =
        use_signal(|| vec![(Metric::Weight, None); 8]);
    let mut analytics_mode: Signal<AnalyticsMode> = use_signal(|| AnalyticsMode::Set);
    let mut volume_exercise: Signal<Option<String>> = use_signal(|| None);
    let all_exercises = exercise_db::use_exercises();
    let custom_exercises = storage::use_custom_exercises();
    let lang_str = use_memo(move || i18n().language().to_string());

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
        let all = all_exercises.read();
        let custom = custom_exercises.read();
        let lang = lang_str.read();
        let mut maps: [std::collections::HashMap<String, String>; 5] =
            std::array::from_fn(|_| std::collections::HashMap::new());
        // maps indices mirror Metric::to_index():
        // 0: Weight, 1: Reps, 2: Distance, 3: Duration, 4: Volume
        for session in sessions {
            for log in &session.exercise_logs {
                let name = exercise_db::resolve_exercise(&all, &custom, &log.exercise_id)
                    .map_or_else(
                        || log.exercise_name.clone(),
                        |ex| ex.name_for_lang(&lang).to_owned(),
                    );
                let is_weighted = log.weight_hg.0 > 0;
                if is_weighted {
                    maps[0].insert(log.exercise_id.clone(), name.clone());
                    if log.reps.is_some() {
                        maps[4].insert(log.exercise_id.clone(), name.clone());
                    }
                } else {
                    maps[3].insert(log.exercise_id.clone(), name.clone());
                    if log.reps.is_some() {
                        maps[1].insert(log.exercise_id.clone(), name.clone());
                    }
                    if log.distance_m.is_some() {
                        maps[2].insert(log.exercise_id.clone(), name.clone());
                    }
                }
            }
        }
        maps.map(|m| {
            let mut v: Vec<_> = m.into_iter().collect();
            v.sort_by(|a, b| a.1.cmp(&b.1));
            v
        })
    });

    let mode = *analytics_mode.read();
    let vol_id = volume_exercise.read().clone();

    let chart_data: SeriesData = {
        let mut data: SeriesData = selected_pairs
            .read()
            .iter()
            .enumerate()
            .filter_map(|(i, (metric, opt_id))| opt_id.as_ref().map(|id| (i, *metric, id.clone())))
            .map(|(i, metric, exercise_id)| {
                let mut points = Vec::new();
                // Returns true if a log entry should contribute to this metric's series.
                // Weight: weighted sets only. Reps/Distance/Duration: non-weighted sets.
                let log_matches = |log: &crate::models::ExerciseLog| -> bool {
                    if log.exercise_id != exercise_id {
                        return false;
                    }
                    let w = log.weight_hg.0 > 0;
                    match metric {
                        Metric::Weight => w,
                        Metric::Reps | Metric::Distance | Metric::Duration => !w,
                        Metric::Volume => false,
                    }
                };
                match mode {
                    AnalyticsMode::Set => {
                        for session in &sessions {
                            for log in &session.exercise_logs {
                                if log_matches(log) {
                                    if let Some(value) = metric.extract_value(log) {
                                        #[allow(clippy::cast_precision_loss)]
                                        points.push((log.start_time as f64, value));
                                    }
                                }
                            }
                        }
                    }
                    AnalyticsMode::SessionAverage => {
                        for session in &sessions {
                            let values: Vec<f64> = session
                                .exercise_logs
                                .iter()
                                .filter(|log| log_matches(log))
                                .filter_map(|log| metric.extract_value(log))
                                .collect();
                            if !values.is_empty() {
                                #[allow(clippy::cast_precision_loss)]
                                let avg = values.iter().sum::<f64>() / values.len() as f64;
                                #[allow(clippy::cast_precision_loss)]
                                points.push((session.start_time as f64, avg));
                            }
                        }
                    }
                    AnalyticsMode::SessionTotal => {
                        for session in &sessions {
                            let values: Vec<f64> = session
                                .exercise_logs
                                .iter()
                                .filter(|log| log_matches(log))
                                .filter_map(|log| metric.extract_value(log))
                                .collect();
                            if !values.is_empty() {
                                // Weight: show max per session (sum is not meaningful)
                                // All other metrics: show sum per session
                                #[allow(clippy::cast_precision_loss)]
                                let total = match metric {
                                    Metric::Weight => {
                                        values.iter().copied().fold(f64::NEG_INFINITY, f64::max)
                                    }
                                    Metric::Reps | Metric::Distance | Metric::Duration => {
                                        values.iter().sum()
                                    }
                                    Metric::Volume => 0.0,
                                };
                                #[allow(clippy::cast_precision_loss)]
                                points.push((session.start_time as f64, total));
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
            .collect();

        // Volume series: chart 3, aggregated according to the global mode
        if let Some(exercise_id) = vol_id {
            let mut points = Vec::new();
            match mode {
                AnalyticsMode::Set => {
                    for session in &sessions {
                        for log in &session.exercise_logs {
                            if log.exercise_id == exercise_id && log.weight_hg.0 > 0 {
                                if let Some(reps) = log.reps {
                                    #[allow(clippy::cast_precision_loss)]
                                    let vol =
                                        f64::from(log.weight_hg.0) / HG_PER_KG * f64::from(reps);
                                    #[allow(clippy::cast_precision_loss)]
                                    points.push((log.start_time as f64, vol));
                                }
                            }
                        }
                    }
                }
                AnalyticsMode::SessionAverage => {
                    for session in &sessions {
                        let vols: Vec<f64> = session
                            .exercise_logs
                            .iter()
                            .filter(|log| log.exercise_id == exercise_id && log.weight_hg.0 > 0)
                            .filter_map(|log| {
                                log.reps
                                    .map(|r| f64::from(log.weight_hg.0) / HG_PER_KG * f64::from(r))
                            })
                            .collect();
                        if !vols.is_empty() {
                            #[allow(clippy::cast_precision_loss)]
                            let avg = vols.iter().sum::<f64>() / vols.len() as f64;
                            #[allow(clippy::cast_precision_loss)]
                            points.push((session.start_time as f64, avg));
                        }
                    }
                }
                AnalyticsMode::SessionTotal => {
                    for session in &sessions {
                        let total: f64 = session
                            .exercise_logs
                            .iter()
                            .filter(|log| log.exercise_id == exercise_id && log.weight_hg.0 > 0)
                            .filter_map(|log| {
                                log.reps
                                    .map(|r| f64::from(log.weight_hg.0) / HG_PER_KG * f64::from(r))
                            })
                            .sum();
                        if total > 0.0 {
                            #[allow(clippy::cast_precision_loss)]
                            points.push((session.start_time as f64, total));
                        }
                    }
                }
            }
            points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            let exercise_name = available_by_metric
                .read()
                .get(Metric::Volume.to_index())
                .and_then(|list| list.iter().find(|(id, _)| *id == exercise_id))
                .map_or_else(|| exercise_id.clone(), |(_, name)| name.clone());
            data.push((VOLUME_SLOT, exercise_name, Metric::Volume, points));
        }
        data
    };

    let volume_exercises: Vec<(String, String)> =
        available_by_metric.read()[Metric::Volume.to_index()].clone();
    let vol_is_locked = volume_exercise.read().is_some();

    rsx! {
        header {
            h1 { {t!("analytics-title")} }
            p { {t!("analytics-subtitle")} }
            fieldset { class: "analytics-mode",
                legend { {t!("analytics-mode-label")} }
                div { class: "mode-options",
                    label {
                        input {
                            r#type: "radio",
                            name: "analytics-mode",
                            checked: mode == AnalyticsMode::Set,
                            onchange: move |_| analytics_mode.set(AnalyticsMode::Set),
                        }
                        {t!("analytics-mode-set")}
                    }
                    label {
                        input {
                            r#type: "radio",
                            name: "analytics-mode",
                            checked: mode == AnalyticsMode::SessionAverage,
                            onchange: move |_| analytics_mode.set(AnalyticsMode::SessionAverage),
                        }
                        {t!("analytics-mode-session-avg")}
                    }
                    label {
                        input {
                            r#type: "radio",
                            name: "analytics-mode",
                            checked: mode == AnalyticsMode::SessionTotal,
                            onchange: move |_| analytics_mode.set(AnalyticsMode::SessionTotal),
                        }
                        {t!("analytics-mode-session-total")}
                    }
                }
            }
            div { class: "exercise-selector",
                div { style: "background: {COLORS[VOLUME_SLOT]};" }
                select {
                    value: "{volume_exercise.read().as_deref().unwrap_or(\"\")}",
                    disabled: vol_is_locked,
                    onchange: move |evt| {
                        let value = evt.value();
                        volume_exercise.set(if value.is_empty() { None } else { Some(value) });
                    },
                    option { value: "", {t!("analytics-volume-exercise-label")} }
                    for (id, name) in volume_exercises.iter() {
                        option { value: "{id}", "{name}" }
                    }
                }
                if vol_is_locked {
                    button {
                        class: "back",
                        r#type: "button",
                        title: t!("analytics-remove-series"),
                        onclick: move |_| volume_exercise.set(None),
                        "✕"
                    }
                }
            }
            label { {t!("analytics-pairs-label")} }
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
                p { {t!("analytics-empty")} }
            } else {
                ChartView { data: chart_data, colors: COLORS.to_vec() }
            }
        }
        BottomNav { active_tab: ActiveTab::Analytics }
    }
}
