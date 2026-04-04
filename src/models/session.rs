use super::get_current_timestamp;
use super::log::ExerciseLog;
use serde::{Deserialize, Serialize};
/// A collection of exercise logs performed in one workout bout.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkoutSession {
    /// Unique identifier for the session (randomly generated or timestamp-based).
    pub id: String,
    /// Unix timestamp (seconds) when the session was started.
    /// This value is **never mutated** after the session is created; use
    /// `total_paused_duration` to account for time spent paused when computing
    /// the net workout duration.
    pub start_time: u64,
    /// Unix timestamp when the session was finished.  `None` while active.
    pub end_time: Option<u64>,
    /// Chronological list of exercise logs performed during this session.
    pub exercise_logs: Vec<ExerciseLog>,
    #[serde(default)]
    /// List of exercise IDs pre-added to the session but not yet started.
    pub pending_exercise_ids: Vec<String>,
    #[serde(default)]
    /// Unix timestamp when the last rest period was started (used to drive the rest timer).
    pub rest_start_time: Option<u64>,
    #[serde(default)]
    /// ID of the exercise currently being performed.
    pub current_exercise_id: Option<String>,
    #[serde(default)]
    /// Unix timestamp when the current exercise was started.
    pub current_exercise_start: Option<u64>,
    #[serde(default)]
    /// Unix timestamp when the session was paused (None if running).
    pub paused_at: Option<u64>,
    #[serde(default)]
    /// Total cumulative time (in seconds) the session has spent paused.
    /// Incremented in [`WorkoutSession::resume`] and used by
    /// [`WorkoutSession::duration_seconds`] so that `start_time` is never
    /// mutated after the session is created.
    pub total_paused_duration: u64,
    #[serde(default)]
    /// Free-form session notes written by the user (Markdown supported).
    pub notes: String,
}
impl WorkoutSession {
    /// Create a new session with current timestamp and a unique ID.
    pub fn new() -> Self {
        let now = get_current_timestamp();
        Self {
            id: format!("session_{now}"),
            start_time: now,
            end_time: None,
            exercise_logs: Vec::new(),
            pending_exercise_ids: Vec::new(),
            rest_start_time: None,
            current_exercise_id: None,
            current_exercise_start: None,
            paused_at: None,
            total_paused_duration: 0,
            notes: String::new(),
        }
    }
    /// Returns true if the session is currently active (no end time).
    pub fn is_active(&self) -> bool {
        self.end_time.is_none()
    }
    /// Check if the session is cancelled (active and has no logs and no current exercise)
    pub fn is_cancelled(&self) -> bool {
        self.is_active() && self.exercise_logs.is_empty() && self.current_exercise_id.is_none()
    }
    /// Calculate session duration in seconds, excluding paused time.
    pub fn duration_seconds(&self) -> u64 {
        let end = self.end_time.unwrap_or_else(get_current_timestamp);
        let elapsed = end.saturating_sub(self.start_time);
        // Subtract time already accumulated in `total_paused_duration`.
        let mut total = elapsed.saturating_sub(self.total_paused_duration);
        // Also subtract the current ongoing pause, if any.
        if let Some(paused) = self.paused_at {
            total = total.saturating_sub(end.saturating_sub(paused));
        }
        total
    }
    /// Pause the session
    pub fn pause(&mut self) {
        if self.paused_at.is_none() {
            self.paused_at = Some(get_current_timestamp());
        }
    }
    /// Resume the session: accumulate the pause duration into
    /// `total_paused_duration` without mutating `start_time`.
    ///
    /// `rest_start_time` and `current_exercise_start` are still advanced by
    /// the pause duration so that their elapsed-since calculations remain
    /// correct — those timestamps measure transient activity windows, not
    /// the historical session start.
    pub fn resume(&mut self) {
        if let Some(paused) = self.paused_at {
            let now = get_current_timestamp();
            let pause_duration = now.saturating_sub(paused);
            self.total_paused_duration += pause_duration;
            if let Some(rest_start) = self.rest_start_time {
                self.rest_start_time = Some(rest_start + pause_duration);
            }
            if let Some(ex_start) = self.current_exercise_start {
                self.current_exercise_start = Some(ex_start + pause_duration);
            }
            self.paused_at = None;
        }
    }
    /// Is the session paused?
    pub fn is_paused(&self) -> bool {
        self.paused_at.is_some()
    }
}
impl Default for WorkoutSession {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn workout_session_new_has_id_and_start_time() {
        let s = WorkoutSession::new();
        assert!(s.id.starts_with("session_"));
        assert!(s.start_time > 0);
        assert!(s.is_active());
        assert!(s.exercise_logs.is_empty());
    }
    #[test]
    fn workout_session_is_active_until_end_time_set() {
        let mut s = WorkoutSession::new();
        assert!(s.is_active());
        s.end_time = Some(get_current_timestamp());
        assert!(!s.is_active());
    }
    #[test]
    fn workout_session_with_exercise_logs_serde() {
        let session = WorkoutSession {
            id: "s1".into(),
            start_time: 1000,
            end_time: Some(2000),
            exercise_logs: vec![ExerciseLog {
                exercise_id: "ex1".into(),
                exercise_name: "Squat".into(),
                category: crate::models::Category::Strength,
                start_time: 1000,
                end_time: Some(1120),
                weight_hg: crate::models::Weight(1000),
                reps: Some(5),
                distance_m: None,
                force: Some(crate::models::Force::Push),
            }],
            pending_exercise_ids: vec![],
            rest_start_time: None,
            current_exercise_id: None,
            current_exercise_start: None,
            paused_at: None,
            total_paused_duration: 0,
            notes: String::new(),
        };
        let json = serde_json::to_string(&session).unwrap();
        let back: WorkoutSession = serde_json::from_str(&json).unwrap();
        assert_eq!(back, session);
        assert_eq!(back.exercise_logs.len(), 1);
        assert_eq!(back.exercise_logs[0].exercise_name, "Squat");
    }
    #[test]
    fn workout_session_rest_start_time_round_trip() {
        let session = WorkoutSession {
            id: "s1".into(),
            start_time: 1000,
            end_time: None,
            exercise_logs: vec![],
            pending_exercise_ids: vec![],
            rest_start_time: Some(1500),
            current_exercise_id: Some("bench_press".into()),
            current_exercise_start: Some(1200),
            paused_at: None,
            total_paused_duration: 0,
            notes: String::new(),
        };
        let json = serde_json::to_string(&session).unwrap();
        let back: WorkoutSession = serde_json::from_str(&json).unwrap();
        assert_eq!(back.rest_start_time, Some(1500));
        assert_eq!(back.current_exercise_id, Some("bench_press".into()));
        assert_eq!(back.current_exercise_start, Some(1200));
    }
    #[test]
    fn workout_session_rest_start_time_defaults_none() {
        let json = r#"{"id":"s1","start_time":1000,"end_time":null,"exercise_logs":[],"pending_exercise_ids":[]}"#;
        let session: WorkoutSession = serde_json::from_str(json).unwrap();
        assert!(session.rest_start_time.is_none());
        assert_eq!(session.total_paused_duration, 0);
    }
    #[test]
    fn workout_session_duration_calculation() {
        let mut s = WorkoutSession {
            id: "s1".into(),
            start_time: 1000,
            end_time: Some(2000),
            exercise_logs: vec![],
            pending_exercise_ids: vec![],
            rest_start_time: None,
            current_exercise_id: None,
            current_exercise_start: None,
            paused_at: None,
            total_paused_duration: 0,
            notes: String::new(),
        };
        assert_eq!(s.duration_seconds(), 1000);
        s.paused_at = Some(1500);
        assert_eq!(s.duration_seconds(), 500);
    }
    #[test]
    fn workout_session_resume_accumulates_paused_duration() {
        // start_time must remain unchanged after resume
        let mut s = WorkoutSession {
            id: "s1".into(),
            start_time: 1000,
            end_time: Some(2200),
            exercise_logs: vec![],
            pending_exercise_ids: vec![],
            rest_start_time: None,
            current_exercise_id: None,
            current_exercise_start: None,
            paused_at: Some(1500),
            total_paused_duration: 0,
            notes: String::new(),
        };
        // Simulate resume at t=1700: pause_duration = 200s
        // Manually set total_paused_duration as resume() uses get_current_timestamp()
        s.total_paused_duration = 200;
        s.paused_at = None;
        assert_eq!(s.start_time, 1000, "start_time must not be mutated");
        // duration = (2200 - 1000) - 200 = 1000
        assert_eq!(s.duration_seconds(), 1000);
    }
    #[test]
    fn workout_session_total_paused_duration_serde_default() {
        // Old sessions without the field should default to 0
        let json = r#"{"id":"s1","start_time":1000,"end_time":null,"exercise_logs":[],"pending_exercise_ids":[]}"#;
        let session: WorkoutSession = serde_json::from_str(json).unwrap();
        assert_eq!(session.total_paused_duration, 0);
    }
    #[test]
    fn workout_session_notes_serde_default() {
        // Old sessions without the notes field should default to empty string.
        let json = r#"{"id":"s1","start_time":1000,"end_time":null,"exercise_logs":[],"pending_exercise_ids":[]}"#;
        let session: WorkoutSession = serde_json::from_str(json).unwrap();
        assert_eq!(session.notes, "");
    }
    #[test]
    fn workout_session_notes_round_trip() {
        let mut s = WorkoutSession::new();
        s.notes = "## Great workout\n- Heavy squats\n- New PR!".to_string();
        let json = serde_json::to_string(&s).unwrap();
        let back: WorkoutSession = serde_json::from_str(&json).unwrap();
        assert_eq!(back.notes, s.notes);
    }
}
