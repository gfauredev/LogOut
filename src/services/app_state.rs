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
use std::sync::Arc;

/// Returns `true` when the screen is currently locked and a write would be
/// restricted to the active session only.
///
/// On Android the check is driven by [`crate::ScreenLockedSignal`] which
/// polls `KeyguardManager.isKeyguardLocked()` once per second.
/// On all other platforms this always returns `false`.
fn screen_is_locked() -> bool {
    #[cfg(target_os = "android")]
    {
        *consume_context::<crate::ScreenLockedSignal>().0.read()
    }
    #[cfg(not(target_os = "android"))]
    {
        false
    }
}
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
/// All reads are issued concurrently.  Both `load_active_sessions` /
/// `compute_all_bests_rows` and `load_custom_exercises` hide their
/// platform-specific dispatch so this function contains no `#[cfg]` blocks.
async fn load_storage_data(
    mut sessions_sig: Signal<Vec<WorkoutSession>>,
    mut custom_sig: Signal<Vec<Arc<Exercise>>>,
    mut cache_sig: Signal<BestsCache>,
    mut toast: Signal<std::collections::VecDeque<String>>,
) {
    use super::storage;
    use futures_util::future::join3;
    let (active_res, bests_res, custom_res) = join3(
        storage::load_active_sessions(),
        storage::compute_all_bests_rows(),
        storage::load_custom_exercises(),
    )
    .await;
    let active = match active_res {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to load active sessions: {e}");
            toast
                .write()
                .push_back(format!("⚠️ Failed to load sessions: {e}"));
            vec![]
        }
    };
    let bests_rows = match bests_res {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to compute exercise bests: {e}");
            vec![]
        }
    };
    let custom = match custom_res {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to load custom exercises: {e}");
            toast
                .write()
                .push_back(format!("⚠️ Failed to load custom exercises: {e}"));
            vec![]
        }
    };
    log::info!(
        "Startup: {} active session(s); {} exercise bests loaded; {} custom exercise(s)",
        active.len(),
        bests_rows.len(),
        custom.len(),
    );
    if !active.is_empty() {
        sessions_sig.set(active);
    }
    cache_sig.set(bests_rows_to_cache(bests_rows));
    if !custom.is_empty() {
        custom_sig.set(custom.into_iter().map(Arc::new).collect());
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
    let previous;
    let is_update;
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
    let mut cache_sig = consume_context::<Signal<BestsCache>>();
    if !session.is_active() {
        if is_update {
            // Only evict exercises whose old log held the personal record.
            // If a log that was the PR is edited/removed, the cache is stale
            // and must be recomputed from storage.  Exercises whose old log
            // was below the PR are unaffected.
            let affected_ids: Vec<String> = {
                let cache = cache_sig.read();
                previous
                    .as_ref()
                    .map(|prev_session| {
                        prev_session
                            .exercise_logs
                            .iter()
                            .filter(|old_log| {
                                let bests =
                                    cache.get(&old_log.exercise_id).cloned().unwrap_or_default();
                                log_was_personal_record(old_log, &bests)
                            })
                            .map(|log| log.exercise_id.clone())
                            .collect::<std::collections::HashSet<_>>()
                            .into_iter()
                            .collect()
                    })
                    .unwrap_or_default()
            };
            if affected_ids.is_empty() {
                // No previously-cached PR was touched; just merge the new logs
                // incrementally so any improvements are recorded without a
                // storage round-trip.
                let mut cache = cache_sig.write();
                for log in &session.exercise_logs {
                    let entry = cache.entry(log.exercise_id.clone()).or_default();
                    merge_log_into_bests(entry, log);
                }
            } else {
                recompute_bests_for_exercises(affected_ids, cache_sig);
            }
        } else {
            // First-time completion: merge the new logs into the cache
            // incrementally — no storage round-trip required.
            let mut cache = cache_sig.write();
            for log in &session.exercise_logs {
                let entry = cache.entry(log.exercise_id.clone()).or_default();
                merge_log_into_bests(entry, log);
            }
        }
    } else if is_update {
        // Session is still active but was updated (e.g. a log was deleted).
        // Check if any previously-cached PR came from a log that is now
        // absent, and if so evict and recompute those entries from storage.
        let affected_ids: Vec<String> = {
            let cache = cache_sig.read();
            previous
                .as_ref()
                .map(|prev_session| {
                    let new_log_ids: std::collections::HashSet<_> = session
                        .exercise_logs
                        .iter()
                        .map(|l| (l.exercise_id.clone(), l.start_time))
                        .collect();
                    prev_session
                        .exercise_logs
                        .iter()
                        .filter(|old_log| {
                            // Log was removed from the session
                            !new_log_ids
                                .contains(&(old_log.exercise_id.clone(), old_log.start_time))
                        })
                        .filter(|old_log| {
                            let bests =
                                cache.get(&old_log.exercise_id).cloned().unwrap_or_default();
                            log_was_personal_record(old_log, &bests)
                        })
                        .map(|log| log.exercise_id.clone())
                        .collect::<std::collections::HashSet<_>>()
                        .into_iter()
                        .collect()
                })
                .unwrap_or_default()
        };
        if !affected_ids.is_empty() {
            recompute_bests_for_exercises(affected_ids, cache_sig);
        }
    }
    let toast = consume_context::<ToastSignal>().0;
    super::storage::enqueue_put_session(session, toast, sig, previous);
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
    let id = id.to_owned();
    let toast = consume_context::<ToastSignal>().0;
    super::storage::enqueue_delete_session(id, toast, sig, snapshot);
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
///
/// **`BestsCache` maintenance**: the new log is merged into the cache
/// immediately (incrementally) so that the ATH is updated at exercise
/// completion rather than waiting for the full session to be saved.
pub fn append_exercise_log(log: ExerciseLog) {
    let sig = use_sessions();
    let Some(session) = sig.read().iter().find(|s| s.is_active()).cloned() else {
        return;
    };
    // Update the BestsCache immediately for this exercise.
    {
        let mut cache_sig = consume_context::<Signal<BestsCache>>();
        let mut cache = cache_sig.write();
        let entry = cache.entry(log.exercise_id.clone()).or_default();
        merge_log_into_bests(entry, &log);
    }
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
    let toast = consume_context::<ToastSignal>().0;
    super::storage::enqueue_put_exercise(exercise, toast);
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
    let toast = consume_context::<ToastSignal>().0;
    super::storage::enqueue_put_exercise(exercise, toast);
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
    /// Weight from the most-recently completed log (for input prefilling).
    pub last_weight_hg: Option<Weight>,
    /// Reps from the most-recently completed log.
    pub last_reps: Option<u32>,
    /// Distance from the most-recently completed log.
    pub last_distance_m: Option<Distance>,
    /// `end_time` of the most-recently completed log.
    pub last_log_end_time: Option<u64>,
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
    if log.weight_hg.0 > 0 {
        let w = log.weight_hg;
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
    // Update last-log values if this log is more recent.
    let log_end = log.end_time.unwrap_or(0);
    if log_end > bests.last_log_end_time.unwrap_or(0) {
        bests.last_log_end_time = Some(log_end);
        bests.last_weight_hg = (log.weight_hg.0 > 0).then_some(log.weight_hg);
        bests.last_reps = log.reps;
        bests.last_distance_m = log.distance_m;
    }
}
/// Returns `true` when any of the log's recorded values exactly matches the
/// corresponding cached personal record, meaning this log was the record setter.
///
/// Only complete logs are checked; an incomplete log always returns `false`.
/// Used to determine whether deleting / editing a log requires a cache eviction.
pub(crate) fn log_was_personal_record(log: &ExerciseLog, bests: &ExerciseBests) -> bool {
    if !log.is_complete() {
        return false;
    }
    (log.weight_hg.0 > 0 && bests.weight_hg.is_some_and(|b| b == log.weight_hg))
        || (log.reps.is_some() && log.reps == bests.reps)
        || (log.distance_m.is_some() && log.distance_m == bests.distance_m)
        || (log.duration_seconds().is_some() && log.duration_seconds() == bests.duration)
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
    dioxus::prelude::spawn(async move {
        match super::storage::compute_bests_rows_for_exercises(exercise_ids).await {
            Ok(rows) => {
                let mut cache = cache_sig.write();
                for row in rows {
                    cache.insert(row.exercise_id.clone(), exercise_bests_from_row(&row));
                }
            }
            Err(e) => {
                log::error!("Failed to recompute bests for exercises: {e}");
            }
        }
    });
}
/// Clear the entire [`BestsCache`] and rebuild it from storage in a background task.
///
/// Used when a **historical completed session** is deleted and the affected
/// exercise IDs are not available in the in-memory signal.  The full rebuild
/// is O(N) but executes asynchronously, so it never blocks the UI.
pub(crate) fn recompute_all_bests(mut cache_sig: Signal<BestsCache>) {
    cache_sig.write().clear();
    dioxus::prelude::spawn(async move {
        match super::storage::compute_all_bests_rows().await {
            Ok(rows) => {
                cache_sig.set(bests_rows_to_cache(rows));
            }
            Err(e) => {
                log::error!("Failed to recompute all bests: {e}");
            }
        }
    });
}
/// Convert a storage [`BestsRow`] into the in-memory [`ExerciseBests`] representation.
fn exercise_bests_from_row(row: &super::storage::BestsRow) -> ExerciseBests {
    ExerciseBests {
        weight_hg: row.max_weight_hg.map(Weight),
        reps: row.max_reps,
        distance_m: row.max_distance_m.map(Distance),
        duration: row.max_duration_s,
        last_weight_hg: row.last_weight_hg.map(Weight),
        last_reps: row.last_reps,
        last_distance_m: row.last_distance_m.map(Distance),
        last_log_end_time: row.last_log_end_time,
    }
}
/// Convert a `Vec<BestsRow>` returned by storage into a full [`BestsCache`].
fn bests_rows_to_cache(rows: Vec<super::storage::BestsRow>) -> BestsCache {
    rows.into_iter()
        .map(|row| {
            let bests = exercise_bests_from_row(&row);
            (row.exercise_id, bests)
        })
        .collect()
}
