use dioxus::prelude::*;
use crate::models::ExerciseLog;
use crate::services::storage;

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
            Metric::Weight => "Weight (lbs)",
            Metric::Reps => "Repetitions",
            Metric::Distance => "Distance (miles)",
            Metric::Duration => "Duration (minutes)",
        }
    }

    fn extract_value(&self, log: &ExerciseLog) -> Option<f64> {
        match self {
            Metric::Weight => log.weight.map(|w| w as f64),
            Metric::Reps => log.reps.map(|r| r as f64),
            Metric::Distance => log.distance.map(|d| d as f64),
            Metric::Duration => log.duration_seconds().map(|d| d as f64 / 60.0),
        }
    }
}

const COLORS: [&str; 8] = [
    "#667eea", // Purple
    "#f093fb", // Pink
    "#4facfe", // Blue
    "#43e97b", // Green
    "#fa709a", // Red
    "#fee140", // Yellow
    "#30cfd0", // Cyan
    "#a8edea", // Light blue
];

#[component]
pub fn AnalyticsPanel() -> Element {
    let mut selected_metric = use_signal(|| Metric::Weight);
    let mut selected_exercises: Signal<Vec<Option<String>>> = use_signal(|| vec![None; 8]);
    
    // Get all sessions to extract exercise data
    let sessions = storage::get_all_sessions();
    
    // Get unique exercise IDs and names
    let available_exercises: Vec<(String, String)> = {
        let mut exercises = std::collections::HashMap::new();
        for session in &sessions {
            for log in &session.exercise_logs {
                exercises.insert(log.exercise_id.clone(), log.exercise_name.clone());
            }
        }
        let mut list: Vec<_> = exercises.into_iter().collect();
        list.sort_by(|a, b| a.1.cmp(&b.1));
        list
    };

    // Collect data points for each selected exercise
    let chart_data: Vec<(String, Vec<(f64, f64)>)> = {
        selected_exercises.read()
            .iter()
            .filter_map(|opt_id| opt_id.as_ref())
            .map(|exercise_id| {
                let mut points = Vec::new();
                let metric = *selected_metric.read();
                
                for session in &sessions {
                    for log in &session.exercise_logs {
                        if &log.exercise_id == exercise_id {
                            if let Some(value) = metric.extract_value(log) {
                                let timestamp = log.start_time as f64;
                                points.push((timestamp, value));
                            }
                        }
                    }
                }
                
                // Sort by timestamp
                points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                
                let exercise_name = available_exercises
                    .iter()
                    .find(|(id, _)| id == exercise_id)
                    .map(|(_, name)| name.clone())
                    .unwrap_or_else(|| exercise_id.clone());
                
                (exercise_name, points)
            })
            .collect()
    };

    rsx! {
        div {
            class: "analytics-panel",
            style: "
                height: 100%;
                display: flex;
                flex-direction: column;
                background: white;
                overflow-y: auto;
            ",
            
            // Header
            div {
                style: "
                    padding: 20px;
                    border-bottom: 2px solid #e0e0e0;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    color: white;
                ",
                h2 {
                    style: "margin: 0 0 10px 0; font-size: 1.5em;",
                    "📊 Analytics"
                }
                p {
                    style: "margin: 0; opacity: 0.9; font-size: 0.9em;",
                    "Track your progress over time"
                }
            }
            
            // Controls
            div {
                style: "padding: 20px; border-bottom: 1px solid #e0e0e0;",
                
                // Metric selection
                div {
                    style: "margin-bottom: 20px;",
                    label {
                        style: "display: block; margin-bottom: 8px; font-weight: bold; color: #333;",
                        "Select Metric"
                    }
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
                        style: "
                            width: 100%;
                            padding: 10px;
                            border: 2px solid #e0e0e0;
                            border-radius: 6px;
                            font-size: 14px;
                            background: white;
                            cursor: pointer;
                        ",
                        option { value: "Weight", "Weight (lbs)" }
                        option { value: "Reps", "Repetitions" }
                        option { value: "Distance", "Distance (miles)" }
                        option { value: "Duration", "Duration (minutes)" }
                    }
                }
                
                // Exercise selections
                div {
                    label {
                        style: "display: block; margin-bottom: 8px; font-weight: bold; color: #333;",
                        "Select Exercises (up to 8)"
                    }
                    
                    for i in 0..8 {
                        {
                            let current_selections = selected_exercises.read().clone();
                            let is_visible = i == 0 || current_selections.get(i - 1).and_then(|x| x.as_ref()).is_some();
                            
                            if is_visible {
                                Some(rsx! {
                                    div {
                                        key: "{i}",
                                        style: "margin-bottom: 10px; display: flex; align-items: center; gap: 10px;",
                                        
                                        div {
                                            style: "
                                                width: 20px;
                                                height: 20px;
                                                border-radius: 50%;
                                                background: {COLORS[i]};
                                                flex-shrink: 0;
                                            ",
                                        }
                                        
                                        select {
                                            value: "{current_selections.get(i).and_then(|x| x.as_ref()).unwrap_or(&String::new())}",
                                            onchange: move |evt| {
                                                let mut selections = selected_exercises.write();
                                                let value = evt.value();
                                                selections[i] = if value.is_empty() {
                                                    None
                                                } else {
                                                    Some(value)
                                                };
                                            },
                                            style: "
                                                flex: 1;
                                                padding: 8px;
                                                border: 2px solid #e0e0e0;
                                                border-radius: 6px;
                                                font-size: 14px;
                                                background: white;
                                                cursor: pointer;
                                            ",
                                            
                                            option { value: "", "-- Select Exercise --" }
                                            for (id, name) in available_exercises.iter() {
                                                option {
                                                    value: "{id}",
                                                    "{name}"
                                                }
                                            }
                                        }
                                    }
                                })
                            } else {
                                None
                            }
                        }
                    }
                }
            }
            
            // Chart
            div {
                style: "flex: 1; padding: 20px; min-height: 300px;",
                
                if chart_data.is_empty() || chart_data.iter().all(|(_, points)| points.is_empty()) {
                    div {
                        style: "
                            display: flex;
                            align-items: center;
                            justify-content: center;
                            height: 300px;
                            color: #999;
                            text-align: center;
                        ",
                        p {
                            "Select exercises to view analytics"
                        }
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
    }
}

#[component]
fn ChartView(data: Vec<(String, Vec<(f64, f64)>)>, metric: Metric, colors: Vec<&'static str>) -> Element {
    // Calculate bounds
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
        
        // Add some padding
        let y_range = max_y - min_y;
        let padding = y_range * 0.1;
        min_y = (min_y - padding).max(0.0);
        max_y = max_y + padding;
        
        (min_x, max_x, min_y, max_y)
    };
    
    let width = 600.0;
    let height = 400.0;
    let padding = 60.0;
    let chart_width = width - 2.0 * padding;
    let chart_height = height - 2.0 * padding;
    
    // Scale functions
    let scale_x = move |x: f64| -> f64 {
        if max_x == min_x {
            padding + chart_width / 2.0
        } else {
            padding + (x - min_x) / (max_x - min_x) * chart_width
        }
    };
    
    let scale_y = move |y: f64| -> f64 {
        if max_y == min_y {
            padding + chart_height / 2.0
        } else {
            padding + chart_height - (y - min_y) / (max_y - min_y) * chart_height
        }
    };
    
    // Format date from timestamp
    let format_date = |timestamp: f64| -> String {
        #[cfg(target_arch = "wasm32")]
        let current_time = js_sys::Date::now() / 1000.0;
        
        #[cfg(not(target_arch = "wasm32"))]
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as f64;
        
        let days_ago = ((current_time - timestamp) / 86400.0) as i64;
        
        if days_ago == 0 {
            "Today".to_string()
        } else if days_ago == 1 {
            "Yesterday".to_string()
        } else {
            format!("{} days ago", days_ago)
        }
    };

    rsx! {
        div {
            style: "width: 100%; overflow-x: auto;",
            
            svg {
                width: "{width}",
                height: "{height}",
                view_box: "0 0 {width} {height}",
                style: "display: block;",
                
                // Y-axis
                line {
                    x1: "{padding}",
                    y1: "{padding}",
                    x2: "{padding}",
                    y2: "{padding + chart_height}",
                    stroke: "#ccc",
                    stroke_width: "2",
                }
                
                // X-axis
                line {
                    x1: "{padding}",
                    y1: "{padding + chart_height}",
                    x2: "{padding + chart_width}",
                    y2: "{padding + chart_height}",
                    stroke: "#ccc",
                    stroke_width: "2",
                }
                
                // Y-axis labels
                for i in 0..5 {
                    {
                        let y_val = min_y + (max_y - min_y) * (i as f64 / 4.0);
                        let y_pos = scale_y(y_val);
                        
                        rsx! {
                            g {
                                key: "ylabel_{i}",
                                line {
                                    x1: "{padding - 5.0}",
                                    y1: "{y_pos}",
                                    x2: "{padding}",
                                    y2: "{y_pos}",
                                    stroke: "#ccc",
                                    stroke_width: "1",
                                }
                                text {
                                    x: "{padding - 10.0}",
                                    y: "{y_pos + 4.0}",
                                    text_anchor: "end",
                                    font_size: "12",
                                    fill: "#666",
                                    "{y_val:.1}"
                                }
                            }
                        }
                    }
                }
                
                // X-axis labels
                {
                    let num_labels = 4.min(data.iter().map(|(_, points)| points.len()).max().unwrap_or(0));
                    rsx! {
                        for i in 0..num_labels {
                            {
                                let x_val = min_x + (max_x - min_x) * (i as f64 / (num_labels - 1).max(1) as f64);
                                let x_pos = scale_x(x_val);
                                
                                rsx! {
                                    g {
                                        key: "xlabel_{i}",
                                        text {
                                            x: "{x_pos}",
                                            y: "{padding + chart_height + 20.0}",
                                            text_anchor: "middle",
                                            font_size: "11",
                                            fill: "#666",
                                            "{format_date(x_val)}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Y-axis label
                text {
                    x: "{padding / 2.0}",
                    y: "{padding + chart_height / 2.0}",
                    text_anchor: "middle",
                    font_size: "14",
                    font_weight: "bold",
                    fill: "#333",
                    transform: "rotate(-90, {padding / 2.0}, {padding + chart_height / 2.0})",
                    "{metric.label()}"
                }
                
                // Plot lines
                for (idx, (exercise_name, points)) in data.iter().enumerate() {
                    {
                        if points.len() >= 2 {
                            let path_data = points
                                .iter()
                                .enumerate()
                                .map(|(i, (x, y))| {
                                    let sx = scale_x(*x);
                                    let sy = scale_y(*y);
                                    if i == 0 {
                                        format!("M {} {}", sx, sy)
                                    } else {
                                        format!("L {} {}", sx, sy)
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join(" ");
                            
                            let color = colors.get(idx).unwrap_or(&"#000");
                            
                            Some(rsx! {
                                g {
                                    key: "line_{idx}",
                                    
                                    // Line
                                    path {
                                        d: "{path_data}",
                                        stroke: "{color}",
                                        stroke_width: "3",
                                        fill: "none",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                    }
                                    
                                    // Points
                                    for (x, y) in points.iter() {
                                        circle {
                                            cx: "{scale_x(*x)}",
                                            cy: "{scale_y(*y)}",
                                            r: "4",
                                            fill: "{color}",
                                            stroke: "white",
                                            stroke_width: "2",
                                        }
                                    }
                                }
                            })
                        } else {
                            None
                        }
                    }
                }
                
                // Legend
                for (idx, (exercise_name, _)) in data.iter().enumerate() {
                    {
                        let y_offset = 20.0 + idx as f64 * 20.0;
                        let color = colors.get(idx).unwrap_or(&"#000");
                        
                        rsx! {
                            g {
                                key: "legend_{idx}",
                                circle {
                                    cx: "{width - 150.0}",
                                    cy: "{y_offset}",
                                    r: "6",
                                    fill: "{color}",
                                }
                                text {
                                    x: "{width - 135.0}",
                                    y: "{y_offset + 4.0}",
                                    font_size: "12",
                                    fill: "#333",
                                    "{exercise_name}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
