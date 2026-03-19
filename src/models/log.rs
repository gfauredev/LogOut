use super::enums::{Category, Force};
use super::exercise_type_tag;
use super::units::{Distance, Weight};
use serde::{Deserialize, Serialize};

/// A single completed (or in-progress) exercise within a [`WorkoutSession`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExerciseLog {
    /// Identifier of the exercise performed.
    pub exercise_id: String,
    /// Display name of the exercise (denormalised for rendering without a DB lookup).
    pub exercise_name: String,
    /// Exercise category, used to decide which metrics to display.
    pub category: Category,
    /// Unix timestamp (seconds) when the exercise was started.
    pub start_time: u64,
    /// Unix timestamp when the exercise was finished.  `None` while in progress.
    pub end_time: Option<u64>,
    /// Weight used, stored in hectograms (see [`Weight`]).
    pub weight_hg: Option<Weight>,
    /// Number of repetitions performed.
    pub reps: Option<u32>,
    /// Distance covered, stored in meters (see [`Distance`]).
    pub distance_m: Option<Distance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Force type of the exercise (push / pull / static).
    pub force: Option<Force>,
}

impl ExerciseLog {
    /// Calculate duration in seconds
    pub fn duration_seconds(&self) -> Option<u64> {
        self.end_time.map(|end| end.saturating_sub(self.start_time))
    }

    /// Check if this log is complete (has end time)
    pub fn is_complete(&self) -> bool {
        self.end_time.is_some()
    }

    /// Returns the CSS class and icon for this log's exercise type tag.
    ///
    /// Mirrors [`Exercise::type_tag`] using the denormalised category and force
    /// stored on the log, so no database lookup is required.
    pub fn type_tag(&self) -> (&'static str, &'static str) {
        exercise_type_tag(self.category, self.force)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exercise_log_is_complete() {
        let mut log = ExerciseLog {
            exercise_id: "ex1".into(),
            exercise_name: "Push-up".into(),
            category: Category::Strength,
            start_time: 1000,
            end_time: None,
            weight_hg: None,
            reps: None,
            distance_m: None,
            force: Some(Force::Push),
        };
        assert!(!log.is_complete());
        log.end_time = Some(1060);
        assert!(log.is_complete());
    }

    #[test]
    fn exercise_log_duration_seconds() {
        let log = ExerciseLog {
            exercise_id: "ex1".into(),
            exercise_name: "Push-up".into(),
            category: Category::Strength,
            start_time: 1000,
            end_time: Some(1060),
            weight_hg: None,
            reps: None,
            distance_m: None,
            force: Some(Force::Push),
        };
        assert_eq!(log.duration_seconds(), Some(60));
    }

    #[test]
    fn exercise_log_duration_seconds_none_when_incomplete() {
        let log = ExerciseLog {
            exercise_id: "ex1".into(),
            exercise_name: "Push-up".into(),
            category: Category::Strength,
            start_time: 1000,
            end_time: None,
            weight_hg: None,
            reps: None,
            distance_m: None,
            force: Some(Force::Push),
        };
        assert_eq!(log.duration_seconds(), None);
    }

    #[test]
    fn exercise_log_duration_saturates_on_underflow() {
        let log = ExerciseLog {
            exercise_id: "ex1".into(),
            exercise_name: "Bench".into(),
            category: Category::Strength,
            start_time: 2000,
            end_time: Some(1000), // end before start
            weight_hg: None,
            reps: None,
            distance_m: None,
            force: None,
        };
        assert_eq!(log.duration_seconds(), Some(0));
    }

    #[test]
    fn exercise_log_serde_round_trip_with_all_fields() {
        let log = ExerciseLog {
            exercise_id: "ex1".into(),
            exercise_name: "Squat".into(),
            category: Category::Strength,
            start_time: 1000,
            end_time: Some(1120),
            weight_hg: Some(Weight(1000)),
            reps: Some(5),
            distance_m: Some(Distance(50)),
            force: Some(Force::Push),
        };
        let json = serde_json::to_string(&log).unwrap();
        let back: ExerciseLog = serde_json::from_str(&json).unwrap();
        assert_eq!(back, log);
    }

    #[test]
    fn exercise_log_force_none_is_omitted_in_json() {
        let log = ExerciseLog {
            exercise_id: "ex1".into(),
            exercise_name: "Run".into(),
            category: Category::Cardio,
            start_time: 1000,
            end_time: Some(2000),
            weight_hg: None,
            reps: None,
            distance_m: Some(Distance(500)),
            force: None,
        };
        let json = serde_json::to_string(&log).unwrap();
        assert!(!json.contains("force"));
    }

    #[test]
    fn exercise_log_type_tag_mirrors_exercise() {
        let log = ExerciseLog {
            exercise_id: "bench1".into(),
            exercise_name: "Bench Press".into(),
            category: Category::Strength,
            force: Some(Force::Push),
            start_time: 1000,
            end_time: Some(1060),
            weight_hg: None,
            reps: None,
            distance_m: None,
        };
        assert_eq!(log.type_tag(), ("tag-strength", "💪"));
    }
}
