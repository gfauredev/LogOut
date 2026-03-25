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
/// Converts a client X coordinate to an SVG X coordinate using the chart's viewBox.
const SVG_COORD_X: &str = r#"
    const svg = document.querySelector("main.analytics svg");
    const r = svg.getBoundingClientRect();
    const vb = svg.viewBox.baseVal;
    const clientX = await dioxus.recv();
    dioxus.send((clientX - r.left) / r.width * vb.width);
"#;
/// Update the cursor timestamp from a client-space X coordinate.
///
/// Spawns an async JS eval to convert the client coordinate into SVG space,
/// then derives the corresponding timestamp via linear interpolation.
fn update_cursor(
    client_x: f64,
    mut cursor_ts: Signal<Option<f64>>,
    left_pad: f64,
    chart_width: f64,
    min_x: f64,
    max_x: f64,
) {
    spawn(async move {
        let mut ev = dioxus::prelude::document::eval(SVG_COORD_X);
        if ev.send(serde_json::json!(client_x)).is_ok() {
            if let Ok(val) = ev.recv::<serde_json::Value>().await {
                if let Some(svg_x) = val.as_f64() {
                    let frac = (svg_x - left_pad) / chart_width;
                    if (0.0..=1.0).contains(&frac) {
                        cursor_ts.set(Some(min_x + frac * (max_x - min_x)));
                    } else {
                        cursor_ts.set(None);
                    }
                }
            }
        }
    });
}
// Canonical metric order: [Weight(0), Reps(1), Distance(2), Duration(3)]
// Layout mapping:
//   0 Weight  → chart 1, left  Y-axis
//   1 Reps    → chart 1, right Y-axis
//   2 Distance→ chart 2, left  Y-axis
//   3 Duration→ chart 2, right Y-axis
const ALL_METRICS: [Metric; 4] = [
    Metric::Weight,
    Metric::Reps,
    Metric::Distance,
    Metric::Duration,
];
#[component]
fn ChartView(data: SeriesData, colors: Vec<&'static str>) -> Element {
    let cursor_ts: Signal<Option<f64>> = use_signal(|| None);
    let mut is_pointer_down: Signal<bool> = use_signal(|| false);
    // ── Layout constants ─────────────────────────────────────────────────────
    let width = 600.0_f64;
    // Height of each individual chart plot area (same as the original single chart).
    let chart_height = 342.0_f64; // = original 400 − top_pad(30) − bottom_pad(28)
    let axis_slot = 55.0_f64;
    let top_pad = 30.0_f64;
    // Vertical gap between chart 1 bottom (= shared X-axis) and chart 2 top.
    // Accommodates the X-axis tick labels (≈18 px) plus visual breathing room.
    let x_gap = 46.0_f64;
    let chart2_bottom_margin = 5.0_f64;
    // ── Metric availability ───────────────────────────────────────────────────
    let metric_has_data: [bool; 4] = ALL_METRICS.map(|m| {
        data.iter()
            .any(|(_, _, dm, pts)| *dm == m && !pts.is_empty())
    });
    // Chart 2 is shown whenever Distance or Duration have data.
    let has_chart2 = metric_has_data[2] || metric_has_data[3];
    // Right axis exists when Reps (chart 1) or Duration (chart 2) have data.
    let has_right_axis = metric_has_data[1] || metric_has_data[3];
    let right_pad = if has_right_axis { axis_slot } else { 10.0_f64 };
    let left_pad = axis_slot;
    let chart_width = (width - left_pad - right_pad).max(50.0);
    // ── Vertical geometry ─────────────────────────────────────────────────────
    let chart1_top = top_pad;
    let chart1_bottom = top_pad + chart_height;
    let chart2_top = chart1_bottom + x_gap;
    let chart2_bottom = chart2_top + chart_height;
    let total_height = if has_chart2 {
        chart2_bottom + chart2_bottom_margin
    } else {
        chart1_bottom + 28.0 // original bottom_pad keeps single-chart SVG height = 400
    };
    // ── X-axis range (shared across both charts) ──────────────────────────────
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
    // ── Per-metric Y-axis data ────────────────────────────────────────────────
    // `axis_data[i]` = Some((unit, y_scale, display_min, display_max)) when metric i has data.
    #[allow(clippy::cast_precision_loss)]
    let axis_data: [Option<(&'static str, f64, f64, f64)>; 4] = std::array::from_fn(|i| {
        if !metric_has_data[i] {
            return None;
        }
        let metric = ALL_METRICS[i];
        let raw_y: Vec<f64> = data
            .iter()
            .filter(|(_, _, m, _)| *m == metric)
            .flat_map(|(_, _, _, pts)| pts.iter().map(|(_, y)| *y))
            .collect();
        let (unit, scale) = adapt_metric_unit(metric, &raw_y);
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
        Some((unit, scale, min_y, max_y))
    });
    // Convert a display-space Y value for metric index `mi` to SVG Y coordinate.
    let y_svg = |y_display: f64, mi: usize| -> f64 {
        let Some((_, _, min_y, max_y)) = axis_data[mi] else {
            return 0.0;
        };
        let (ct, cb) = if mi < 2 {
            (chart1_top, chart1_bottom)
        } else {
            (chart2_top, chart2_bottom)
        };
        let h = cb - ct;
        if (max_y - min_y).abs() < f64::EPSILON {
            ct + h / 2.0
        } else {
            ct + h - (y_display - min_y) / (max_y - min_y) * h
        }
    };
    let format_date = |ts: f64| -> String {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        crate::utils::format_session_date(ts as u64)
    };
    // ── Cursor tooltip values ─────────────────────────────────────────────────
    let cursor_values: Vec<(usize, String, f64, &'static str)> = if let Some(ts) = *cursor_ts.read()
    {
        data.iter()
            .filter_map(|(slot_idx, name, metric, points)| {
                if points.is_empty() {
                    return None;
                }
                let mi = metric.to_index();
                let (unit, scale, _, _) = axis_data[mi]?;
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
    // ── Interaction geometry ──────────────────────────────────────────────────
    // The transparent rect covers chart 1, the gap (X labels), and chart 2 (if shown).
    let interact_height = if has_chart2 {
        chart2_bottom - chart1_top
    } else {
        chart_height
    };
    let xlabel_y = chart1_bottom + 18.0;
    let num_labels = 4
        .min(data.iter().map(|(_, _, _, p)| p.len()).max().unwrap_or(0))
        .max(2);
    rsx! {
        svg {
            width: "100%",
            height: "auto",
            view_box: "0 0 {width} {total_height}",
            style: "cursor: crosshair; touch-action: pan-y;",
            onmouseup: move |_| {
                is_pointer_down.set(false);
            },
            onmouseleave: move |_| {
                is_pointer_down.set(false);
            },
            // ── X-axis line (shared bottom of chart 1 / top reference of chart 2) ──
            line {
                x1: "{left_pad}",
                y1: "{chart1_bottom}",
                x2: "{left_pad + chart_width}",
                y2: "{chart1_bottom}",
                stroke: "#555",
                stroke_width: "1",
            }
            if has_chart2 {
                line {
                    x1: "{left_pad}",
                    y1: "{chart2_bottom}",
                    x2: "{left_pad + chart_width}",
                    y2: "{chart2_bottom}",
                    stroke: "#555",
                    stroke_width: "1",
                }
            }
            // ── Y-axes ────────────────────────────────────────────────────────
            for i in 0..4_usize {
                if let Some((unit, _, min_y, max_y)) = axis_data[i] {
                    {
                        let is_right = i % 2 == 1;
                        let x_pos = if is_right { left_pad + chart_width } else { left_pad };
                        let (ct, cb) = if i < 2 {
                            (chart1_top, chart1_bottom)
                        } else {
                            (chart2_top, chart2_bottom)
                        };
                        let tick_x1 = if is_right { x_pos } else { x_pos - 4.0 };
                        let tick_x2 = if is_right { x_pos + 4.0 } else { x_pos };
                        let text_x = if is_right { x_pos + 7.0 } else { x_pos - 7.0 };
                        let text_anchor: &str = if is_right { "start" } else { "end" };
                        let unit_anchor: &str = if is_right { "start" } else { "middle" };
                        rsx! {
                            g { key: "axis_{i}",
                                line {
                                    x1: "{x_pos}",
                                    y1: "{ct}",
                                    x2: "{x_pos}",
                                    y2: "{cb}",
                                    stroke: "#555",
                                    stroke_width: "1",
                                    stroke_opacity: "0.7",
                                }
                                text {
                                    x: "{x_pos}",
                                    y: "{ct - 6.0}",
                                    text_anchor: "{unit_anchor}",
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
                                        let sy = y_svg(y_val, i);
                                        rsx! {
                                            g { key: "tick_{tick}",
                                                line {
                                                    x1: "{tick_x1}",
                                                    y1: "{sy}",
                                                    x2: "{tick_x2}",
                                                    y2: "{sy}",
                                                    stroke: "#555",
                                                    stroke_width: "1",
                                                }
                                                text {
                                                    x: "{text_x}",
                                                    y: "{sy + 4.0}",
                                                    text_anchor: "{text_anchor}",
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
                    }
                }
            }
            // ── X-axis labels (drawn once, shared between charts) ─────────────
            for i in 0..num_labels {
                {
                    #[allow(clippy::cast_precision_loss)]
                    let x_val = if num_labels <= 1 {
                        f64::midpoint(min_x, max_x)
                    } else {
                        min_x + (max_x - min_x) * (i as f64 / (num_labels - 1) as f64)
                    };
                    let sx = scale_x(x_val);
                    rsx! {
                        g { key: "xlabel_{i}",
                            text {
                                x: "{sx}",
                                y: "{xlabel_y}",
                                text_anchor: "middle",
                                font_size: "11",
                                fill: "#aaa",
                                "{format_date(x_val)}"
                            }
                        }
                    }
                }
            }
            // ── Data series ───────────────────────────────────────────────────
            for (slot_idx , _ , metric , points) in data.iter() {
                {
                    let mi = metric.to_index();
                    if let Some((_, scale, _, _)) = axis_data[mi] {
                        let color = *colors.get(*slot_idx).unwrap_or(&"#ccc");
                        if points.len() >= 2 {
                            let path_d = points
                                .iter()
                                .enumerate()
                                .map(|(pi, (x, y))| {
                                    let sx = scale_x(*x);
                                    let sy = y_svg(y * scale, mi);
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
                                            cy: "{y_svg(y * scale, mi)}",
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
                                    cy: "{y_svg(y * scale, mi)}",
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
            // ── Cursor lines (one per visible chart, sharing the same timestamp) ──
            if let Some(ts) = *cursor_ts.read() {
                {
                    let cx = scale_x(ts);
                    rsx! {
                        line {
                            x1: "{cx}",
                            y1: "{chart1_top}",
                            x2: "{cx}",
                            y2: "{chart1_bottom}",
                            stroke: "#fff",
                            stroke_width: "1",
                            stroke_opacity: "0.5",
                            stroke_dasharray: "4 3",
                            pointer_events: "none",
                        }
                        if has_chart2 {
                            line {
                                x1: "{cx}",
                                y1: "{chart2_top}",
                                x2: "{cx}",
                                y2: "{chart2_bottom}",
                                stroke: "#fff",
                                stroke_width: "1",
                                stroke_opacity: "0.5",
                                stroke_dasharray: "4 3",
                                pointer_events: "none",
                            }
                        }
                    }
                }
            }
            // ── Interaction overlay ───────────────────────────────────────────
            // Transparent rect covering the full chart area; receives all pointer
            // and touch events so cursor can be slid freely.
            rect {
                x: "{left_pad}",
                y: "{chart1_top}",
                width: "{chart_width}",
                height: "{interact_height}",
                fill: "transparent",
                onclick: move |evt| {
                    let cx = evt.client_coordinates().x;
                    update_cursor(cx, cursor_ts, left_pad, chart_width, min_x, max_x);
                },
                onmousedown: move |evt| {
                    is_pointer_down.set(true);
                    let cx = evt.client_coordinates().x;
                    update_cursor(cx, cursor_ts, left_pad, chart_width, min_x, max_x);
                },
                onmousemove: move |evt| {
                    if *is_pointer_down.read() {
                        let cx = evt.client_coordinates().x;
                        update_cursor(cx, cursor_ts, left_pad, chart_width, min_x, max_x);
                    }
                },
                onmouseup: move |_| {
                    is_pointer_down.set(false);
                },
                ontouchstart: move |evt| {
                    if let Some(touch) = evt.touches().first() {
                        let cx = touch.client_coordinates().x;
                        update_cursor(cx, cursor_ts, left_pad, chart_width, min_x, max_x);
                    }
                },
                ontouchmove: move |evt| {
                    if let Some(touch) = evt.touches().first() {
                        let cx = touch.client_coordinates().x;
                        update_cursor(cx, cursor_ts, left_pad, chart_width, min_x, max_x);
                    }
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
