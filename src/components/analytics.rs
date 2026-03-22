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
    /// Returns the index of this metric in the `available_by_metric` array.
    fn to_index(self) -> usize {
        match self {
            Metric::Weight => 0,
            Metric::Reps => 1,
            Metric::Distance => 2,
            Metric::Duration => 3,
        }
    }
    #[allow(clippy::cast_precision_loss)]
    fn extract_value(self, log: &ExerciseLog) -> Option<f64> {
        match self {
            Metric::Weight => log.weight_hg.map(|w| f64::from(w.0) / 10.0),
            Metric::Reps => log.reps.map(f64::from),
            Metric::Distance => log.distance_m.map(|d| f64::from(d.0) / 1000.0),
            Metric::Duration => log.duration_seconds().map(|d| d as f64 / 60.0),
        }
    }
}
/// Determine the most adapted display unit for a metric based on the actual
/// data values, so the Y-axis stays in a readable range.
/// Returns `(short_unit, scale_factor)` where `scale_factor` is applied to
/// the raw values to produce display values.
fn adapt_metric_unit(metric: Metric, values: &[f64]) -> (&'static str, f64) {
    let avg = if values.is_empty() {
        0.0
    } else {
        #[allow(clippy::cast_precision_loss)]
        let len = values.len() as f64;
        values.iter().sum::<f64>() / len
    };
    match metric {
        Metric::Weight => ("kg", 1.0),
        Metric::Reps => ("reps", 1.0),
        Metric::Distance => {
            if avg < 1.0 {
                ("m", 1000.0)
            } else {
                ("km", 1.0)
            }
        }
        Metric::Duration => {
            if avg < 3.0 {
                ("s", 60.0)
            } else if avg < 180.0 {
                ("min", 1.0)
            } else {
                ("h", 1.0 / 60.0)
            }
        }
    }
}
const COLORS: [&str; 8] = [
    "#3498db", "#e74c3c", "#2ecc71", "#9b59b6", "#e67e22", "#f1c40f", "#16a085", "#e91e63",
];
/// A single metric–exercise data series:
/// (original slot index, display name, metric, timestamped values).
type SeriesData = Vec<(usize, String, Metric, Vec<(f64, f64)>)>;
#[component]
pub fn Analytics() -> Element {
    let mut selected_pairs: Signal<Vec<(Metric, Option<String>)>> =
        use_signal(|| vec![(Metric::Weight, None); 8]);
    let sessions_resource = use_resource(move || async move {
        let mut all: Vec<crate::models::WorkoutSession> = Vec::new();
        let mut offset = 0usize;
        let page_size = 500usize;
        loop {
            let page = storage::load_completed_sessions_page(page_size, offset).await;
            let fetched = page.len();
            all.extend(page);
            if fetched < page_size {
                break;
            }
            offset += fetched;
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
                {
                    let pairs = selected_pairs.read().clone();
                    let is_visible = i == 0
                        || pairs.get(i - 1).is_some_and(|(_, opt_id)| opt_id.is_some());
                    if is_visible {
                        let (current_metric, current_exercise) = pairs[i].clone();
                        let exercises_for_slot: Vec<_> = available_by_metric
                            .read()[current_metric.to_index()]
                            .iter()
                            .filter(|(id, _)| {
                                !pairs
                                    .iter()
                                    .enumerate()
                                    .any(|(j, (m, opt_id))| {
                                        j != i && *m == current_metric
                                            && opt_id.as_deref() == Some(id.as_str())
                                    })
                            })
                            .cloned()
                            .collect();
                        Some(rsx! {
                            div { key: "{i}", class: "exercise-selector",
                                div { style: "background: {COLORS[i]};" }
                                select {
                                    value: "{current_metric:?}",
                                    onchange: move |evt| {
                                        let mut pairs = selected_pairs.write();
                                        pairs[i].0 = match evt.value().as_str() {
                                            "Reps" => Metric::Reps,
                                            "Distance" => Metric::Distance,
                                            "Duration" => Metric::Duration,
                                            _ => Metric::Weight,
                                        };
                                        pairs[i].1 = None;
                                    },
                                    option { value: "Weight", "Weight (kg)" }
                                    option { value: "Reps", "Repetitions" }
                                    option { value: "Distance", "Distance" }
                                    option { value: "Duration", "Duration" }
                                }
                                select {
                                    value: "{current_exercise.as_deref().unwrap_or(\"\")}",
                                    onchange: move |evt| {
                                        let mut pairs = selected_pairs.write();
                                        let value = evt.value();
                                        pairs[i].1 = if value.is_empty() { None } else { Some(value) };
                                    },
                                    option { value: "", "-- Select Exercise --" }
                                    for (id , name) in exercises_for_slot.iter() {
                                        option { value: "{id}", "{name}" }
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
const SVG_COORD: &str = r#"
    const svg = document.querySelector("main.analytics svg");
    const pt = svg.createSVGPoint();
    const coords = await dioxus.recv();
    pt.x = coords[0];
    pt.y = coords[1];
    const svgPt = pt.matrixTransform(svg.getScreenCTM().inverse());
    dioxus.send([svgPt.x, svgPt.y]);
"#;
#[component]
fn ChartView(data: SeriesData, colors: Vec<&'static str>) -> Element {
    let mut cursor_ts: Signal<Option<f64>> = use_signal(|| None);
    let width = 600.0_f64;
    let height = 400.0_f64;
    let axis_slot = 55.0_f64;
    let top_pad = 30.0_f64;
    let bottom_pad = 28.0_f64;
    let right_pad = 10.0_f64;
    let mut distinct_metrics: Vec<Metric> = Vec::new();
    for (_, _, metric, pts) in &data {
        if !pts.is_empty() && !distinct_metrics.contains(metric) {
            distinct_metrics.push(*metric);
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let n_axes = distinct_metrics.len().max(1) as f64;
    let left_pad = axis_slot * n_axes;
    let chart_width = (width - left_pad - right_pad).max(50.0);
    let chart_height = height - top_pad - bottom_pad;
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    for (_, _, _, pts) in &data {
        for (x, _) in pts {
            min_x = min_x.min(*x);
            max_x = max_x.max(*x);
        }
    }
    let scale_x = move |x: f64| -> f64 {
        if (max_x - min_x).abs() < f64::EPSILON {
            left_pad + chart_width / 2.0
        } else {
            left_pad + (x - min_x) / (max_x - min_x) * chart_width
        }
    };
    let axes: Vec<(&'static str, f64, f64, f64, f64)> = distinct_metrics
        .iter()
        .enumerate()
        .map(|(i, metric)| {
            let raw_y: Vec<f64> = data
                .iter()
                .filter(|(_, _, m, _)| m == metric)
                .flat_map(|(_, _, _, pts)| pts.iter().map(|(_, y)| *y))
                .collect();
            let (unit, scale) = adapt_metric_unit(*metric, &raw_y);
            let scaled: Vec<f64> = raw_y.iter().map(|y| y * scale).collect();
            let s_min = scaled.iter().copied().fold(f64::INFINITY, f64::min);
            let s_max = scaled.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let rng = if (s_max - s_min).abs() < f64::EPSILON {
                1.0
            } else {
                s_max - s_min
            };
            let min_y = (s_min - rng * 0.1).max(0.0);
            let max_y = s_max + rng * 0.1;
            #[allow(clippy::cast_precision_loss)]
            let x_pos = axis_slot * (i as f64 + 1.0);
            (unit, scale, min_y, max_y, x_pos)
        })
        .collect();
    let y_svg = |y_display: f64, axis_idx: usize| -> f64 {
        let (_, _, min_y, max_y, _) = axes[axis_idx];
        if (max_y - min_y).abs() < f64::EPSILON {
            top_pad + chart_height / 2.0
        } else {
            top_pad + chart_height - (y_display - min_y) / (max_y - min_y) * chart_height
        }
    };
    let format_date = |ts: f64| -> String {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        crate::utils::format_session_date(ts as u64)
    };
    let cursor_values: Vec<(usize, String, f64, &'static str)> = if let Some(ts) = *cursor_ts.read()
    {
        data.iter()
            .filter_map(|(slot_idx, name, metric, points)| {
                if points.is_empty() {
                    return None;
                }
                let axis_idx = distinct_metrics.iter().position(|m| m == metric)?;
                let (unit, scale, _, _, _) = axes[axis_idx];
                let nearest = points.iter().min_by(|(t1, _), (t2, _)| {
                    (t1 - ts)
                        .abs()
                        .partial_cmp(&(t2 - ts).abs())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })?;
                Some((*slot_idx, name.clone(), nearest.1 * scale, unit))
            })
            .collect()
    } else {
        Vec::new()
    };
    rsx! {
        svg {
            width: "100%",
            height: "auto",
            view_box: "0 0 {width} {height}",
            style: "cursor: crosshair;",
            line {
                x1: "{left_pad}",
                y1: "{top_pad + chart_height}",
                x2: "{left_pad + chart_width}",
                y2: "{top_pad + chart_height}",
                stroke: "#555",
                stroke_width: "1",
            }
            for (axis_idx , (unit , _scale , min_y , max_y , x_pos)) in axes.iter().enumerate() {
                g { key: "axis_{axis_idx}",
                    line {
                        x1: "{x_pos}",
                        y1: "{top_pad}",
                        x2: "{x_pos}",
                        y2: "{top_pad + chart_height}",
                        stroke: "#555",
                        stroke_width: "1",
                        stroke_opacity: "0.7",
                    }
                    text {
                        x: "{x_pos}",
                        y: "{top_pad - 6.0}",
                        text_anchor: "middle",
                        font_size: "12",
                        font_weight: "bold",
                        fill: "#aaa",
                        "{unit}"
                    }
                    for tick in 0..5_usize {
                        {
                            #[allow(clippy::cast_precision_loss)]
                            let frac = tick as f64 / 4.0;
                            let y_val = min_y + (max_y - min_y) * frac;
                            let sy = y_svg(y_val, axis_idx);
                            rsx! {
                                g { key: "tick_{tick}",
                                    line {
                                        x1: "{x_pos - 4.0}",
                                        y1: "{sy}",
                                        x2: "{x_pos}",
                                        y2: "{sy}",
                                        stroke: "#555",
                                        stroke_width: "1",
                                    }
                                    text {
                                        x: "{x_pos - 7.0}",
                                        y: "{sy + 4.0}",
                                        text_anchor: "end",
                                        font_size: "11",
                                        fill: "#888",
                                        "{y_val:.1}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            {
                let num_labels = 4
                    .min(data.iter().map(|(_, _, _, p)| p.len()).max().unwrap_or(0))
                    .max(2);
                rsx! {
                    for i in 0..num_labels {
                        {
                            #[allow(clippy::cast_precision_loss)]
                            let x_val = min_x + (max_x - min_x) * (i as f64 / (num_labels - 1) as f64);
                            let sx = scale_x(x_val);
                            rsx! {
                                g { key: "xlabel_{i}",
                                    text {
                                        x: "{sx}",
                                        y: "{top_pad + chart_height + 18.0}",
                                        text_anchor: "middle",
                                        font_size: "11",
                                        fill: "#aaa",
                                        "{format_date(x_val)}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            for (slot_idx , _ , metric , points) in data.iter() {
                {
                    let axis_idx_opt = distinct_metrics.iter().position(|m| m == metric);
                    if let Some(axis_idx) = axis_idx_opt {
                        let (_, scale, _, _, _) = axes[axis_idx];
                        let color = *colors.get(*slot_idx).unwrap_or(&"#ccc");
                        if points.len() >= 2 {
                            let path_d = points
                                .iter()
                                .enumerate()
                                .map(|(pi, (x, y))| {
                                    let sx = scale_x(*x);
                                    let sy = y_svg(y * scale, axis_idx);
                                    if pi == 0 {
                                        format!("M {sx} {sy}")
                                    } else {
                                        format!("L {sx} {sy}")
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join(" ");
                            Some(rsx! {
                                g { key: "series_{slot_idx}",
                                    path {
                                        d: "{path_d}",
                                        stroke: "{color}",
                                        stroke_width: "3",
                                        fill: "none",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                    }
                                    for (x , y) in points.iter() {
                                        circle {
                                            cx: "{scale_x(*x)}",
                                            cy: "{y_svg(y * scale, axis_idx)}",
                                            r: "4",
                                            fill: "{color}",
                                            stroke: "#111",
                                            stroke_width: "2",
                                        }
                                    }
                                }
                            })
                        } else if points.len() == 1 {
                            let (x, y) = points[0];
                            Some(rsx! {
                                circle {
                                    key: "dot_{slot_idx}",
                                    cx: "{scale_x(x)}",
                                    cy: "{y_svg(y * scale, axis_idx)}",
                                    r: "5",
                                    fill: "{color}",
                                    stroke: "#111",
                                    stroke_width: "2",
                                }
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            }
            for (legend_idx , (slot_idx , exercise_name , metric , _)) in data.iter().enumerate() {
                {
                    if distinct_metrics.contains(metric) {
                        #[allow(clippy::cast_precision_loss)]
                        let ly = top_pad + legend_idx as f64 * 18.0;
                        let color = *colors.get(*slot_idx).unwrap_or(&"#ccc");
                        Some(rsx! {
                            g { key: "legend_{slot_idx}",
                                circle {
                                    cx: "{width - 140.0}",
                                    cy: "{ly}",
                                    r: "5",
                                    fill: "{color}",
                                }
                                text {
                                    x: "{width - 127.0}",
                                    y: "{ly + 4.0}",
                                    font_size: "11",
                                    fill: "#e0e0e0",
                                    "{exercise_name}"
                                }
                            }
                        })
                    } else {
                        None
                    }
                }
            }
            if let Some(ts) = *cursor_ts.read() {
                {
                    let cx = scale_x(ts);
                    rsx! {
                        line {
                            x1: "{cx}",
                            y1: "{top_pad}",
                            x2: "{cx}",
                            y2: "{top_pad + chart_height}",
                            stroke: "#fff",
                            stroke_width: "1",
                            stroke_opacity: "0.5",
                            stroke_dasharray: "4 3",
                            pointer_events: "none",
                        }
                    }
                }
            }
            rect {
                x: "{left_pad}",
                y: "{top_pad}",
                width: "{chart_width}",
                height: "{chart_height}",
                fill: "transparent",
                onclick: move |evt| {
                    let client_x = evt.client_coordinates().x;
                    let client_y = evt.client_coordinates().y;
                    let lp = left_pad;
                    let cw = chart_width;
                    let mx = min_x;
                    let dx = max_x - min_x;
                    spawn(async move {
                        let mut ev = dioxus::prelude::document::eval(SVG_COORD);
                        if ev.send(serde_json::json!([client_x, client_y])).is_ok() {
                            if let Ok(result) = ev.recv::<serde_json::Value>().await {
                                if let Some(arr) = result.as_array() {
                                    if let Some(svg_x) = arr
                                        .first()
                                        .and_then(serde_json::Value::as_f64)
                                    {
                                        let frac = (svg_x - lp) / cw;
                                        if (0.0..=1.0).contains(&frac) {
                                            cursor_ts.set(Some(mx + frac * dx));
                                        } else {
                                            cursor_ts.set(None);
                                        }
                                    }
                                }
                            }
                        }
                    });
                },
            }
        }
        if !cursor_values.is_empty() {
            div { class: "cursor-values",
                for (slot_idx , name , value , unit) in cursor_values.iter() {
                    div { class: "cursor-value-row",
                        span {
                            class: "cursor-swatch",
                            style: "background:{colors.get(*slot_idx).unwrap_or(&\"#ccc\")};",
                        }
                        span { class: "cursor-name", "{name}" }
                        span { class: "cursor-val", "{value:.1} {unit}" }
                    }
                }
            }
        }
    }
}
