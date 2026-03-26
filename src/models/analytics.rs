use crate::models::{ExerciseLog, HG_PER_KG, M_PER_KM};

#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum Metric {
    Weight,
    Reps,
    Distance,
    Duration,
}

impl Metric {
    /// Returns the index of this metric in the `available_by_metric` array.
    pub fn to_index(self) -> usize {
        match self {
            Metric::Weight => 0,
            Metric::Reps => 1,
            Metric::Distance => 2,
            Metric::Duration => 3,
        }
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn extract_value(self, log: &ExerciseLog) -> Option<f64> {
        match self {
            Metric::Weight => log.weight_hg.map(|w| f64::from(w.0) / HG_PER_KG),
            Metric::Reps => log.reps.map(f64::from),
            Metric::Distance => log.distance_m.map(|d| f64::from(d.0) / M_PER_KM),
            Metric::Duration => log.duration_seconds().map(|d| d as f64 / 60.0),
        }
    }
}

/// Determine the most adapted display unit for a metric based on the actual
/// data values, so the Y-axis stays in a readable range.
/// Returns `(short_unit, scale_factor)` where `scale_factor` is applied to
/// the raw values to produce display values.
pub fn adapt_metric_unit(metric: Metric, values: &[f64]) -> (&'static str, f64) {
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
                ("m", M_PER_KM)
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
