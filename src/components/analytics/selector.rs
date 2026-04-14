use crate::models::analytics::Metric;
use dioxus::prelude::*;
use dioxus_i18n::t;

#[component]
pub fn MetricSelector(
    i: usize,
    color: &'static str,
    selected_pairs: Signal<Vec<(Metric, Option<String>)>>,
    available_by_metric: Memo<[Vec<(String, String)>; 4]>,
) -> Element {
    let pairs = selected_pairs.read().clone();
    let is_visible = i == 0 || pairs.get(i - 1).is_some_and(|(_, opt_id)| opt_id.is_some());

    if !is_visible {
        return rsx! {};
    }

    let (current_metric, current_exercise) = pairs[i].clone();
    let is_locked = current_exercise.is_some();
    let exercises_for_slot: Vec<_> = available_by_metric.read()[current_metric.to_index()]
        .iter()
        .filter(|(id, _)| {
            !pairs.iter().enumerate().any(|(j, (m, opt_id))| {
                j != i && *m == current_metric && opt_id.as_deref() == Some(id.as_str())
            })
        })
        .cloned()
        .collect();

    rsx! {
        div { key: "{i}", class: "exercise-selector",
            div { style: "background: {color};" }
            select {
                value: "{current_metric:?}",
                disabled: is_locked,
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
                option { value: "Weight", {t!("analytics-metric-weight")} }
                option { value: "Reps", {t!("analytics-metric-reps")} }
                option { value: "Distance", {t!("analytics-metric-distance")} }
                option { value: "Duration", {t!("analytics-metric-duration")} }
            }
            select {
                value: "{current_exercise.as_deref().unwrap_or(\"\")}",
                disabled: is_locked,
                onchange: move |evt| {
                    let mut pairs = selected_pairs.write();
                    let value = evt.value();
                    pairs[i].1 = if value.is_empty() { None } else { Some(value) };
                },
                option { value: "", {t!("analytics-select-exercise")} }
                for (id, name) in exercises_for_slot.iter() {
                    option { value: "{id}", "{name}" }
                }
            }
            if is_locked {
                button {
                    class: "back",
                    r#type: "button",
                    title: t!("analytics-remove-series"),
                    onclick: move |_| {
                        let mut pairs = selected_pairs.write();
                        pairs[i] = (Metric::Weight, None);
                        for j in i..7 {
                            if pairs[j].1.is_none() && pairs[j + 1].1.is_some() {
                                pairs[j] = pairs[j + 1].clone();
                                pairs[j + 1] = (Metric::Weight, None);
                            } else {
                                break;
                            }
                        }
                    },
                    "✕"
                }
            }
        }
    }
}
