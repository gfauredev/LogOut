use crate::components::{ActiveTab, BottomNav};
use crate::models::ExerciseLog;
use crate::services::storage;
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Metric {
    Weight,
    Reps,
    Distance,
    Duration,
}

impl Metric {
    fn label(&self) -> &'static str {
        match self {
            Metric::Weight => "Weight (kg)",
            Metric::Reps => "Repetitions",
            Metric::Distance => "Distance (km)",
            Metric::Duration => "Duration (minutes)",
        }
    }

    fn extract_value(&self, log: &ExerciseLog) -> Option<f64> {
        match self {
            Metric::Weight => log.weight_hg.map(|w| w.0 as f64 / 10.0),
            Metric::Reps => log.reps.map(|r| r as f64),
            Metric::Distance => log.distance_m.map(|d| d.0 as f64 / 1000.0),
            Metric::Duration => log.duration_seconds().map(|d| d as f64 / 60.0),
        }
    }
}

const COLORS: [&str; 8] = [
    "#667eea", "#f093fb", "#4facfe", "#43e97b", "#fa709a", "#fee140", "#30cfd0", "#a8edea",
];

#[component]
pub fn AnalyticsPage() -> Element {
    let mut selected_metric = use_signal(|| Metric::Weight);
    let mut selected_exercises: Signal<Vec<Option<String>>> = use_signal(|| vec![None; 8]);

    let sessions = storage::use_sessions();

    // Get unique exercise IDs and names, filtered by selected metric
    let available_exercises = use_memo(move || {
        let sessions = sessions.read();
        let metric = *selected_metric.read();
        let mut exercises = std::collections::HashMap::<String, String>::new();
        for session in sessions.iter() {
            for log in &session.exercise_logs {
                let tracks_metric = match metric {
                    Metric::Weight => log.weight_hg.is_some(),
                    Metric::Reps => log.reps.is_some(),
                    Metric::Distance => log.distance_m.is_some(),
                    Metric::Duration => true,
                };
                if tracks_metric {
                    exercises.insert(log.exercise_id.clone(), log.exercise_name.clone());
                }
            }
        }
        let mut list: Vec<_> = exercises.into_iter().collect();
        list.sort_by(|a, b| a.1.cmp(&b.1));
        list
    });

    // Collect data points for each selected exercise
    let chart_data: Vec<(String, Vec<(f64, f64)>)> = {
        let sessions = sessions.read();
        selected_exercises
            .read()
            .iter()
            .filter_map(|opt_id| opt_id.as_ref())
            .map(|exercise_id| {
                let mut points = Vec::new();
                let metric = *selected_metric.read();

                for session in sessions.iter() {
                    for log in &session.exercise_logs {
                        if &log.exercise_id == exercise_id {
                            if let Some(value) = metric.extract_value(log) {
                                points.push((log.start_time as f64, value));
                            }
                        }
                    }
                }

                points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

                let exercise_name = available_exercises
                    .read()
                    .iter()
                    .find(|(id, _)| id == exercise_id)
                    .map(|(_, name)| name.clone())
                    .unwrap_or_else(|| exercise_id.clone());

                (exercise_name, points)
            })
            .collect()
    };

    rsx! {
        header {
            h1 { "📊 Analytics" }
            p { "Track your progress over time" }
        }
        main { class: "analytics-panel",
            section { class: "controls",
                label { class: "form-label form-label--color", "Select Metric" }
                select {
                    value: "{selected_metric:?}",
                    onchange: move |evt| {
                        selected_metric.set(match evt.value().as_str() {
                            "Weight" => Metric::Weight,
                            "Reps" => Metric::Reps,
                            "Distance" => Metric::Distance,
                            "Duration" => Metric::Duration,
                            _ => Metric::Weight,
                        });
                    },
                    class: "form-select form-select--chart",
                    option { value: "Weight", "Weight (kg)" }
                    option { value: "Reps", "Repetitions" }
                    option { value: "Distance", "Distance (km)" }
                    option { value: "Duration", "Duration (minutes)" }
                }
                label { class: "form-label form-label--color", "Select Exercises (up to 8)" }
                for i in 0..8 {
                    {
                        let current_selections = selected_exercises.read().clone();
                        let is_visible = i == 0 || current_selections.get(i - 1).and_then(|x| x.as_ref()).is_some();
                        if is_visible {
                            Some(rsx! {
                                div {
                                    key: "{i}",
                                    class: "exercise-selector",
                                    div {
                                        class: "color-dot",
                                        style: "background: {COLORS[i]};",
                                    }
                                    select {
                                        value: "{current_selections.get(i).and_then(|x| x.as_ref()).unwrap_or(&String::new())}",
                                        onchange: move |evt| {
                                            let mut selections = selected_exercises.write();
                                            let value = evt.value();
                                            selections[i] = if value.is_empty() { None } else { Some(value) };
                                        },
                                        class: "form-select form-select--chart",
                                        option { value: "", "-- Select Exercise --" }
                                        for (id, name) in available_exercises.read().iter() {
                                            option { value: "{id}", "{name}" }
                                        }
                                    }
                                }
                            })
                        } else { None }
                    }
                }
            }
            section { class: "chart",
                if chart_data.is_empty() || chart_data.iter().all(|(_, points)| points.is_empty()) {
                    div {
                        class: "chart-empty",
                        p { "Select exercises to view analytics" }
                    }
                } else {
                    ChartView {
                        data: chart_data.clone(),
                        metric: *selected_metric.read(),
                        colors: COLORS.to_vec(),
                    }
                }
            }
        }
        BottomNav { active_tab: ActiveTab::Analytics }
    }
}

