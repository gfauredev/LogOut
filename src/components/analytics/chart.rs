use crate::models::analytics::{adapt_metric_unit, Metric};
use dioxus::prelude::*;

/// A single metric–exercise data series:
/// (original slot index, display name, metric, timestamped values).
pub type SeriesData = Vec<(usize, String, Metric, Vec<(f64, f64)>)>;

/// Converts a client X coordinate to an SVG X coordinate using the chart's viewBox.
const SVG_COORD_X: &str = r#"
    const svg = document.querySelector("main.analytics svg");
    const r = svg.getBoundingClientRect();
    const vb = svg.viewBox.baseVal;
    const clientX = await dioxus.recv();
    dioxus.send((clientX - r.left) / r.width * vb.width);
"#;

/// Canonical metric order: [Weight(0), Reps(1), Distance(2), Duration(3)]
const ALL_METRICS: [Metric; 4] = [
    Metric::Weight,
    Metric::Reps,
    Metric::Distance,
    Metric::Duration,
];

/// Update the cursor timestamp from a client-space X coordinate.
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

#[component]
pub fn ChartView(data: SeriesData, colors: Vec<&'static str>) -> Element {
    let cursor_ts: Signal<Option<f64>> = use_signal(|| None);
    let mut is_pointer_down: Signal<bool> = use_signal(|| false);

    // ── Layout constants ─────────────────────────────────────────────────────
    let width = 600.0_f64;
    let chart_height = 342.0_f64;
    let axis_slot = 55.0_f64;
    let top_pad = 30.0_f64;
    let x_gap = 46.0_f64;
    let chart2_bottom_margin = 5.0_f64;

    // ── Metric availability ───────────────────────────────────────────────────
    let metric_has_data: [bool; 4] = ALL_METRICS.map(|m| {
        data.iter()
            .any(|(_, _, dm, pts)| *dm == m && !pts.is_empty())
    });
    let has_chart2 = metric_has_data[2] || metric_has_data[3];
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
        chart1_bottom + 28.0
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
            for (slot_idx , _ , metric , points) in data.iter() {
                {
                    let mi = metric.to_index();
                    if let Some((_, scale, _, _)) = axis_data[mi] {
                        let color = *colors.get(*slot_idx).unwrap_or(&"#ccc");
                        if points.len() >= 2 {
                            #[allow(clippy::cast_precision_loss)]
                            let n = points.len() as f64;
                            let sum_x: f64 = points.iter().map(|(x, _)| x).sum();
                            let sum_y: f64 = points.iter().map(|(_, y)| y * scale).sum();
                            let sum_xx: f64 = points.iter().map(|(x, _)| x * x).sum();
                            let sum_xy: f64 = points.iter().map(|(x, y)| x * y * scale).sum();
                            let denom = n * sum_xx - sum_x * sum_x;
                            let (trend_x1, trend_y1, trend_x2, trend_y2) = if denom.abs()
                                > f64::EPSILON
                            {
                                let slope = (n * sum_xy - sum_x * sum_y) / denom;
                                let intercept = (sum_y - slope * sum_x) / n;
                                let x1 = points.first().map_or(min_x, |(x, _)| *x);
                                let x2 = points.last().map_or(max_x, |(x, _)| *x);
                                (x1, slope * x1 + intercept, x2, slope * x2 + intercept)
                            } else {
                                let mean_y = sum_y / n;
                                (min_x, mean_y, max_x, mean_y)
                            };
                            Some(rsx! {
                                g { key: "series_{slot_idx}",
                                    line {
                                        x1: "{scale_x(trend_x1)}",
                                        y1: "{y_svg(trend_y1, mi)}",
                                        x2: "{scale_x(trend_x2)}",
                                        y2: "{y_svg(trend_y2, mi)}",
                                        stroke: "{color}",
                                        stroke_width: "2",
                                        stroke_dasharray: "8 4",
                                        stroke_linecap: "round",
                                        opacity: "0.7",
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
