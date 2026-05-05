use crate::models::{ExerciseLog, HG_PER_KG, M_PER_KM};
/// Minimum average duration (in minutes) below which values are displayed in seconds.
const DURATION_MINS_SECS_THRESHOLD: f64 = 3.0;
/// Minimum average duration (in minutes) below which values are displayed in minutes rather than hours.
const DURATION_HOURS_MINS_THRESHOLD: f64 = 180.0;

/// Global aggregation mode for the Analytics view.
/// Applies uniformly to all exercise series and to the Volume chart.
///
/// Note: in [`AnalyticsMode::SessionTotal`] mode, [`crate::models::analytics::Metric::Weight`]
/// shows the **maximum** weight per session rather than the sum, because summing weights
/// across sets is not a meaningful fitness metric.
#[derive(Clone, Copy, PartialEq, Debug, Default, serde::Serialize, serde::Deserialize)]
pub enum AnalyticsMode {
    /// One data point per set (timestamp = set start time).
    #[default]
    Set,
    /// One data point per session: the mean value across all sets.
    SessionAverage,
    /// One data point per session: the total (sum, or max for Weight) across all sets.
    SessionTotal,
}

#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum Metric {
    Weight,
    Reps,
    Distance,
    Duration,
    /// Volume = weight × reps, aggregated across sets; always shown on chart 3.
    Volume,
}

impl Metric {
    /// Returns the index of this metric in the `available_by_metric` array.
    /// Canonical order: Weight(0), Reps(1), Distance(2), Duration(3), Volume(4)
    pub fn to_index(self) -> usize {
        match self {
            Metric::Weight => 0,
            Metric::Reps => 1,
            Metric::Distance => 2,
            Metric::Duration => 3,
            Metric::Volume => 4,
        }
    }

    /// Extract a per-set value from a single exercise log.
    ///
    /// Returns `None` for [`Metric::Volume`], which must be computed from
    /// weight and reps together and is always session-aggregated.
    #[allow(clippy::cast_precision_loss)]
    pub fn extract_value(self, log: &ExerciseLog) -> Option<f64> {
        match self {
            Metric::Weight => (log.weight_hg.0 > 0).then(|| f64::from(log.weight_hg.0) / HG_PER_KG),
            Metric::Reps => log.reps.map(f64::from),
            Metric::Distance => log.distance_m.map(|d| f64::from(d.0) / M_PER_KM),
            Metric::Duration => log.duration_seconds().map(|d| d as f64 / 60.0),
            Metric::Volume => None,
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
        {
            values.iter().sum::<f64>() / (values.len() as f64)
        }
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
            if avg < DURATION_MINS_SECS_THRESHOLD {
                ("s", 60.0)
            } else if avg < DURATION_HOURS_MINS_THRESHOLD {
                ("min", 1.0)
            } else {
                ("h", 1.0 / 60.0)
            }
        }
        Metric::Volume => ("kg·reps", 1.0),
    }
}
