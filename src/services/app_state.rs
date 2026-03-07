//! Dioxus application state: reactive signals and their persistence helpers.
//!
//! This module owns all Dioxus context-based state and the functions that
//! read/write it.  The underlying storage is handled by the sibling
//! [`storage`](super::storage) module; this module just wires the Dioxus
//! reactive primitives to those backends.

use crate::models::{get_current_timestamp, Exercise, ExerciseLog, WorkoutSession};
use crate::ToastSignal;
use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
use log::{error, info};

// ──────────────────────────────────────────
// Dioxus context-based state
// ──────────────────────────────────────────

/// Provide the shared workout-session and custom-exercise signals at the top of
/// the component tree.  Call exactly once inside the root `App` component.
pub fn provide_app_state() {
    use_context_provider(|| Signal::new(Vec::<WorkoutSession>::new()));
    use_context_provider(|| Signal::new(Vec::<Exercise>::new()));

    // Load persisted data into the signals via a resource (lifecycle-managed).
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

// ──────────────────────────────────────────
// Initial data load (via use_resource)
// ──────────────────────────────────────────

async fn load_storage_data() {
    // ── Web platform (wasm32 + IndexedDB) ────────────────────────────────────
    #[cfg(target_arch = "wasm32")]
    {
        use super::storage::idb;

        let mut sessions_sig = use_sessions();
        let mut custom_sig = use_custom_exercises();
        let mut toast = consume_context::<ToastSignal>().0;

        match idb::get_all::<WorkoutSession>(idb::STORE_SESSIONS).await {
            Ok(sessions) if !sessions.is_empty() => {
                info!("Loaded {} sessions from IndexedDB", sessions.len());
                sessions_sig.set(sessions);
            }
            Err(e) => {
                error!("Failed to load sessions from IndexedDB: {e}");
                toast.set(Some(format!("⚠️ Failed to load sessions: {e}")));
            }
            _ => {}
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

    // ── Native platform (Android / desktop) ──────────────────────────────────
    #[cfg(not(target_arch = "wasm32"))]
    {
        use super::storage::native_storage;

        let mut sessions_sig = use_sessions();
        let mut custom_sig = use_custom_exercises();
        let mut toast = consume_context::<ToastSignal>().0;

        match native_storage::get_all::<WorkoutSession>(native_storage::STORE_SESSIONS) {
            Ok(sessions) if !sessions.is_empty() => {
                log::info!("Loaded {} sessions from storage", sessions.len());
                sessions_sig.set(sessions);
            }
            Err(e) => {
                log::error!("Failed to load sessions: {e}");
                toast.set(Some(format!("⚠️ Failed to load sessions: {e}")));
            }
            _ => {}
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

// ──────────────────────────────────────────
// Persistence helpers (fire-and-forget)
// ──────────────────────────────────────────

/// Enqueue `session` for persistence without touching the in-memory signal.
///
/// All public mutation functions update the signal in-place and then call
/// this to schedule the write.  Using the queue means writes survive component
/// unmounts (e.g. finishing a session removes `SessionView`).
fn persist_session(session: WorkoutSession) {
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

// ──────────────────────────────────────────
// Public mutation helpers (granular DB writes)
// ──────────────────────────────────────────

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
    persist_session(session);
}

/// Remove the session with `id` from the in-memory signal and from the backend.
pub fn delete_session(id: &str) {
    let mut sig = use_sessions();
    sig.write().retain(|s| s.id != id);

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

// ──────────────────────────────────────────
// Session-mutation helpers (extracted from SessionView)
// ──────────────────────────────────────────

/// Mark `exercise_id` as the currently active exercise in a session.
///
/// Clears any in-progress rest timer, sets the exercise and its start
/// timestamp, and persists the change.  Returns the start timestamp that was
/// recorded so the caller can update its own UI state without a second signal
/// read.
pub fn session_start_exercise(session_id: &str, exercise_id: String) -> u64 {
    let timestamp = get_current_timestamp();
    let mut sig = use_sessions();
    let to_persist = {
        let mut sessions = sig.write();
        sessions.iter_mut().find(|s| s.id == session_id).map(|s| {
            s.rest_start_time = None;
            s.current_exercise_id = Some(exercise_id);
            s.current_exercise_start = Some(timestamp);
            s.clone()
        })
    };
    if let Some(session) = to_persist {
        persist_session(session);
    }
    timestamp
}

/// Remove the first occurrence of `exercise_id` from the pending list, start
/// it as the active exercise, and persist the change.
///
/// Returns the start timestamp so the caller can sync local UI signals.
pub fn session_start_pending_exercise(session_id: &str, exercise_id: &str) -> u64 {
    let timestamp = get_current_timestamp();
    let mut sig = use_sessions();
    let to_persist = {
        let mut sessions = sig.write();
        sessions.iter_mut().find(|s| s.id == session_id).map(|s| {
            // Remove only the first occurrence so repeated exercises are consumed one at a time.
            let mut removed = false;
            s.pending_exercise_ids.retain(|x| {
                if !removed && x == exercise_id {
                    removed = true;
                    false
                } else {
                    true
                }
            });
            s.rest_start_time = None;
            s.current_exercise_id = Some(exercise_id.to_owned());
            s.current_exercise_start = Some(timestamp);
            s.clone()
        })
    };
    if let Some(session) = to_persist {
        persist_session(session);
    }
    timestamp
}

/// Append a completed exercise log to the session, start the rest timer, and
/// clear the active-exercise fields.
///
/// Returns the rest-start timestamp so the caller can start the rest timer UI.
pub fn session_complete_exercise(session_id: &str, log: ExerciseLog) -> u64 {
    let rest_start = get_current_timestamp();
    let mut sig = use_sessions();
    let to_persist = {
        let mut sessions = sig.write();
        sessions.iter_mut().find(|s| s.id == session_id).map(|s| {
            s.exercise_logs.push(log);
            s.rest_start_time = Some(rest_start);
            s.current_exercise_id = None;
            s.current_exercise_start = None;
            s.clone()
        })
    };
    if let Some(session) = to_persist {
        persist_session(session);
    }
    rest_start
}

/// Clear the currently active exercise from the session without recording a log.
pub fn session_cancel_exercise(session_id: &str) {
    let mut sig = use_sessions();
    let to_persist = {
        let mut sessions = sig.write();
        sessions.iter_mut().find(|s| s.id == session_id).map(|s| {
            s.current_exercise_id = None;
            s.current_exercise_start = None;
            s.clone()
        })
    };
    if let Some(session) = to_persist {
        persist_session(session);
    }
}

/// Finish or cancel the session identified by `session_id`.
///
/// * If the session has at least one exercise log it is **finished**: the
///   `end_time` is set to now and the session is persisted.
/// * If the session has no exercise logs it is **cancelled**: it is removed
///   from the signal and deleted from storage.
///
/// Returns `true` when the session was finished, `false` when it was cancelled.
pub fn session_finish(session_id: &str) -> bool {
    let has_exercises = {
        let sig = use_sessions();
        let has = sig
            .read()
            .iter()
            .find(|s| s.id == session_id)
            .map(|s| !s.is_cancelled())
            .unwrap_or(false);
        has
    };

    if has_exercises {
        let end_time = get_current_timestamp();
        let mut sig = use_sessions();
        let to_persist = {
            let mut sessions = sig.write();
            sessions.iter_mut().find(|s| s.id == session_id).map(|s| {
                s.end_time = Some(end_time);
                s.clone()
            })
        };
        if let Some(session) = to_persist {
            persist_session(session);
        }
        true
    } else {
        delete_session(session_id);
        false
    }
}

// ──────────────────────────────────────────
// Read helpers
// ──────────────────────────────────────────

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
