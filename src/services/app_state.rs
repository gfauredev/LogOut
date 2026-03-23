//! Dioxus application state: reactive signals and their persistence helpers.
//!
//! This module owns all Dioxus context-based state and the functions that
//! read/write it.  The underlying storage is handled by the sibling
//! [`storage`](super::storage) module; this module just wires the Dioxus
//! reactive primitives to those backends.
use crate::models::{
    get_current_timestamp, Distance, Exercise, ExerciseLog, Weight, WorkoutSession,
};
use crate::ToastSignal;
use dioxus::prelude::*;
#[cfg(target_arch = "wasm32")]
use log::{error, info};
/// Provide the shared workout-session and custom-exercise signals at the top of
/// the component tree.  Call exactly once inside the root `App` component.
pub fn provide_app_state() {
    use_context_provider(|| Signal::new(Vec::<WorkoutSession>::new()));
    use_context_provider(|| Signal::new(Vec::<Exercise>::new()));
    use_context_provider(|| Signal::new(BestsCache::new()));
    use_resource(load_storage_data);
}
/// Obtain the reactive sessions signal from the Dioxus context.
pub fn use_sessions() -> Signal<Vec<WorkoutSession>> {
    consume_context::<Signal<Vec<WorkoutSession>>>()
}
/// Obtain the reactive custom-exercises signal from the Dioxus context.
pub fn use_custom_exercises() -> Signal<Vec<Exercise>> {
    consume_context::<Signal<Vec<Exercise>>>()
}
async fn load_storage_data() {
    #[cfg(target_arch = "wasm32")]
    {
        use super::storage::idb;
        let mut sessions_sig = use_sessions();
        let mut custom_sig = use_custom_exercises();
        let mut toast = consume_context::<ToastSignal>().0;
        match idb::get_all::<WorkoutSession>(idb::STORE_SESSIONS).await {
            Ok(sessions) => {
                let active: Vec<WorkoutSession> =
                    sessions.into_iter().filter(|s| s.is_active()).collect();
                if !active.is_empty() {
                    info!("Loaded {} active sessions from IndexedDB", active.len());
                    sessions_sig.set(active);
                }
            }
            Err(e) => {
                error!("Failed to load sessions from IndexedDB: {e}");
                toast.set(Some(format!("⚠️ Failed to load sessions: {e}")));
            }
        }
        match idb::get_all::<Exercise>(idb::STORE_CUSTOM_EXERCISES).await {
            Ok(custom) if !custom.is_empty() => {
                info!("Loaded {} custom exercises from IndexedDB", custom.len());
                custom_sig.set(custom);
            }
            Err(e) => {
                error!("Failed to load custom exercises from IndexedDB: {e}");
                toast.set(Some(format!("⚠️ Failed to load custom exercises: {e}")));
            }
            _ => {}
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use super::storage::native_storage;
        let mut sessions_sig = use_sessions();
        let mut custom_sig = use_custom_exercises();
        let mut toast = consume_context::<ToastSignal>().0;
        match native_storage::get_all::<WorkoutSession>(native_storage::STORE_SESSIONS) {
            Ok(sessions) => {
                let active: Vec<WorkoutSession> = sessions
                    .into_iter()
                    .filter(WorkoutSession::is_active)
                    .collect();
                if !active.is_empty() {
                    log::info!("Loaded {} active sessions from storage", active.len());
                    sessions_sig.set(active);
                }
            }
            Err(e) => {
                log::error!("Failed to load sessions: {e}");
                toast.set(Some(format!("⚠️ Failed to load sessions: {e}")));
            }
        }
        match native_storage::get_all::<Exercise>(native_storage::STORE_CUSTOM_EXERCISES) {
            Ok(custom) if !custom.is_empty() => {
                log::info!("Loaded {} custom exercises from storage", custom.len());
                custom_sig.set(custom);
            }
            Err(e) => {
                log::error!("Failed to load custom exercises: {e}");
                toast.set(Some(format!("⚠️ Failed to load custom exercises: {e}")));
            }
            _ => {}
        }
    }
}
/// Upsert `session` into the in-memory signal, then persist it to the backend.
///
/// If a session with the same `id` already exists in the signal it is replaced;
/// otherwise the session is appended.  The persistence write is fire-and-forget
/// (errors are surfaced via the toast signal).
pub fn save_session(session: WorkoutSession) {
    let mut sig = use_sessions();
    {
        let mut sessions = sig.write();
        if let Some(pos) = sessions.iter().position(|s| s.id == session.id) {
            sessions[pos] = session.clone();
        } else {
            sessions.push(session.clone());
        }
    }
    if !session.is_active() {
        let mut cache_sig = consume_context::<Signal<BestsCache>>();
        let mut cache = cache_sig.write();
        for log in &session.exercise_logs {
            let entry = cache
                .entry(log.exercise_id.clone())
                .or_insert(ExerciseBests {
                    weight_hg: None,
                    reps: None,
                    distance_m: None,
                    duration: None,
                });
            merge_log_into_bests(entry, log);
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        use super::storage::idb_queue;
        let toast = consume_context::<ToastSignal>().0;
        idb_queue::enqueue(idb_queue::IdbOp::PutSession(session, toast));
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use super::storage::native_queue;
        let toast = consume_context::<ToastSignal>().0;
        native_queue::enqueue(native_queue::NativeOp::PutSession(session, toast));
    }
}
/// Remove the session with `id` from the in-memory signal and from the backend.
pub fn delete_session(id: &str) {
    let mut sig = use_sessions();
    let exercise_ids: Vec<String> = sig
        .read()
        .iter()
        .find(|s| s.id == id)
        .map(|s| {
            s.exercise_logs
                .iter()
                .map(|l| l.exercise_id.clone())
                .collect()
        })
        .unwrap_or_default();
    sig.write().retain(|s| s.id != id);
    if !exercise_ids.is_empty() {
        let mut cache_sig = consume_context::<Signal<BestsCache>>();
        let mut cache = cache_sig.write();
        for ex_id in &exercise_ids {
            cache.remove(ex_id);
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        use super::storage::idb_queue;
        let id = id.to_owned();
        let toast = consume_context::<ToastSignal>().0;
        idb_queue::enqueue(idb_queue::IdbOp::DeleteSession(id, toast));
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use super::storage::native_queue;
        let id = id.to_owned();
        let toast = consume_context::<ToastSignal>().0;
        native_queue::enqueue(native_queue::NativeOp::DeleteSession(id, toast));
    }
}
/// Mark `exercise_id` as the active exercise in the current session.
///
/// Clears the rest timer, sets `current_exercise_id` and
/// `current_exercise_start` on the active session, then persists.
/// No-op when there is no active session.
pub fn begin_exercise_in_session(exercise_id: String, exercise_start: u64) {
    let sig = use_sessions();
    let Some(session) = sig.read().iter().find(|s| s.is_active()).cloned() else {
        return;
    };
    let mut updated = session;
    updated.rest_start_time = None;
    updated.current_exercise_id = Some(exercise_id);
    updated.current_exercise_start = Some(exercise_start);
    save_session(updated);
}
/// Append a completed exercise log to the active session and start the rest timer.
///
/// Pushes `log` onto the session's `exercise_logs`, records the current time
/// as `rest_start_time`, and clears `current_exercise_id` /
/// `current_exercise_start`, then persists.  No-op when there is no active
/// session.
pub fn append_exercise_log(log: ExerciseLog) {
    let sig = use_sessions();
    let Some(session) = sig.read().iter().find(|s| s.is_active()).cloned() else {
        return;
    };
    let mut updated = session;
    updated.exercise_logs.push(log);
    updated.rest_start_time = Some(get_current_timestamp());
    updated.current_exercise_id = None;
    updated.current_exercise_start = None;
    save_session(updated);
}
/// Discard the in-progress exercise in the active session (no log is written).
///
/// Clears `current_exercise_id` and `current_exercise_start` on the active
/// session, then persists.  No-op when there is no active session.
pub fn cancel_exercise_in_session() {
    let sig = use_sessions();
    let Some(session) = sig.read().iter().find(|s| s.is_active()).cloned() else {
        return;
    };
    let mut updated = session;
    updated.current_exercise_id = None;
    updated.current_exercise_start = None;
    save_session(updated);
}
/// Remove `exercise_id` from the pending list and make it the active exercise.
///
/// Only the **first** occurrence of `exercise_id` in `pending_exercise_ids` is
/// removed (FIFO order).  Clears the rest timer, sets `current_exercise_id`
/// and `current_exercise_start`, then persists.  No-op when there is no
/// active session.
pub fn start_pending_exercise_in_session(exercise_id: String, exercise_start: u64) {
    let sig = use_sessions();
    let Some(session) = sig.read().iter().find(|s| s.is_active()).cloned() else {
        return;
    };
    let mut updated = session;
    let mut removed = false;
    updated.pending_exercise_ids.retain(|x| {
        if !removed && x == &exercise_id {
            removed = true;
            false
        } else {
            true
        }
    });
    updated.rest_start_time = None;
    updated.current_exercise_id = Some(exercise_id);
    updated.current_exercise_start = Some(exercise_start);
    save_session(updated);
}
/// Append `exercise` to the custom-exercises signal and persist it to the backend.
pub fn add_custom_exercise(exercise: Exercise) {
    let mut sig = use_custom_exercises();
    sig.write().push(exercise.clone());
    #[cfg(target_arch = "wasm32")]
    {
        use super::storage::idb_queue;
        let toast = consume_context::<ToastSignal>().0;
        idb_queue::enqueue(idb_queue::IdbOp::PutExercise(exercise, toast));
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use super::storage::native_queue;
        let toast = consume_context::<ToastSignal>().0;
        native_queue::enqueue(native_queue::NativeOp::PutExercise(exercise, toast));
    }
}
/// Replace the custom exercise with the same `id` in the signal and persist the update.
pub fn update_custom_exercise(exercise: Exercise) {
    let mut sig = use_custom_exercises();
    {
        let mut exercises = sig.write();
        if let Some(pos) = exercises.iter().position(|e| e.id == exercise.id) {
            exercises[pos] = exercise.clone();
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        use super::storage::idb_queue;
        let toast = consume_context::<ToastSignal>().0;
        idb_queue::enqueue(idb_queue::IdbOp::PutExercise(exercise, toast));
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use super::storage::native_queue;
        let toast = consume_context::<ToastSignal>().0;
        native_queue::enqueue(native_queue::NativeOp::PutExercise(exercise, toast));
    }
}
/// Returns the last completed [`ExerciseLog`] for `exercise_id` across all
/// stored sessions, or `None` if the exercise has never been logged.
///
/// Iterates sessions in reverse chronological order so the most recent log is
/// returned first.  Only complete logs (those with an `end_time`) are considered.
pub fn get_last_exercise_log(exercise_id: &str) -> Option<ExerciseLog> {
    let sessions = use_sessions();
    let sessions = sessions.read();
    find_last_exercise_log(&sessions, exercise_id).cloned()
}
/// Pure search helper used by [`get_last_exercise_log`] and unit tests.
///
/// Searches `sessions` in reverse order for the most recent completed log
/// whose `exercise_id` matches `exercise_id`.
pub(crate) fn find_last_exercise_log<'a>(
    sessions: &'a [WorkoutSession],
    exercise_id: &str,
) -> Option<&'a ExerciseLog> {
    for session in sessions.iter().rev() {
        for log in session.exercise_logs.iter().rev() {
            if log.exercise_id == exercise_id && log.is_complete() {
                return Some(log);
            }
        }
    }
    None
}
/// All-time best (personal record) values for a specific exercise, derived by
/// scanning every completed log across all stored sessions.
#[derive(Clone)]
pub struct ExerciseBests {
    /// Heaviest weight ever lifted for this exercise.
    pub weight_hg: Option<Weight>,
    /// Most repetitions ever performed in a single set.
    pub reps: Option<u32>,
    /// Longest distance ever covered in a single set.
    pub distance_m: Option<Distance>,
    /// Longest set duration ever recorded.
    pub duration: Option<u64>,
}
/// In-memory cache of per-exercise all-time bests, maintained incrementally.
///
/// The cache is populated lazily: on the first call to [`get_exercise_bests`]
/// for a given `exercise_id` the entire sessions signal is scanned once and
/// the result is stored here.  Subsequent calls return the cached value
/// without touching the sessions signal.
///
/// When a session is **completed** (saved with `is_active() == false`) the
/// relevant exercise entries are updated incrementally from the new session
/// only, avoiding a full rescan.
///
/// When a session is **deleted** the cache entries for every exercise in that
/// session are evicted so they are recomputed from scratch on the next access.
type BestsCache = std::collections::HashMap<String, ExerciseBests>;
/// Merge one exercise log's values into an existing best, updating it in place.
fn merge_log_into_bests(bests: &mut ExerciseBests, log: &ExerciseLog) {
    if !log.is_complete() {
        return;
    }
    if let Some(w) = log.weight_hg {
        bests.weight_hg = Some(match bests.weight_hg {
            None => w,
            Some(prev) => {
                if w.0 > prev.0 {
                    w
                } else {
                    prev
                }
            }
        });
    }
    if let Some(r) = log.reps {
        bests.reps = Some(match bests.reps {
            None => r,
            Some(prev) => prev.max(r),
        });
    }
    if let Some(d) = log.distance_m {
        bests.distance_m = Some(match bests.distance_m {
            None => d,
            Some(prev) => {
                if d.0 > prev.0 {
                    d
                } else {
                    prev
                }
            }
        });
    }
    if let Some(dur) = log.duration_seconds() {
        bests.duration = Some(match bests.duration {
            None => dur,
            Some(prev) => prev.max(dur),
        });
    }
}
/// Returns the all-time personal bests for `exercise_id`.
///
/// On the first call for a given exercise the bests are computed by scanning
/// all completed logs in the sessions signal and the result is cached.
/// Subsequent calls return the cached value in O(1).
pub fn get_exercise_bests(exercise_id: &str) -> ExerciseBests {
    let mut cache_sig = consume_context::<Signal<BestsCache>>();
    {
        let cache = cache_sig.read();
        if let Some(cached) = cache.get(exercise_id) {
            return cached.clone();
        }
    }
    let sessions = use_sessions();
    let sessions = sessions.read();
    let mut bests = ExerciseBests {
        weight_hg: None,
        reps: None,
        distance_m: None,
        duration: None,
    };
    for session in sessions.iter() {
        for log in &session.exercise_logs {
            if log.exercise_id == exercise_id {
                merge_log_into_bests(&mut bests, log);
            }
        }
    }
    cache_sig
        .write()
        .insert(exercise_id.to_owned(), bests.clone());
    bests
}
