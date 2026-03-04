//! Dioxus application state: reactive signals and their persistence helpers.
//!
//! This module owns all Dioxus context-based state and the functions that
//! read/write it.  The underlying storage is handled by the sibling
//! [`storage`](super::storage) module; this module just wires the Dioxus
//! reactive primitives to those backends.

use crate::models::{Exercise, ExerciseLog, WorkoutSession};
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

    // Use wasm_bindgen_futures::spawn_local instead of Dioxus spawn so that the
    // IndexedDB write is not cancelled when the calling component unmounts
    // (e.g. when finishing a session causes SessionView to be removed).
    // Writes go through the async queue to prevent concurrent transaction conflicts.
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
