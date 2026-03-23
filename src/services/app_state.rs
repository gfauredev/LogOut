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
use std::sync::Arc;
/// Provide the shared workout-session and custom-exercise signals at the top of
/// the component tree.  Call exactly once inside the root `App` component.
pub fn provide_app_state() {
    let sessions_sig = use_context_provider(|| Signal::new(Vec::<WorkoutSession>::new()));
    let custom_sig = use_context_provider(|| Signal::new(Vec::<Arc<Exercise>>::new()));
    let cache_sig = use_context_provider(|| Signal::new(BestsCache::new()));
    let toast = consume_context::<ToastSignal>().0;
    use_resource(move || load_storage_data(sessions_sig, custom_sig, cache_sig, toast));
}
/// Obtain the reactive sessions signal from the Dioxus context.
pub fn use_sessions() -> Signal<Vec<WorkoutSession>> {
    consume_context::<Signal<Vec<WorkoutSession>>>()
}
/// Obtain the reactive custom-exercises signal from the Dioxus context.
pub fn use_custom_exercises() -> Signal<Vec<Arc<Exercise>>> {
    consume_context::<Signal<Vec<Arc<Exercise>>>>()
}
/// Load initial data from storage into the app signals.
///
/// Only **active** sessions are placed into the sessions signal; completed
/// sessions are accessed on demand through the pagination API
/// ([`super::storage::load_completed_sessions_page`]).  This avoids loading
/// the entire workout history into memory at startup.
///
/// The [`BestsCache`] is pre-populated in the same pass so that the first
/// call to [`get_exercise_bests`] for any exercise returns an immediately
/// correct value without scanning the sessions signal.
///
/// On the native target both blocking DB reads are wrapped in
/// [`tokio::task::spawn_blocking`] so they do not stall the async runtime.
/// On the web target the two `IndexedDB` reads are issued concurrently via
/// [`futures_util::future::join`].
#[allow(clippy::too_many_lines)]
async fn load_storage_data(
    mut sessions_sig: Signal<Vec<WorkoutSession>>,
    mut custom_sig: Signal<Vec<Arc<Exercise>>>,
    mut cache_sig: Signal<BestsCache>,
    mut toast: Signal<std::collections::VecDeque<String>>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        use super::storage::idb;
        use futures_util::future::join;
        // Issue both IDB reads concurrently — no need to wait for sessions
        // before starting the custom-exercises fetch.
        let (sessions_result, custom_result) = join(
            idb::get_all::<WorkoutSession>(idb::STORE_SESSIONS),
            idb::get_all::<Exercise>(idb::STORE_CUSTOM_EXERCISES),
        )
        .await;
        match sessions_result {
            Ok(sessions) => {
                // Single pass: separate active from completed and build the
                // BestsCache without keeping completed sessions in memory.
                let mut cache = BestsCache::new();
                let mut active = Vec::new();
                for session in sessions {
                    if session.is_active() {
                        active.push(session);
                    } else {
                        for log in &session.exercise_logs {
                            let entry = cache.entry(log.exercise_id.clone()).or_default();
                            merge_log_into_bests(entry, log);
                        }
                    }
                }
                info!(
                    "Startup: {} active session(s); bests cache populated",
                    active.len()
                );
                if !active.is_empty() {
                    sessions_sig.set(active);
                }
                cache_sig.set(cache);
            }
            Err(e) => {
                error!("Failed to load sessions from IndexedDB: {e}");
                toast
                    .write()
                    .push_back(format!("⚠️ Failed to load sessions: {e}"));
            }
        }
        match custom_result {
            Ok(custom) if !custom.is_empty() => {
                info!("Loaded {} custom exercises from IndexedDB", custom.len());
                custom_sig.set(custom.into_iter().map(Arc::new).collect());
            }
            Err(e) => {
                error!("Failed to load custom exercises from IndexedDB: {e}");
                toast
                    .write()
                    .push_back(format!("⚠️ Failed to load custom exercises: {e}"));
            }
            _ => {}
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use super::storage::native_storage;
        // Run both blocking reads concurrently; neither depends on the other.
        let (sessions_result, custom_result) = futures_util::future::join(
            tokio::task::spawn_blocking(|| {
                native_storage::get_all::<WorkoutSession>(native_storage::STORE_SESSIONS)
            }),
            tokio::task::spawn_blocking(|| {
                native_storage::get_all::<Exercise>(native_storage::STORE_CUSTOM_EXERCISES)
            }),
        )
        .await;
        match sessions_result {
            Ok(Ok(sessions)) => {
                let sessions: Vec<WorkoutSession> = sessions;
                log::info!(
                    "Startup: loaded {} session(s) from storage; computing bests",
                    sessions.len()
                );
                // Single pass: separate active sessions and build the BestsCache.
                let mut cache = BestsCache::new();
                let mut active = Vec::new();
                for session in sessions {
                    if session.is_active() {
                        active.push(session);
                    } else {
                        for log in &session.exercise_logs {
                            let entry = cache.entry(log.exercise_id.clone()).or_default();
                            merge_log_into_bests(entry, log);
                        }
                    }
                }
                if !active.is_empty() {
                    sessions_sig.set(active);
                }
                cache_sig.set(cache);
            }
            Ok(Err(e)) => {
                log::error!("Failed to load sessions: {e}");
                toast
                    .write()
                    .push_back(format!("⚠️ Failed to load sessions: {e}"));
            }
            Err(e) => {
                log::error!("spawn_blocking panicked loading sessions: {e}");
                toast
                    .write()
                    .push_back("⚠️ Failed to load sessions (internal error)".into());
            }
        }
        match custom_result {
            Ok(Ok(custom)) if !custom.is_empty() => {
                let custom: Vec<Exercise> = custom;
                log::info!("Loaded {} custom exercises from storage", custom.len());
                custom_sig.set(custom.into_iter().map(Arc::new).collect());
            }
            Ok(Err(e)) => {
                log::error!("Failed to load custom exercises: {e}");
                toast
                    .write()
                    .push_back(format!("⚠️ Failed to load custom exercises: {e}"));
            }
            Err(e) => {
                log::error!("spawn_blocking panicked loading custom exercises: {e}");
                toast
                    .write()
                    .push_back("⚠️ Failed to load custom exercises (internal error)".into());
            }
            _ => {}
        }
    }
}
/// Upsert `session` into the in-memory signal, then persist it to the backend.
///
/// If a session with the same `id` already exists in the signal it is replaced;
/// otherwise the session is appended.
///
/// **Optimistic update**: the signal is mutated immediately before the write
/// is confirmed.  If the background write fails the signal is reverted to its
/// previous state and an error toast is shown.
///
/// **`BestsCache` maintenance**:
/// * When a session is **completed for the first time**, its logs are merged
///   incrementally so the cache stays up-to-date without a storage query.
/// * When an **existing completed session is updated**, the affected entries
///   are evicted and a background task re-reads storage to recompute them
///   accurately.
pub fn save_session(session: WorkoutSession) {
    let mut sig = use_sessions();
    let previous: Option<WorkoutSession>;
    let is_update: bool;
    {
        let mut sessions = sig.write();
        if let Some(pos) = sessions.iter().position(|s| s.id == session.id) {
            previous = Some(sessions[pos].clone());
            sessions[pos] = session.clone();
            is_update = true;
        } else {
            previous = None;
            sessions.push(session.clone());
            is_update = false;
        }
    }
    if !session.is_active() {
        let mut cache_sig = consume_context::<Signal<BestsCache>>();
        if is_update {
            // Evict the stale entries and schedule a background recompute from
            // storage.  This avoids an O(N) synchronous re-scan of all sessions.
            let exercise_ids: Vec<String> = session
                .exercise_logs
                .iter()
                .map(|l| l.exercise_id.clone())
                .collect();
            recompute_bests_for_exercises(exercise_ids, cache_sig);
        } else {
            // First-time completion: merge the new logs into the cache
            // incrementally — no storage round-trip required.
            let mut cache = cache_sig.write();
            for log in &session.exercise_logs {
                let entry = cache.entry(log.exercise_id.clone()).or_default();
                merge_log_into_bests(entry, log);
            }
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        use super::storage::idb_queue;
        let toast = consume_context::<ToastSignal>().0;
        idb_queue::enqueue(idb_queue::IdbOp::PutSession {
            session,
            toast,
            sessions_sig: sig,
            previous,
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use super::storage::native_queue;
        let toast = consume_context::<ToastSignal>().0;
        native_queue::enqueue(native_queue::NativeOp::PutSession {
            session,
            toast,
            sessions_sig: sig,
            previous,
        });
    }
}
/// Remove the session with `id` from the in-memory signal and from the backend.
///
/// **Optimistic update**: the session is removed from the signal before the
/// backend delete is confirmed.  On failure the signal is restored and a toast
/// is shown.
///
/// If the deleted session's exercise logs are present in the in-memory signal
/// the affected [`BestsCache`] entries are refreshed via a targeted background
/// query.  Otherwise the entire cache is rebuilt from storage so no stale
/// personal-record values remain after deletion.
pub fn delete_session(id: &str) {
    let mut sig = use_sessions();
    // Capture the full session for potential revert and for exercise_id lookup.
    let snapshot: Option<WorkoutSession> = sig.read().iter().find(|s| s.id == id).cloned();
    let exercise_ids: Vec<String> = snapshot
        .as_ref()
        .map(|s| {
            s.exercise_logs
                .iter()
                .map(|l| l.exercise_id.clone())
                .collect()
        })
        .unwrap_or_default();
    sig.write().retain(|s| s.id != id);
    let cache_sig = consume_context::<Signal<BestsCache>>();
    if exercise_ids.is_empty() {
        // The session was not in the in-memory signal (historical completed
        // session) — we don't know which exercises are affected, so rebuild
        // the entire cache from storage.
        recompute_all_bests(cache_sig);
    } else {
        recompute_bests_for_exercises(exercise_ids, cache_sig);
    }
    #[cfg(target_arch = "wasm32")]
    {
        use super::storage::idb_queue;
        let id = id.to_owned();
        let toast = consume_context::<ToastSignal>().0;
        idb_queue::enqueue(idb_queue::IdbOp::DeleteSession {
            id,
            toast,
            sessions_sig: sig,
            snapshot,
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use super::storage::native_queue;
        let id = id.to_owned();
        let toast = consume_context::<ToastSignal>().0;
        native_queue::enqueue(native_queue::NativeOp::DeleteSession {
            id,
            toast,
            sessions_sig: sig,
            snapshot,
        });
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
    sig.write().push(Arc::new(exercise.clone()));
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
            exercises[pos] = Arc::new(exercise.clone());
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
#[derive(Clone, Default)]
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
/// The cache is **fully populated at startup** by scanning all completed
/// sessions once during [`load_storage_data`].  Subsequent calls to
/// [`get_exercise_bests`] are always O(1) cache lookups.
///
/// When a session is **completed for the first time** the new logs are merged
/// in directly (O(logs)), keeping the cache correct without a storage round-trip.
///
/// When a session is **deleted** or **updated** the affected entries are evicted
/// and a background async task recomputes them from storage, so the
/// synchronous hot path is never blocked by an O(N) scan.
pub(crate) type BestsCache = std::collections::HashMap<String, ExerciseBests>;
/// Merge one exercise log's values into an existing best, updating it in place.
pub(crate) fn merge_log_into_bests(bests: &mut ExerciseBests, log: &ExerciseLog) {
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
/// Always O(1): reads directly from the [`BestsCache`] that was populated at
/// startup and kept up-to-date by [`save_session`] / [`delete_session`].
/// Returns an empty [`ExerciseBests`] when no completed logs exist yet (e.g.
/// during the brief window while a background recompute is in flight).
pub fn get_exercise_bests(exercise_id: &str) -> ExerciseBests {
    let cache_sig = consume_context::<Signal<BestsCache>>();
    let cached = cache_sig.read().get(exercise_id).cloned();
    cached.unwrap_or_default()
}
/// Evict the cache entries for `exercise_ids` and schedule a background task
/// that re-reads storage and reinserts accurate values.
///
/// This is the targeted version used after an in-place session **update**:
/// only the exercises present in the edited session need to be refreshed.
pub(crate) fn recompute_bests_for_exercises(
    exercise_ids: Vec<String>,
    mut cache_sig: Signal<BestsCache>,
) {
    // Evict the stale entries immediately so the next read returns the
    // default (rather than a permanently wrong cached value) while the
    // background recompute is in flight.
    {
        let mut cache = cache_sig.write();
        for id in &exercise_ids {
            cache.remove(id);
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        use super::storage::idb;
        wasm_bindgen_futures::spawn_local(async move {
            match idb::get_all::<WorkoutSession>(idb::STORE_SESSIONS).await {
                Ok(sessions) => {
                    let id_set: std::collections::HashSet<&str> =
                        exercise_ids.iter().map(String::as_str).collect();
                    let mut new_bests = BestsCache::new();
                    for session in &sessions {
                        if !session.is_active() {
                            for log in &session.exercise_logs {
                                if id_set.contains(log.exercise_id.as_str()) {
                                    let entry =
                                        new_bests.entry(log.exercise_id.clone()).or_default();
                                    merge_log_into_bests(entry, log);
                                }
                            }
                        }
                    }
                    let mut cache = cache_sig.write();
                    for ex_id in &exercise_ids {
                        let bests = new_bests.remove(ex_id).unwrap_or_default();
                        cache.insert(ex_id.clone(), bests);
                    }
                }
                Err(e) => {
                    log::error!("Failed to recompute bests for exercises: {e}");
                }
            }
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use super::storage::native_storage;
        dioxus::prelude::spawn(async move {
            match tokio::task::spawn_blocking(move || {
                let sessions =
                    native_storage::get_all::<WorkoutSession>(native_storage::STORE_SESSIONS)?;
                let id_set: std::collections::HashSet<String> = exercise_ids.into_iter().collect();
                let mut new_bests = BestsCache::new();
                for session in &sessions {
                    if !session.is_active() {
                        for log in &session.exercise_logs {
                            if id_set.contains(&log.exercise_id) {
                                let entry = new_bests.entry(log.exercise_id.clone()).or_default();
                                merge_log_into_bests(entry, log);
                            }
                        }
                    }
                }
                Ok::<_, native_storage::StorageError>((id_set, new_bests))
            })
            .await
            {
                Ok(Ok((id_set, new_bests))) => {
                    let mut cache = cache_sig.write();
                    for ex_id in &id_set {
                        let bests = new_bests.get(ex_id).cloned().unwrap_or_default();
                        cache.insert(ex_id.clone(), bests);
                    }
                }
                Ok(Err(e)) => {
                    log::error!("Failed to recompute bests: {e}");
                }
                Err(e) => {
                    log::error!("spawn_blocking panicked for bests recompute: {e}");
                }
            }
        });
    }
}
/// Clear the entire [`BestsCache`] and rebuild it from storage in a background task.
///
/// Used when a **historical completed session** is deleted and the affected
/// exercise IDs are not available in the in-memory signal.  The full rebuild
/// is O(N) but executes asynchronously, so it never blocks the UI.
pub(crate) fn recompute_all_bests(mut cache_sig: Signal<BestsCache>) {
    cache_sig.write().clear();
    #[cfg(target_arch = "wasm32")]
    {
        use super::storage::idb;
        wasm_bindgen_futures::spawn_local(async move {
            match idb::get_all::<WorkoutSession>(idb::STORE_SESSIONS).await {
                Ok(sessions) => {
                    let mut new_cache = BestsCache::new();
                    for session in &sessions {
                        if !session.is_active() {
                            for log in &session.exercise_logs {
                                let entry = new_cache.entry(log.exercise_id.clone()).or_default();
                                merge_log_into_bests(entry, log);
                            }
                        }
                    }
                    cache_sig.set(new_cache);
                }
                Err(e) => {
                    log::error!("Failed to rebuild bests cache: {e}");
                }
            }
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use super::storage::native_storage;
        dioxus::prelude::spawn(async move {
            match tokio::task::spawn_blocking(|| {
                native_storage::get_all::<WorkoutSession>(native_storage::STORE_SESSIONS)
            })
            .await
            {
                Ok(Ok(sessions)) => {
                    let mut new_cache = BestsCache::new();
                    for session in &sessions {
                        if !session.is_active() {
                            for log in &session.exercise_logs {
                                let entry = new_cache.entry(log.exercise_id.clone()).or_default();
                                merge_log_into_bests(entry, log);
                            }
                        }
                    }
                    cache_sig.set(new_cache);
                }
                Ok(Err(e)) => {
                    log::error!("Failed to rebuild bests cache: {e}");
                }
                Err(e) => {
                    log::error!("spawn_blocking panicked for bests cache rebuild: {e}");
                }
            }
        });
    }
}
