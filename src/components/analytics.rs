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
        // Raw values are in km; switch to metres when avg < 1 km (keeps 0.0–999.9)
        Metric::Distance => {
            if avg < 1.0 {
                ("m", 1000.0)
            } else {
                ("km", 1.0)
            }
        }
        // Raw values are in minutes; switch to seconds (< 3 min) or hours (≥ 180 min)
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
    "#3498db", // blue  (force / cardio)
    "#e74c3c", // red   (primary muscle / strength)
    "#2ecc71", // green (secondary muscle)
    "#9b59b6", // purple (equipment)
    "#e67e22", // orange (category)
    "#f1c40f", // yellow (level)
    "#16a085", // teal   (static)
    "#e91e63", // pink
];

/// A single metric–exercise data series: (display name, metric, timestamped values).
type SeriesData = Vec<(String, Metric, Vec<(f64, f64)>)>;

#[component]
pub fn Analytics() -> Element {
    // Each slot holds a (metric, optional exercise_id) pair.  A slot becomes
    // visible only once the preceding slot has an exercise selected.
    let mut selected_pairs: Signal<Vec<(Metric, Option<String>)>> =
        use_signal(|| vec![(Metric::Weight, None); 8]);

    let sessions = storage::use_sessions();

    // Pre-compute the sorted list of available exercises for each metric so
    // that we can look them up cheaply while rendering the selectors.
    // Index 0 → Weight, 1 → Reps, 2 → Distance, 3 → Duration
    let available_by_metric = use_memo(move || {
        let sessions = sessions.read();
        let mut maps: [std::collections::HashMap<String, String>; 4] =
            std::array::from_fn(|_| std::collections::HashMap::new());
        for session in sessions.iter() {
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
                // Duration is always trackable
                maps[3].insert(log.exercise_id.clone(), log.exercise_name.clone());
            }
        }
        maps.map(|m| {
            let mut v: Vec<_> = m.into_iter().collect();
            v.sort_by(|a, b| a.1.cmp(&b.1));
            v
        })
    });

    // Collect data points for each fully-specified (metric, exercise) pair.
    // Each entry carries the metric so ChartView can assign per-metric Y-axes.
    let chart_data: SeriesData = {
        let sessions = sessions.read();
        selected_pairs
            .read()
            .iter()
            .filter_map(|(metric, opt_id)| opt_id.as_ref().map(|id| (*metric, id.clone())))
            .map(|(metric, exercise_id)| {
                let mut points = Vec::new();

                for session in sessions.iter() {
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

                // Look up a display name from available_by_metric
                let metric_idx = metric.to_index();
                let exercise_name = available_by_metric
                    .read()
                    .get(metric_idx)
                    .and_then(|list| list.iter().find(|(id, _)| id == &exercise_id))
                    .map_or_else(|| exercise_id.clone(), |(_, name)| name.clone());

                (exercise_name, metric, points)
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
                    // A slot is visible when it is the first slot, or the
                    // preceding slot already has an exercise selected.
                    let is_visible = i == 0
                        || pairs.get(i - 1).is_some_and(|(_, opt_id)| opt_id.is_some());
                    if is_visible {
                        let (current_metric, current_exercise) = pairs[i].clone();
                        let exercises_for_slot =
                            available_by_metric.read()[current_metric.to_index()].clone();
                        Some(rsx! {
                            div {
                                key: "{i}",
                                class: "exercise-selector",
                                div {
                                    style: "background: {COLORS[i]};",
                                }
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
                                        // Clear exercise when the metric changes
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
                                    for (id, name) in exercises_for_slot.iter() {
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
            if chart_data.is_empty() || chart_data.iter().all(|(_, _, points)| points.is_empty()) {
                p { "Select exercises to view analytics" }
            } else {
                ChartView {
                    data: chart_data,
                    colors: COLORS.to_vec(),
                }
            }
        }
        BottomNav { active_tab: ActiveTab::Analytics }
    }
}

#[component]
fn ChartView(data: SeriesData, colors: Vec<&'static str>) -> Element {
    let width = 600.0_f64;
    let height = 400.0_f64;
    let axis_slot = 55.0_f64; // horizontal space reserved per Y-axis
    let top_pad = 30.0_f64; // above chart top (for unit labels)
    let bottom_pad = 28.0_f64; // below chart bottom (for x-axis dates)
    let right_pad = 10.0_f64;

    // ── Collect distinct metrics (only those that have data points) ──────────
    let mut distinct_metrics: Vec<Metric> = Vec::new();
    for (_, metric, pts) in &data {
        if !pts.is_empty() && !distinct_metrics.contains(metric) {
            distinct_metrics.push(*metric);
        }
    }

    #[allow(clippy::cast_precision_loss)]
    let n_axes = distinct_metrics.len().max(1) as f64;
    let left_pad = axis_slot * n_axes;
    let chart_width = (width - left_pad - right_pad).max(50.0);
    let chart_height = height - top_pad - bottom_pad;

    // ── Global X range ───────────────────────────────────────────────────────
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    for (_, _, pts) in &data {
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

    // ── Per-axis precomputed data ────────────────────────────────────────────
    // For each distinct metric: (unit, scale_factor, min_y, max_y, x_pos, color)
    let axes: Vec<(&'static str, f64, f64, f64, f64, &'static str)> = distinct_metrics
        .iter()
        .enumerate()
        .map(|(i, metric)| {
            let raw_y: Vec<f64> = data
                .iter()
                .filter(|(_, m, _)| m == metric)
                .flat_map(|(_, _, pts)| pts.iter().map(|(_, y)| *y))
                .collect();
            let (unit, scale) = adapt_metric_unit(*metric, &raw_y);
            let scaled: Vec<f64> = raw_y.iter().map(|y| y * scale).collect();
            let s_min = scaled.iter().cloned().fold(f64::INFINITY, f64::min);
            let s_max = scaled.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let rng = if (s_max - s_min).abs() < f64::EPSILON {
                1.0
            } else {
                s_max - s_min
            };
            let min_y = (s_min - rng * 0.1).max(0.0);
            let max_y = s_max + rng * 0.1;
            #[allow(clippy::cast_precision_loss)]
            let x_pos = axis_slot * (i as f64 + 1.0);
            // Color matches the first series that uses this metric
            let color_idx = data
                .iter()
                .position(|(_, m, _)| m == metric)
                .unwrap_or(0);
            let color = *colors.get(color_idx).unwrap_or(&"#ccc");
            (unit, scale, min_y, max_y, x_pos, color)
        })
        .collect();

    // Inline helper: SVG y coordinate for a display value on a given axis
    let y_svg = |y_display: f64, axis_idx: usize| -> f64 {
        let (_, _, min_y, max_y, _, _) = axes[axis_idx];
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

    rsx! {
        svg {
            width: "100%",
            height: "auto",
            view_box: "0 0 {width} {height}",

            // X-axis baseline
            line {
                x1: "{left_pad}",
                y1: "{top_pad + chart_height}",
                x2: "{left_pad + chart_width}",
                y2: "{top_pad + chart_height}",
                stroke: "#555",
                stroke_width: "1",
            }

            // ── Y-axes (one per distinct metric) ─────────────────────────────
            for (axis_idx, (unit, _scale, min_y, max_y, x_pos, ax_color)) in axes.iter().enumerate() {
                g { key: "axis_{axis_idx}",
                    // Axis line
                    line {
                        x1: "{x_pos}",
                        y1: "{top_pad}",
                        x2: "{x_pos}",
                        y2: "{top_pad + chart_height}",
                        stroke: "{ax_color}",
                        stroke_width: "1",
                        stroke_opacity: "0.5",
                    }
                    // Unit label on top of the axis (short, no rotation)
                    text {
                        x: "{x_pos}",
                        y: "{top_pad - 6.0}",
                        text_anchor: "middle",
                        font_size: "12",
                        font_weight: "bold",
                        fill: "{ax_color}",
                        "{unit}"
                    }
                    // 5 tick marks + numeric labels
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
                                        stroke: "{ax_color}",
                                        stroke_width: "1",
                                        stroke_opacity: "0.5",
                                    }
                                    text {
                                        x: "{x_pos - 7.0}",
                                        y: "{sy + 4.0}",
                                        text_anchor: "end",
                                        font_size: "11",
                                        fill: "{ax_color}",
                                        "{y_val:.1}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ── X-axis date labels ───────────────────────────────────────────
            {
                let num_labels = 4
                    .min(data.iter().map(|(_, _, p)| p.len()).max().unwrap_or(0))
                    .max(2);
                rsx! {
                    for i in 0..num_labels {
                        {
                            #[allow(clippy::cast_precision_loss)]
                            let x_val =
                                min_x + (max_x - min_x) * (i as f64 / (num_labels - 1) as f64);
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

            // ── Plot lines + dots per series ─────────────────────────────────
            for (idx, (_, metric, points)) in data.iter().enumerate() {
                {
                    let axis_idx_opt =
                        distinct_metrics.iter().position(|m| m == metric);
                    if let Some(axis_idx) = axis_idx_opt {
                        let (_, scale, _, _, _, _) = axes[axis_idx];
                        let color = *colors.get(idx).unwrap_or(&"#ccc");
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
                                g { key: "series_{idx}",
                                    path {
                                        d: "{path_d}",
                                        stroke: "{color}",
                                        stroke_width: "3",
                                        fill: "none",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                    }
                                    for (x, y) in points.iter() {
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
                                    key: "dot_{idx}",
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

            // ── Legend ───────────────────────────────────────────────────────
            for (idx, (exercise_name, metric, _)) in data.iter().enumerate() {
                {
                    if distinct_metrics.contains(metric) {
                        #[allow(clippy::cast_precision_loss)]
                        let ly = top_pad + idx as f64 * 18.0;
                        let color = *colors.get(idx).unwrap_or(&"#ccc");
                        Some(rsx! {
                            g { key: "legend_{idx}",
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
        }
    }
}