#[component]
fn ChartView(
    data: Vec<(String, Vec<(f64, f64)>)>,
    metric: Metric,
    colors: Vec<&'static str>,
) -> Element {
    let (min_x, max_x, min_y, max_y) = {
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for (_, points) in &data {
            for (x, y) in points {
                min_x = min_x.min(*x);
                max_x = max_x.max(*x);
                min_y = min_y.min(*y);
                max_y = max_y.max(*y);
            }
        }

        let y_range = max_y - min_y;
        let padding = y_range * 0.1;
        min_y = (min_y - padding).max(0.0);
        max_y += padding;

        (min_x, max_x, min_y, max_y)
    };

    let width = 600.0;
    let height = 400.0;
    let pad = 60.0;
    let chart_width = width - 2.0 * pad;
    let chart_height = height - 2.0 * pad;

    let scale_x = move |x: f64| -> f64 {
        if max_x == min_x {
            pad + chart_width / 2.0
        } else {
            pad + (x - min_x) / (max_x - min_x) * chart_width
        }
    };

    let scale_y = move |y: f64| -> f64 {
        if max_y == min_y {
            pad + chart_height / 2.0
        } else {
            pad + chart_height - (y - min_y) / (max_y - min_y) * chart_height
        }
    };

    let format_date = |timestamp: f64| -> String {
        #[cfg(target_arch = "wasm32")]
        let current_time = js_sys::Date::now() / 1000.0;
        #[cfg(not(target_arch = "wasm32"))]
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as f64;

        let days_ago = ((current_time - timestamp) / 86400.0) as i64;
        match days_ago {
            0 => "Today".to_string(),
            1 => "Yesterday".to_string(),
            n => format!("{} days ago", n),
        }
    };

    rsx! {
        svg {
            width: "100%",
            height: "auto",
            view_box: "0 0 {width} {height}",
            class: "chart-svg",

            // Y-axis
            line { x1: "{pad}", y1: "{pad}", x2: "{pad}", y2: "{pad + chart_height}", stroke: "#ccc", stroke_width: "2" }
            // X-axis
            line { x1: "{pad}", y1: "{pad + chart_height}", x2: "{pad + chart_width}", y2: "{pad + chart_height}", stroke: "#ccc", stroke_width: "2" }

            // Y-axis labels
            for i in 0..5 {
                {
                    let y_val = min_y + (max_y - min_y) * (i as f64 / 4.0);
                    let y_pos = scale_y(y_val);
                    rsx! {
                        g { key: "ylabel_{i}",
                            line { x1: "{pad - 5.0}", y1: "{y_pos}", x2: "{pad}", y2: "{y_pos}", stroke: "#ccc", stroke_width: "1" }
                            text { x: "{pad - 10.0}", y: "{y_pos + 4.0}", text_anchor: "end", font_size: "12", fill: "#ccc", "{y_val:.1}" }
                        }
                    }
                }
            }

            // X-axis labels
            {
                let num_labels = 4.min(data.iter().map(|(_, p)| p.len()).max().unwrap_or(0)).max(2);
                rsx! {
                    for i in 0..num_labels {
                        {
                            let x_val = min_x + (max_x - min_x) * (i as f64 / (num_labels - 1) as f64);
                            let x_pos = scale_x(x_val);
                            rsx! {
                                g { key: "xlabel_{i}",
                                    text { x: "{x_pos}", y: "{pad + chart_height + 20.0}", text_anchor: "middle", font_size: "11", fill: "#ccc", "{format_date(x_val)}" }
                                }
                            }
                        }
                    }
                }
            }

            // Y-axis label
            text {
                x: "{pad / 2.0}", y: "{pad + chart_height / 2.0}",
                text_anchor: "middle", font_size: "14", font_weight: "bold", fill: "#e0e0e0",
                transform: "rotate(-90, {pad / 2.0}, {pad + chart_height / 2.0})",
                "{metric.label()}"
            }

            // Plot lines
            for (idx, (_exercise_name, points)) in data.iter().enumerate() {
                {
                    if points.len() >= 2 {
                        let path_data = points.iter().enumerate()
                            .map(|(i, (x, y))| {
                                let sx = scale_x(*x); let sy = scale_y(*y);
                                if i == 0 { format!("M {} {}", sx, sy) } else { format!("L {} {}", sx, sy) }
                            })
                            .collect::<Vec<_>>().join(" ");
                        let color = colors.get(idx).unwrap_or(&"#ccc");
                        Some(rsx! {
                            g { key: "line_{idx}",
                                path { d: "{path_data}", stroke: "{color}", stroke_width: "3", fill: "none", stroke_linecap: "round", stroke_linejoin: "round" }
                                for (x, y) in points.iter() {
                                    circle { cx: "{scale_x(*x)}", cy: "{scale_y(*y)}", r: "4", fill: "{color}", stroke: "#111", stroke_width: "2" }
                                }
                            }
                        })
                    } else { None }
                }
            }

            // Legend
            for (idx, (exercise_name, _)) in data.iter().enumerate() {
                {
                    let y_offset = 20.0 + idx as f64 * 20.0;
                    let color = colors.get(idx).unwrap_or(&"#ccc");
                    rsx! {
                        g { key: "legend_{idx}",
                            circle { cx: "{width - 150.0}", cy: "{y_offset}", r: "6", fill: "{color}" }
                            text { x: "{width - 135.0}", y: "{y_offset + 4.0}", font_size: "12", fill: "#e0e0e0", "{exercise_name}" }
                        }
                    }
                }
            }
        }
    }
}
