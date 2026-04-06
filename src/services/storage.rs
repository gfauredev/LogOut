//! Platform-specific storage backends for the `LogOut` application.
//!
//! This module provides two storage backends that share the same logical interface:
//!
//! - **Web** (`wasm32`): `IndexedDB` via the `rexie` crate, serialised through an
//!   async write queue so concurrent callers never fight over read-write
//!   transactions.
//! - **Native** (Android / desktop): `SQLite` via `rusqlite` stored in the OS
//!   app-data directory.
//!
//! All Dioxus reactive state (signals, context helpers, mutation functions) lives
//! in the sibling [`app_state`](super::app_state) module and is re-exported here
//! for backward compatibility.
pub use super::app_state::{
    add_custom_exercise, append_exercise_log, begin_exercise_in_session,
    cancel_exercise_in_session, delete_session, get_exercise_bests, get_last_exercise_log,
    provide_app_state, save_session, start_pending_exercise_in_session, update_custom_exercise,
    use_custom_exercises, use_sessions,
};
/// Aggregated per-exercise personal-record values returned by
/// [`compute_all_bests_rows`] and [`compute_bests_rows_for_exercises`].
///
/// Raw numeric types match the storage representation so callers can build
/// [`crate::services::app_state::ExerciseBests`] values without an extra
/// conversion step.
pub struct BestsRow {
    /// The exercise this row describes.
    pub exercise_id: String,
    /// Maximum `weight_hg` (hectograms) across all completed logs.
    pub max_weight_hg: Option<u16>,
    /// Maximum repetition count across all completed logs.
    pub max_reps: Option<u32>,
    /// Maximum `distance_m` (metres) across all completed logs.
    pub max_distance_m: Option<u32>,
    /// Maximum set duration (seconds) across all completed logs.
    pub max_duration_s: Option<u64>,
    /// `weight_hg` from the most-recently completed log (for input prefilling).
    pub last_weight_hg: Option<u16>,
    /// Repetition count from the most-recently completed log.
    pub last_reps: Option<u32>,
    /// `distance_m` from the most-recently completed log.
    pub last_distance_m: Option<u32>,
    /// `end_time` of the most-recently completed log (used to merge entries).
    pub last_log_end_time: Option<u64>,
}
/// Unified error type returned by all async storage read operations.
///
/// Wraps platform-specific errors (`IndexedDB` on `wasm32`, `SQLite` on native)
/// as a human-readable message so callers need no platform-specific `match`
/// arms.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// The underlying storage backend returned an error.
    #[error("{0}")]
    Backend(String),
    /// A blocking background task panicked (native platforms only).
    #[error("Background task panicked: {0}")]
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    TaskPanic(String),
}
#[cfg(target_arch = "wasm32")]
impl From<idb::IdbError> for StorageError {
    fn from(e: idb::IdbError) -> Self {
        StorageError::Backend(e.to_string())
    }
}
#[cfg(not(target_arch = "wasm32"))]
impl From<native_storage::StorageError> for StorageError {
    fn from(e: native_storage::StorageError) -> Self {
        StorageError::Backend(e.to_string())
    }
}
/// Unified async interface implemented by both the `IndexedDB` (web) and `SQLite`
/// (native) storage backends.
///
/// Dispatch happens through [`platform_storage()`], keeping the public API
/// functions free of `#[cfg(target_arch = "wasm32")]` branching.  Business
/// logic is therefore decoupled from the platform-specific storage layer.
pub trait AsyncStorageProvider {
    /// Load a page of completed sessions, sorted by `start_time` descending.
    async fn load_completed_sessions_page(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<crate::models::WorkoutSession>, StorageError>;
    /// Load all active (in-progress) sessions.
    async fn load_active_sessions(
        &self,
    ) -> Result<Vec<crate::models::WorkoutSession>, StorageError>;
    /// Load all custom exercises.
    async fn load_custom_exercises(&self) -> Result<Vec<crate::models::Exercise>, StorageError>;
    /// Compute per-exercise all-time bests across every completed session.
    async fn compute_all_bests_rows(&self) -> Result<Vec<BestsRow>, StorageError>;
    /// Compute per-exercise all-time bests restricted to the given IDs.
    async fn compute_bests_rows_for_exercises(
        &self,
        exercise_ids: Vec<String>,
    ) -> Result<Vec<BestsRow>, StorageError>;
    /// Returns the total number of sessions in storage.
    async fn session_count(&self) -> Result<usize, StorageError>;
}
/// Returns the platform-specific storage backend.
///
/// Selection is made at compile time: [`IdbStorage`] on `wasm32` and
/// [`NativeStorage`] on native.  Keeping the `#[cfg]` dispatch in this single
/// private helper allows every public API function to remain free of inline
/// conditional compilation.
#[cfg(target_arch = "wasm32")]
#[inline]
fn platform_storage() -> IdbStorage {
    IdbStorage
}
#[cfg(not(target_arch = "wasm32"))]
#[inline]
fn platform_storage() -> NativeStorage {
    NativeStorage
}
/// Load a page of completed sessions, sorted by `start_time` descending.
///
/// On native platforms, executes a SQL query with `LIMIT`/`OFFSET` so only
/// the requested rows are transferred from the database, avoiding the need to
/// load, clone, and sort the entire history in memory.
///
/// On the web platform, all sessions are retrieved from `IndexedDB`, then
/// filtered, sorted, and sliced — true cursor-based pagination requires IDB
/// indices which would add schema-migration complexity.
///
/// Returns `Err` when storage access fails, allowing the UI to surface the
/// error appropriately.
pub async fn load_completed_sessions_page(
    limit: usize,
    offset: usize,
) -> Result<Vec<crate::models::WorkoutSession>, StorageError> {
    platform_storage()
        .load_completed_sessions_page(limit, offset)
        .await
}
/// Load only the **active** (in-progress) sessions from storage.
///
/// On native this issues `SELECT … WHERE end_time IS NULL`, so completed
/// sessions are never deserialised.  On wasm all sessions are fetched from
/// `IndexedDB` and completed ones are discarded immediately.
///
/// Returns `Err` when storage access fails, allowing the UI to surface the
/// error appropriately.
pub async fn load_active_sessions() -> Result<Vec<crate::models::WorkoutSession>, StorageError> {
    platform_storage().load_active_sessions().await
}
/// Returns the total number of sessions in storage (active + completed).
///
/// Selection happens at the database level so this is O(1) regardless of
/// the number of sessions on disk.
pub async fn load_session_count() -> Result<usize, StorageError> {
    platform_storage().session_count().await
}
/// Load all custom exercises from storage.
///
/// Returns `Err` when storage access fails, allowing the UI to surface the
/// error appropriately.
pub async fn load_custom_exercises() -> Result<Vec<crate::models::Exercise>, StorageError> {
    platform_storage().load_custom_exercises().await
}
/// Compute per-exercise all-time bests across every **completed** session.
///
/// On native this executes a single SQL aggregation query so no session JSON
/// is ever deserialised into Rust structs.  On wasm all sessions are fetched
/// from `IndexedDB` and aggregated in memory (SQL is not available there).
///
/// Returns `Err` when storage access fails.
pub async fn compute_all_bests_rows() -> Result<Vec<BestsRow>, StorageError> {
    platform_storage().compute_all_bests_rows().await
}
/// Compute per-exercise all-time bests restricted to the given `exercise_ids`.
///
/// On native this passes the IDs directly to the SQL query so only the
/// requested exercises are aggregated.  On wasm all sessions are loaded and
/// filtered in memory.
///
/// Returns `Err` when storage access fails.
pub async fn compute_bests_rows_for_exercises(
    exercise_ids: Vec<String>,
) -> Result<Vec<BestsRow>, StorageError> {
    platform_storage()
        .compute_bests_rows_for_exercises(exercise_ids)
        .await
}
/// Aggregate per-exercise bests from an in-memory session slice (wasm helper).
#[cfg(target_arch = "wasm32")]
fn bests_rows_from_sessions(sessions: &[crate::models::WorkoutSession]) -> Vec<BestsRow> {
    fn update_max<T: Ord>(slot: &mut Option<T>, new: T) {
        match slot {
            None => *slot = Some(new),
            Some(prev) => {
                if new > *prev {
                    *prev = new;
                }
            }
        }
    }
    let mut map: std::collections::HashMap<String, BestsRow> = std::collections::HashMap::new();
    for session in sessions {
        if !session.is_active() {
            for log in &session.exercise_logs {
                if !log.is_complete() {
                    continue;
                }
                let entry = map
                    .entry(log.exercise_id.clone())
                    .or_insert_with(|| BestsRow {
                        exercise_id: log.exercise_id.clone(),
                        max_weight_hg: None,
                        max_reps: None,
                        max_distance_m: None,
                        max_duration_s: None,
                        last_weight_hg: None,
                        last_reps: None,
                        last_distance_m: None,
                        last_log_end_time: None,
                    });
                if log.weight_hg.0 > 0 {
                    update_max(&mut entry.max_weight_hg, log.weight_hg.0);
                }
                if let Some(r) = log.reps {
                    update_max(&mut entry.max_reps, r);
                }
                if let Some(d) = log.distance_m {
                    update_max(&mut entry.max_distance_m, d.0);
                }
                if let Some(dur) = log.duration_seconds() {
                    update_max(&mut entry.max_duration_s, dur);
                }
                // Track most-recently completed log per exercise for prefilling.
                let log_end = log.end_time.unwrap_or(0);
                if log_end > entry.last_log_end_time.unwrap_or(0) {
                    entry.last_log_end_time = Some(log_end);
                    entry.last_weight_hg = (log.weight_hg.0 > 0).then_some(log.weight_hg.0);
                    entry.last_reps = log.reps;
                    entry.last_distance_m = log.distance_m.map(|d| d.0);
                }
            }
        }
    }
    map.into_values().collect()
}
/// Enqueue a session upsert on the platform-specific background write queue.
///
/// Abstracts over [`idb_queue`] (web) and [`native_queue`] (native) so
/// callers in [`super::app_state`] need no `#[cfg]` for this operation.
pub fn enqueue_put_session(
    session: crate::models::WorkoutSession,
    toast: dioxus::signals::Signal<std::collections::VecDeque<String>>,
    sessions_sig: dioxus::signals::Signal<Vec<crate::models::WorkoutSession>>,
    previous: Option<crate::models::WorkoutSession>,
) {
    #[cfg(target_arch = "wasm32")]
    idb_queue::enqueue(idb_queue::IdbOp::PutSession {
        session,
        toast,
        sessions_sig,
        previous,
    });
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (toast, sessions_sig); // Used via use_native_results
        native_queue::enqueue(native_queue::NativeOp::PutSession { session, previous });
    }
}
/// Enqueue a session deletion on the platform-specific background write queue.
pub fn enqueue_delete_session(
    id: String,
    toast: dioxus::signals::Signal<std::collections::VecDeque<String>>,
    sessions_sig: dioxus::signals::Signal<Vec<crate::models::WorkoutSession>>,
    snapshot: Option<crate::models::WorkoutSession>,
) {
    #[cfg(target_arch = "wasm32")]
    idb_queue::enqueue(idb_queue::IdbOp::DeleteSession {
        id,
        toast,
        sessions_sig,
        snapshot,
    });
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (toast, sessions_sig); // Used via use_native_results
        native_queue::enqueue(native_queue::NativeOp::DeleteSession { id, snapshot });
    }
}
/// Enqueue a custom-exercise upsert on the platform-specific background write queue.
pub fn enqueue_put_exercise(
    exercise: crate::models::Exercise,
    toast: dioxus::signals::Signal<std::collections::VecDeque<String>>,
) {
    #[cfg(target_arch = "wasm32")]
    idb_queue::enqueue(idb_queue::IdbOp::PutExercise(exercise, toast));
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = toast; // Used via use_native_results
        native_queue::enqueue(native_queue::NativeOp::PutExercise(exercise));
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) mod idb {
    use rexie::{ObjectStore, Rexie, TransactionMode};
    use wasm_bindgen::JsValue;
    const DB_NAME: &str = "log_out_db";
    const DB_VERSION: u32 = 3;
    pub const STORE_SESSIONS: &str = "sessions";
    pub const STORE_CUSTOM_EXERCISES: &str = "custom_exercises";
    pub const STORE_EXERCISES: &str = "exercises";
    /// Dedicated object store for binary image data (key: UUID string, value: `Uint8Array`).
    pub const STORE_IMAGES: &str = "images";
    /// Structured error type for `IndexedDB` operations via the `rexie` crate.
    ///
    /// Using a typed enum instead of `String` preserves the underlying cause so
    /// that callers can inspect or display it with full context.
    #[derive(Debug, thiserror::Error)]
    pub enum IdbError {
        /// A lower-level `rexie` / IndexedDB error.
        #[error("IndexedDB error: {0}")]
        Rexie(#[from] rexie::Error),
        /// A `serde-wasm-bindgen` serialisation or deserialisation error.
        #[error("Serialization error: {0}")]
        Serde(#[from] serde_wasm_bindgen::Error),
    }
    /// Open (or create) the IndexedDB database via rexie.
    pub(super) async fn open_db() -> Result<Rexie, rexie::Error> {
        Rexie::builder(DB_NAME)
            .version(DB_VERSION)
            .add_object_store(ObjectStore::new(STORE_SESSIONS).key_path("id"))
            .add_object_store(ObjectStore::new(STORE_CUSTOM_EXERCISES).key_path("id"))
            .add_object_store(ObjectStore::new(STORE_EXERCISES).key_path("id"))
            .add_object_store(ObjectStore::new(STORE_IMAGES))
            .build()
            .await
    }
    /// Put a single serialisable item into a store (upsert by key).
    pub async fn put_item<T: serde::Serialize>(store_name: &str, item: &T) -> Result<(), IdbError> {
        let db = open_db().await?;
        let tx = db.transaction(&[store_name], TransactionMode::ReadWrite)?;
        let store = tx.store(store_name)?;
        let js_val = serde_wasm_bindgen::to_value(item)?;
        store.put(&js_val, None).await?;
        tx.done().await?;
        Ok(())
    }
    /// Put many serialisable items into a store in a single transaction.
    /// More efficient than calling [`put_item`] in a loop because only one
    /// database connection and one transaction are opened.
    ///
    /// Serialisation is performed in chunks of [`PUT_ALL_CHUNK_SIZE`] items,
    /// yielding to the browser's macro-task queue between each chunk via a
    /// zero-delay `setTimeout`.  This prevents large datasets (e.g. 800+
    /// exercises) from synchronously blocking the UI thread during the
    /// serialisation phase.  The IndexedDB transaction is opened only after
    /// all serialisation is complete so the yield points cannot cause the
    /// transaction to auto-commit prematurely.
    ///
    /// All individual `put` requests are issued concurrently within the same
    /// transaction via [`futures_util::future::try_join_all`] so the browser can
    /// pipeline them instead of waiting for each one before issuing the next.
    pub async fn put_all<T: serde::Serialize>(
        store_name: &str,
        items: &[T],
    ) -> Result<(), IdbError> {
        /// Number of items to serialise per chunk before yielding.
        const PUT_ALL_CHUNK_SIZE: usize = 50;
        // Serialise in chunks, yielding to the macro-task queue between each
        // chunk so the browser can paint frames and handle input events.
        // The transaction is opened only after all serialisation completes so
        // that the yield points never risk triggering an early auto-commit.
        let mut js_values = Vec::with_capacity(items.len());
        for chunk in items.chunks(PUT_ALL_CHUNK_SIZE) {
            for item in chunk {
                js_values.push(serde_wasm_bindgen::to_value(item)?);
            }
            gloo_timers::future::TimeoutFuture::new(0).await;
        }
        let db = open_db().await?;
        let tx = db.transaction(&[store_name], TransactionMode::ReadWrite)?;
        let store = tx.store(store_name)?;
        let put_futs: Vec<_> = js_values
            .iter()
            .map(|js_val| store.put(js_val, None))
            .collect();
        futures_util::future::try_join_all(put_futs).await?;
        tx.done().await?;
        Ok(())
    }
    /// Delete an item from a store by its key.
    pub async fn delete_item(store_name: &str, key: &str) -> Result<(), IdbError> {
        let db = open_db().await?;
        let tx = db.transaction(&[store_name], TransactionMode::ReadWrite)?;
        let store = tx.store(store_name)?;
        store.delete(JsValue::from_str(key)).await?;
        tx.done().await?;
        Ok(())
    }
    /// Remove all items from a store.
    pub async fn clear_all(store_name: &str) -> Result<(), IdbError> {
        let db = open_db().await?;
        let tx = db.transaction(&[store_name], TransactionMode::ReadWrite)?;
        let store = tx.store(store_name)?;
        store.clear().await?;
        tx.done().await?;
        Ok(())
    }
    /// Load all items from a store.
    pub async fn get_all<T: serde::de::DeserializeOwned>(
        store_name: &str,
    ) -> Result<Vec<T>, IdbError> {
        let db = open_db().await?;
        let tx = db.transaction(&[store_name], TransactionMode::ReadOnly)?;
        let store = tx.store(store_name)?;
        let js_values = store.get_all(None, None).await?;
        let mut items = Vec::new();
        for (i, js_val) in js_values.into_iter().enumerate() {
            match serde_wasm_bindgen::from_value::<T>(js_val) {
                Ok(item) => items.push(item),
                Err(e) => {
                    log::warn!("Skipping corrupt IndexedDB entry at index {i}: {e}")
                }
            }
        }
        Ok(items)
    }
}
/// Zero-size marker type that binds [`AsyncStorageProvider`] to the
/// `IndexedDB` backend exposed by [`idb`].
#[cfg(target_arch = "wasm32")]
pub struct IdbStorage;
/// [`AsyncStorageProvider`] implementation for the `IndexedDB` (wasm32) backend.
#[cfg(target_arch = "wasm32")]
impl AsyncStorageProvider for IdbStorage {
    async fn load_completed_sessions_page(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<crate::models::WorkoutSession>, StorageError> {
        let mut sessions =
            idb::get_all::<crate::models::WorkoutSession>(idb::STORE_SESSIONS).await?;
        sessions.retain(|s| !s.is_active());
        sessions.sort_by(|a, b| b.start_time.cmp(&a.start_time));
        Ok(sessions.into_iter().skip(offset).take(limit).collect())
    }
    async fn load_active_sessions(
        &self,
    ) -> Result<Vec<crate::models::WorkoutSession>, StorageError> {
        let sessions = idb::get_all::<crate::models::WorkoutSession>(idb::STORE_SESSIONS).await?;
        Ok(sessions.into_iter().filter(|s| s.is_active()).collect())
    }
    async fn load_custom_exercises(&self) -> Result<Vec<crate::models::Exercise>, StorageError> {
        Ok(idb::get_all::<crate::models::Exercise>(idb::STORE_CUSTOM_EXERCISES).await?)
    }
    async fn compute_all_bests_rows(&self) -> Result<Vec<BestsRow>, StorageError> {
        let sessions = idb::get_all::<crate::models::WorkoutSession>(idb::STORE_SESSIONS).await?;
        Ok(bests_rows_from_sessions(&sessions))
    }
    async fn compute_bests_rows_for_exercises(
        &self,
        exercise_ids: Vec<String>,
    ) -> Result<Vec<BestsRow>, StorageError> {
        let id_set: std::collections::HashSet<String> = exercise_ids.into_iter().collect();
        let all = self.compute_all_bests_rows().await?;
        Ok(all
            .into_iter()
            .filter(|row| id_set.contains(&row.exercise_id))
            .collect())
    }
    async fn session_count(&self) -> Result<usize, StorageError> {
        Ok(
            idb::get_all::<crate::models::WorkoutSession>(idb::STORE_SESSIONS)
                .await?
                .len(),
        )
    }
}
#[cfg(target_arch = "wasm32")]
pub(crate) mod idb_queue {
    use super::idb;
    use crate::models::{Exercise, WorkoutSession};
    use dioxus::prelude::WritableExt;
    use dioxus::signals::Signal;
    use std::cell::RefCell;
    use std::collections::VecDeque;
    /// A pending write operation, including the toast signal for error reporting.
    pub enum IdbOp {
        /// Upsert a session.  On write failure the sessions signal is reverted to
        /// `previous` (the value before the optimistic update).
        PutSession {
            session: WorkoutSession,
            toast: Signal<std::collections::VecDeque<String>>,
            sessions_sig: Signal<Vec<WorkoutSession>>,
            /// `None` means the session was newly inserted; reverting removes it.
            /// `Some(old)` means it was an update; reverting restores `old`.
            previous: Option<WorkoutSession>,
        },
        /// Delete a session by ID.  On failure the sessions signal is restored
        /// using `snapshot` (if the session was present in the signal).
        DeleteSession {
            id: String,
            toast: Signal<std::collections::VecDeque<String>>,
            sessions_sig: Signal<Vec<WorkoutSession>>,
            /// The session that was removed from the signal, for revert on failure.
            snapshot: Option<WorkoutSession>,
        },
        PutExercise(Exercise, Signal<std::collections::VecDeque<String>>),
    }
    thread_local! {
        /// (draining, pending_ops)
        static QUEUE: RefCell<(bool, VecDeque<IdbOp>)> = RefCell::new((
            false,
            VecDeque::new(),
        ));
    }
    /// Enqueue a write operation.  If no drain is currently running, starts one.
    pub fn enqueue(op: IdbOp) {
        QUEUE.with(|q| {
            let mut q = q.borrow_mut();
            q.1.push_back(op);
            if !q.0 {
                q.0 = true;
                wasm_bindgen_futures::spawn_local(drain());
            }
        });
    }
    /// Drain all pending queue items immediately.
    ///
    /// Intended to be called from a `pagehide` / `beforeunload` handler so that
    /// writes that are still queued when the user closes the tab are not lost.
    /// The function spawns a local future and returns immediately; the async
    /// operations run as microtasks before the browser tears down the JS context.
    pub fn flush() {
        QUEUE.with(|q| {
            let draining = q.borrow().0;
            if !draining {
                q.borrow_mut().0 = true;
                wasm_bindgen_futures::spawn_local(drain());
            }
        });
    }
    async fn drain() {
        loop {
            let op = QUEUE.with(|q| q.borrow_mut().1.pop_front());
            match op {
                None => {
                    QUEUE.with(|q| q.borrow_mut().0 = false);
                    break;
                }
                Some(IdbOp::PutSession {
                    session: s,
                    mut toast,
                    mut sessions_sig,
                    previous,
                }) => {
                    if let Err(e) = idb::put_item(idb::STORE_SESSIONS, &s).await {
                        log::error!("IDB queue: failed to put session {}: {e}", s.id);
                        toast
                            .write()
                            .push_back(format!("⚠️ Failed to save session: {e}"));
                        // Revert the optimistic signal update.
                        let mut sessions = sessions_sig.write();
                        match previous {
                            None => sessions.retain(|x| x.id != s.id),
                            Some(old) => {
                                if let Some(pos) = sessions.iter().position(|x| x.id == s.id) {
                                    sessions[pos] = old;
                                }
                            }
                        }
                    }
                }
                Some(IdbOp::DeleteSession {
                    id,
                    mut toast,
                    mut sessions_sig,
                    snapshot,
                }) => {
                    if let Err(e) = idb::delete_item(idb::STORE_SESSIONS, &id).await {
                        log::error!("IDB queue: failed to delete session {id}: {e}");
                        toast
                            .write()
                            .push_back(format!("⚠️ Failed to delete session: {e}"));
                        // Revert: re-insert the session into the signal if we
                        // had a snapshot of it.
                        if let Some(session) = snapshot {
                            sessions_sig.write().push(session);
                        }
                    }
                }
                Some(IdbOp::PutExercise(ex, mut toast)) => {
                    if let Err(e) = idb::put_item(idb::STORE_CUSTOM_EXERCISES, &ex).await {
                        log::error!("IDB queue: failed to put exercise {}: {e}", ex.id);
                        toast
                            .write()
                            .push_back(format!("⚠️ Failed to save exercise: {e}"));
                    }
                }
            }
        }
    }
    /// Register a `pagehide` event listener that flushes any remaining queued
    /// writes before the browser may terminate the page.
    ///
    /// Call once at app startup.  The closure is intentionally leaked
    /// (`Closure::forget`) because it must live for the duration of the page.
    pub fn register_pagehide_flush() {
        use wasm_bindgen::prelude::Closure;
        use wasm_bindgen::JsCast as _;
        let closure: Closure<dyn Fn()> = Closure::wrap(Box::new(|| {
            flush();
        }));
        if let Some(window) = web_sys::window() {
            let _ = window
                .add_event_listener_with_callback("pagehide", closure.as_ref().unchecked_ref());
        }
        closure.forget();
    }
}
/// IndexedDB-backed exercise storage for the web platform.
#[cfg(target_arch = "wasm32")]
pub mod idb_exercises {
    use super::idb;
    use crate::models::Exercise;
    /// Retrieve all cached exercises from the IndexedDB exercises store.
    pub async fn get_all_exercises() -> Result<Vec<Exercise>, idb::IdbError> {
        idb::get_all::<Exercise>(idb::STORE_EXERCISES).await
    }
    /// Persist `exercises` to the IndexedDB exercises store in a single transaction.
    pub async fn store_all_exercises(exercises: &[Exercise]) {
        if let Err(e) = idb::put_all(idb::STORE_EXERCISES, exercises).await {
            log::error!("Failed to store exercises in IndexedDB: {e}");
        }
    }
    /// Remove all cached exercises from the IndexedDB exercises store.
    pub async fn clear_all_exercises() {
        if let Err(e) = idb::clear_all(idb::STORE_EXERCISES).await {
            log::error!("Failed to clear exercises from IndexedDB: {e}");
        }
    }
}
/// `IndexedDB`-backed binary image storage for the web platform.
///
/// Images are stored as raw bytes under a stable UUID key.  Only the UUID is
/// written into [`Exercise::images`], keeping the JSON metadata small.  The
/// actual bytes are fetched here on demand – only when an exercise card is
/// rendered – and turned into a short-lived `blob:` URL for the `<img>` tag.
#[cfg(target_arch = "wasm32")]
pub mod idb_images {
    use js_sys::{ArrayBuffer, Uint8Array};
    use rexie::TransactionMode;
    use wasm_bindgen::JsValue;
    use web_sys::Url;
    /// Persist `bytes` under `image_key` in the `images` object store.
    pub async fn store_image(image_key: &str, bytes: &[u8]) -> Result<(), super::idb::IdbError> {
        let db = super::idb::open_db().await?;
        let tx = db.transaction(&[super::idb::STORE_IMAGES], TransactionMode::ReadWrite)?;
        let store = tx.store(super::idb::STORE_IMAGES)?;
        let arr = Uint8Array::from(bytes);
        let key = JsValue::from_str(image_key);
        store.put(&arr.into(), Some(&key)).await?;
        tx.done().await?;
        Ok(())
    }
    /// Load the bytes stored under `image_key` and return a `blob:` URL that can be
    /// used directly in an `<img src>` attribute.  Returns `None` when the key
    /// is not found.  The caller is responsible for calling
    /// `URL.revokeObjectURL` when the URL is no longer needed.
    pub async fn get_image_blob_url(image_key: &str) -> Option<String> {
        let db = super::idb::open_db().await.ok()?;
        let tx = db
            .transaction(&[super::idb::STORE_IMAGES], TransactionMode::ReadOnly)
            .ok()?;
        let store = tx.store(super::idb::STORE_IMAGES).ok()?;
        let key = JsValue::from_str(image_key);
        let value = store.get(key).await.ok()??;
        if value.is_undefined() || value.is_null() {
            return None;
        }
        let array_buffer = ArrayBuffer::from(value);
        let bytes = Uint8Array::new(&array_buffer);
        let byte_vec: Vec<u8> = bytes.to_vec();
        let uint8_array = Uint8Array::from(byte_vec.as_slice());
        let parts = js_sys::Array::new();
        parts.push(&uint8_array.buffer());
        let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &{
            let opts = web_sys::BlobPropertyBag::new();
            opts.set_type("image/*");
            opts
        })
        .ok()?;
        Url::create_object_url_with_blob(&blob).ok()
    }
}
/// File-backed exercise storage for native platforms (Android / desktop).
#[cfg(not(target_arch = "wasm32"))]
pub mod native_exercises {
    use super::native_storage;
    use crate::models::Exercise;
    /// Retrieve all cached exercises from the `SQLite` exercises store.
    pub fn get_all_exercises() -> Vec<Exercise> {
        native_storage::get_all::<Exercise>(native_storage::STORE_EXERCISES).unwrap_or_default()
    }
    /// Persist `exercises` to the `SQLite` exercises store.
    pub fn store_all_exercises(exercises: &[Exercise]) {
        if let Err(e) = native_storage::store_all(native_storage::STORE_EXERCISES, exercises) {
            log::error!("Failed to store exercises: {e}");
        }
    }
    /// Remove all cached exercises from the `SQLite` exercises store.
    pub fn clear_all_exercises() {
        if let Err(e) = native_storage::store_all::<Exercise>(native_storage::STORE_EXERCISES, &[])
        {
            log::error!("Failed to clear exercises: {e}");
        }
    }
}
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use super::native_queue;
/// `SQLite`-backed storage for Android and desktop builds.
///
/// A single `log-out.db` `SQLite` database file is kept inside the app-
/// specific data directory.  On Android this is `Context.getFilesDir()`
/// (queried via `ndk-context` at runtime); on other platforms it is
/// `dirs::data_local_dir()/log-out/`.
/// Each "store" maps to a table with columns `id TEXT PRIMARY KEY, data TEXT`.
/// A separate `config` table holds arbitrary key/value string pairs.
///
/// On first launch, the database is initialized with the current schema.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod native_storage {
    use rusqlite::{params, Connection};
    use serde::{de::DeserializeOwned, Serialize};
    use std::path::PathBuf;
    /// On Android, ask the OS for the app's internal files directory via JNI.
    ///
    /// Uses `ndk_context::android_context()` (which Dioxus already sets up)
    /// to obtain the `JavaVM` and `Activity` pointers without requiring any
    /// custom `MainActivity.kt`.  Calls `Activity.getFilesDir()` and then
    /// `File.getAbsolutePath()` to get a `String` path back.
    ///
    /// Returns `None` on any JNI error; `data_dir()` will then fall back to
    /// `dirs::data_local_dir()`.
    #[cfg(target_os = "android")]
    fn android_files_dir() -> Option<PathBuf> {
        use jni::{objects::JObject, JavaVM};
        let ctx = ndk_context::android_context();
        if ctx.vm().is_null() {
            log::error!("android_files_dir: JavaVM pointer is NULL! ndk-context not initialized?");
            return None;
        }
        if ctx.context().is_null() {
            log::error!("android_files_dir: Context pointer is NULL! ndk-context not initialized?");
            return None;
        }
        // SAFETY: pointers are valid for the lifetime of the process and were
        // set up by the Dioxus / Android runtime before Rust code runs.
        let vm = match unsafe { JavaVM::from_raw(ctx.vm().cast()) } {
            Ok(vm) => vm,
            Err(e) => {
                log::error!("android_files_dir: JavaVM::from_raw failed: {e:?}");
                return None;
            }
        };
        let mut env = match vm.attach_current_thread() {
            Ok(env) => env,
            Err(e) => {
                log::error!("android_files_dir: attach_current_thread failed: {e:?}");
                return None;
            }
        };
        let activity = unsafe { JObject::from_raw(ctx.context() as jni::sys::jobject) };
        let files_dir = match env
            .call_method(&activity, "getFilesDir", "()Ljava/io/File;", &[])
            .and_then(jni::objects::JValueGen::l)
        {
            Ok(obj) => obj,
            Err(e) => {
                log::error!("android_files_dir: getFilesDir() failed: {e:?}");
                return None;
            }
        };
        let path_jobj = match env
            .call_method(&files_dir, "getAbsolutePath", "()Ljava/lang/String;", &[])
            .and_then(jni::objects::JValueGen::l)
        {
            Ok(obj) => obj,
            Err(e) => {
                log::error!("android_files_dir: getAbsolutePath() failed: {e:?}");
                return None;
            }
        };
        let path_str: jni::objects::JString = path_jobj.into();
        let result = match env.get_string(&path_str) {
            Ok(s) => {
                let p = PathBuf::from(String::from(s));
                log::info!("android_files_dir: Success! Path: {}", p.display());
                Some(p)
            }
            Err(e) => {
                log::error!("android_files_dir: get_string failed: {e:?}");
                None
            }
        };
        result
    }
    /// Returns the app's external files directory via JNI (`getExternalFilesDir(null)`).
    ///
    /// On Android, this resolves to a path like
    /// `/storage/emulated/0/Android/data/<package>/files/`.  The directory is
    /// readable by the user via a file manager without any special permissions.
    /// Returns `None` on any JNI error.
    ///
    /// SAFETY: the `JavaVM` pointer is process-lifetime, set up by the Android /
    /// Dioxus runtime before Rust code runs.  `JavaVM::from_raw` wraps the raw
    /// pointer without taking ownership; the JVM is not destroyed when `vm` is
    /// dropped because `jni::JavaVM::drop` is a no-op for attached VMs.
    #[cfg(target_os = "android")]
    pub fn android_external_files_dir() -> Option<std::path::PathBuf> {
        use jni::{objects::JObject, JavaVM};
        let ctx = ndk_context::android_context();
        if ctx.vm().is_null() || ctx.context().is_null() {
            log::error!("android_external_files_dir: Context or VM is NULL!");
            return None;
        }
        let vm = match unsafe { JavaVM::from_raw(ctx.vm().cast()) } {
            Ok(vm) => vm,
            Err(e) => {
                log::error!("android_external_files_dir: JavaVM::from_raw: {e:?}");
                return None;
            }
        };
        let mut env = match vm.attach_current_thread() {
            Ok(env) => env,
            Err(e) => {
                log::error!("android_external_files_dir: attach_current_thread: {e:?}");
                return None;
            }
        };
        let activity = unsafe { JObject::from_raw(ctx.context() as jni::sys::jobject) };
        let null_obj = JObject::null();
        let files_dir = match env
            .call_method(
                &activity,
                "getExternalFilesDir",
                "(Ljava/lang/String;)Ljava/io/File;",
                &[jni::objects::JValue::Object(&null_obj)],
            )
            .and_then(jni::objects::JValueGen::l)
        {
            Ok(obj) => obj,
            Err(e) => {
                log::error!("android_external_files_dir: getExternalFilesDir: {e:?}");
                return None;
            }
        };
        if files_dir.is_null() {
            log::warn!("android_external_files_dir: getExternalFilesDir returned null (external storage not mounted?)");
            return None;
        }
        let path_jobj = match env
            .call_method(&files_dir, "getAbsolutePath", "()Ljava/lang/String;", &[])
            .and_then(jni::objects::JValueGen::l)
        {
            Ok(obj) => obj,
            Err(e) => {
                log::error!("android_external_files_dir: getAbsolutePath: {e:?}");
                return None;
            }
        };
        let path_str: jni::objects::JString = path_jobj.into();
        // Bind to a local so the `JavaStr` temporary is dropped before `vm`
        // and `path_str` go out of scope (avoids E0597).
        let result = match env.get_string(&path_str) {
            Ok(s) => {
                let p = std::path::PathBuf::from(String::from(s));
                log::info!("android_external_files_dir: {}", p.display());
                Some(p)
            }
            Err(e) => {
                log::error!("android_external_files_dir: get_string: {e:?}");
                None
            }
        };
        result
    }

    /// Saves a text file to the global Android Downloads folder using MediaStore.
    ///
    /// On Android 10+ (API 29) this is the preferred way to write to public
    /// directories without requiring the broad `WRITE_EXTERNAL_STORAGE`
    /// permission.  The file is inserted into the `MediaStore.Downloads`
    /// collection.
    #[cfg(target_os = "android")]
    pub fn android_save_to_downloads(filename: &str, content: &str) -> Result<String, String> {
        use jni::{objects::JObject, JavaVM};
        let ctx = ndk_context::android_context();
        if ctx.vm().is_null() || ctx.context().is_null() {
            return Err("Android context not available".into());
        }
        let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }
            .map_err(|e| format!("JavaVM::from_raw: {e}"))?;
        let mut env = vm
            .attach_current_thread()
            .map_err(|e| format!("attach_current_thread: {e}"))?;
        let activity = unsafe { JObject::from_raw(ctx.context() as jni::sys::jobject) };

        // ContentValues values = new ContentValues();
        let values = env
            .new_object("android/content/ContentValues", "()V", &[])
            .map_err(|e| format!("new ContentValues: {e}"))?;

        let jfilename = env
            .new_string(filename)
            .map_err(|e| format!("new_string filename: {e}"))?;
        let jmime = env
            .new_string("application/json")
            .map_err(|e| format!("new_string mime: {e}"))?;
        let jrel_path = env
            .new_string("Download/")
            .map_err(|e| format!("new_string rel_path: {e}"))?;

        let jdisplay_name_key = env
            .new_string("_display_name")
            .map_err(|e| format!("new_string _display_name: {e}"))?;
        let jmime_type_key = env
            .new_string("mime_type")
            .map_err(|e| format!("new_string mime_type: {e}"))?;
        let jrelative_path_key = env
            .new_string("relative_path")
            .map_err(|e| format!("new_string relative_path: {e}"))?;

        // values.put("_display_name", filename);
        env.call_method(
            &values,
            "put",
            "(Ljava/lang/String;Ljava/lang/String;)V",
            &[
                jni::objects::JValue::from(&jdisplay_name_key),
                jni::objects::JValue::from(&jfilename),
            ],
        )
        .map_err(|e| format!("ContentValues.put name: {e}"))?;

        // values.put("mime_type", "application/json");
        env.call_method(
            &values,
            "put",
            "(Ljava/lang/String;Ljava/lang/String;)V",
            &[
                jni::objects::JValue::from(&jmime_type_key),
                jni::objects::JValue::from(&jmime),
            ],
        )
        .map_err(|e| format!("ContentValues.put mime: {e}"))?;

        // values.put("relative_path", "Download/");
        env.call_method(
            &values,
            "put",
            "(Ljava/lang/String;Ljava/lang/String;)V",
            &[
                jni::objects::JValue::from(&jrelative_path_key),
                jni::objects::JValue::from(&jrel_path),
            ],
        )
        .map_err(|e| format!("ContentValues.put path: {e}"))?;

        // ContentResolver resolver = context.getContentResolver();
        let resolver = env
            .call_method(
                &activity,
                "getContentResolver",
                "()Landroid/content/ContentResolver;",
                &[],
            )
            .map_err(|e| format!("getContentResolver: {e}"))?
            .l()
            .map_err(|e| format!("ContentResolver obj: {e}"))?;

        // Uri uri = MediaStore.Downloads.getContentUri("external");
        let jexternal = env
            .new_string("external")
            .map_err(|e| format!("new_string external: {e}"))?;
        let external_uri = env
            .call_static_method(
                "android/provider/MediaStore$Downloads",
                "getContentUri",
                "(Ljava/lang/String;)Landroid/net/Uri;",
                &[jni::objects::JValue::from(&jexternal)],
            )
            .map_err(|e| format!("MediaStore.Downloads.getContentUri: {e}"))?
            .l()
            .map_err(|e| format!("Uri obj: {e}"))?;

        // Uri fileUri = resolver.insert(external_uri, values);
        let file_uri = env
            .call_method(
                &resolver,
                "insert",
                "(Landroid/net/Uri;Landroid/content/ContentValues;)Landroid/net/Uri;",
                &[
                    jni::objects::JValue::from(&external_uri),
                    jni::objects::JValue::from(&values),
                ],
            )
            .map_err(|e| format!("resolver.insert: {e}"))?
            .l()
            .map_err(|e| format!("fileUri obj: {e}"))?;

        if file_uri.is_null() {
            return Err("MediaStore insert returned null (duplicate filename?)".into());
        }

        // OutputStream os = resolver.openOutputStream(fileUri, "w");
        let jwrite_mode = env
            .new_string("w")
            .map_err(|e| format!("new_string w: {e}"))?;
        let os = env
            .call_method(
                &resolver,
                "openOutputStream",
                "(Landroid/net/Uri;Ljava/lang/String;)Ljava/io/OutputStream;",
                &[
                    jni::objects::JValue::from(&file_uri),
                    jni::objects::JValue::from(&jwrite_mode),
                ],
            )
            .map_err(|e| format!("openOutputStream: {e}"))?
            .l()
            .map_err(|e| format!("OutputStream obj: {e}"))?;

        // os.write(content.getBytes());
        let jcontent_bytes = env
            .byte_array_from_slice(content.as_bytes())
            .map_err(|e| format!("byte_array_from_slice: {e}"))?;
        let content_bytes_obj = jni::objects::JObject::from(jcontent_bytes);
        env.call_method(
            &os,
            "write",
            "([B)V",
            &[jni::objects::JValue::from(&content_bytes_obj)],
        )
        .map_err(|e| format!("OutputStream.write: {e}"))?;

        // os.close();
        env.call_method(&os, "close", "()V", &[])
            .map_err(|e| format!("OutputStream.close: {e}"))?;

        Ok(format!("Download/{filename}"))
    }
    pub const STORE_SESSIONS: &str = "sessions";
    pub const STORE_CUSTOM_EXERCISES: &str = "custom_exercises";
    pub const STORE_EXERCISES: &str = "exercises";
    /// Name of the application data sub-directory under the OS data dir.
    #[cfg(not(test))]
    const APP_DATA_DIR_NAME: &str = "log-out";
    /// File name of the `SQLite` database within the application data directory.
    pub const DB_FILENAME: &str = "log-out.db";
    /// `SQLite` `user_version` value written on a successful schema migration.
    /// Any database with a lower version is wiped and recreated from scratch.
    const SCHEMA_VERSION: u32 = 2;
    /// Structured error type for native (`SQLite`) storage operations.
    #[derive(Debug, thiserror::Error)]
    pub enum StorageError {
        /// Unknown store name — indicates a programming error.
        #[error("Unknown store: {0}")]
        UnknownStore(String),
        /// Database-level error from `rusqlite`.
        #[error("Database error: {0}")]
        Database(#[from] rusqlite::Error),
        /// JSON serialisation / deserialisation error.
        #[error("Serialization error: {0}")]
        Serialization(#[from] serde_json::Error),
        /// OS-level I/O error (directory creation, file access, etc.).
        #[error("IO error: {0}")]
        Io(#[from] std::io::Error),
    }
    /// Returns a static SQL table name for a known store, or
    /// `Err(StorageError::UnknownStore)` for an unrecognised name.
    ///
    /// Using this function for all table-name resolution ensures that no
    /// dynamic string can ever reach a SQL statement, eliminating table-name
    /// injection as a risk regardless of call order.
    ///
    /// The return type is `&'static str`, which means the value is always one
    /// of a fixed set of compile-time string literals — never arbitrary
    /// user-controlled input.  Interpolating this value into a SQL string is
    /// therefore safe, equivalent in risk to writing the table name directly.
    fn store_table(store_name: &str) -> Result<&'static str, StorageError> {
        match store_name {
            STORE_SESSIONS => Ok("sessions"),
            STORE_CUSTOM_EXERCISES => Ok("custom_exercises"),
            STORE_EXERCISES => Ok("exercises"),
            other => Err(StorageError::UnknownStore(other.to_string())),
        }
    }
    /// Returns the application data directory, creating it if necessary.
    ///
    /// The result is computed once and cached in a process-wide `OnceLock` so
    /// that all call sites — including the `imgcache://` custom-protocol handler
    /// running on a `WebView` thread — always see the same path regardless of
    /// whether the JNI call succeeds on every thread.
    pub fn data_dir() -> PathBuf {
        // In test builds each nextest process gets its own isolated directory
        // so concurrent test runs never share the same SQLite file.
        #[cfg(test)]
        return test_data_dir();
        #[cfg(not(test))]
        {
            static DATA_DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
            DATA_DIR
                .get_or_init(|| {
                    let dir = if let Ok(custom) = std::env::var("LOGOUT_DATA_DIR") {
                        PathBuf::from(custom)
                    } else {
                        #[cfg(target_os = "android")]
                        if let Some(dir) = android_files_dir() {
                            dir
                        } else {
                            dirs::data_local_dir()
                                .unwrap_or_else(|| PathBuf::from("."))
                                .join(APP_DATA_DIR_NAME)
                        }
                        #[cfg(not(target_os = "android"))]
                        dirs::data_local_dir()
                            .unwrap_or_else(|| PathBuf::from("."))
                            .join(APP_DATA_DIR_NAME)
                    };
                    log::info!("Resolved data_dir: {}", dir.display());
                    dir
                })
                .clone()
        }
    }
    /// Returns a per-process temporary directory for test isolation.
    ///
    /// Each nextest invocation runs in its own process, so using the process
    /// ID as a unique suffix guarantees that concurrent tests never share the
    /// same `SQLite` database file.
    #[cfg(test)]
    fn test_data_dir() -> PathBuf {
        static DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
        DIR.get_or_init(|| {
            let dir = std::env::temp_dir().join(format!("logout-test-{}", std::process::id()));
            std::fs::create_dir_all(&dir)
                .expect("failed to create per-process test data directory");
            dir
        })
        .clone()
    }
    fn db_path() -> PathBuf {
        data_dir().join(DB_FILENAME)
    }
    /// Returns the directory used for storing cached exercise images.
    ///
    /// On Android, this prefers the external files directory so that the images
    /// are visible to the user and can be backed up or managed by the system
    /// gallery.  Falls back to the internal data directory if external storage
    /// is unavailable.
    pub fn images_dir() -> PathBuf {
        static IMAGES_DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
        IMAGES_DIR
            .get_or_init(|| {
                let dir = {
                    #[cfg(target_os = "android")]
                    if let Some(dir) = android_external_files_dir() {
                        dir.join("images")
                    } else {
                        data_dir().join("images")
                    }
                    #[cfg(not(target_os = "android"))]
                    data_dir().join("images")
                };
                log::info!("Resolved images_dir: {}", dir.display());
                dir
            })
            .clone()
    }
    /// Runs incremental schema migrations to bring the database up to the current version.
    ///
    /// Any schema version below 2 (including a blank database) causes all tables to be
    /// dropped and recreated fresh.  Data preservation is not attempted — the app has no
    /// established user base yet.
    ///
    /// Separated from [`open_db`] so it can be called in tests after a manual schema
    /// reset without needing to re-create the long-lived connection.
    fn apply_migration_if_needed(conn: &Connection) -> Result<(), StorageError> {
        let schema_version: u32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if schema_version < SCHEMA_VERSION {
            // Fresh install or outdated schema: drop everything and start clean.
            // SCHEMA_VERSION must match the `PRAGMA user_version` value at the end.
            conn.execute_batch(
                "DROP TABLE IF EXISTS sessions;
                 DROP TABLE IF EXISTS custom_exercises;
                 DROP TABLE IF EXISTS exercises;
                 DROP TABLE IF EXISTS config;
                 CREATE TABLE sessions (
                     id          TEXT    PRIMARY KEY,
                     data        TEXT    NOT NULL,
                     start_time  INTEGER GENERATED ALWAYS AS (
                                     CASE WHEN json_valid(data)
                                          THEN CAST(json_extract(data, '$.start_time') AS INTEGER)
                                          ELSE NULL END
                                 ) STORED,
                     end_time    INTEGER GENERATED ALWAYS AS (
                                     CASE WHEN json_valid(data)
                                          THEN CAST(json_extract(data, '$.end_time') AS INTEGER)
                                          ELSE NULL END
                                 ) STORED
                 );
                 CREATE INDEX IF NOT EXISTS idx_sessions_end_time   ON sessions(end_time)   WHERE end_time   IS NOT NULL;
                 CREATE INDEX IF NOT EXISTS idx_sessions_start_time ON sessions(start_time) WHERE start_time IS NOT NULL;
                 CREATE TABLE custom_exercises (id TEXT PRIMARY KEY, data TEXT NOT NULL);
                 CREATE TABLE exercises         (id TEXT PRIMARY KEY, data TEXT NOT NULL);
                 CREATE TABLE config            (key TEXT PRIMARY KEY, value TEXT NOT NULL);
                 PRAGMA user_version = 2;",
            )?;
        }
        Ok(())
    }
    /// Returns a mutex guard for the long-lived `SQLite` connection.
    ///
    /// The connection is opened **once** via [`std::sync::OnceLock`] and reused for the
    /// lifetime of the process.  The schema migration is also applied exactly once,
    /// inside the `OnceLock` initialiser, so it never runs on subsequent calls.
    ///
    /// If the data directory cannot be created or the database file cannot be opened on
    /// the first call, an error is returned and cached permanently — all subsequent calls
    /// will return the same error without retrying.
    fn open_db() -> Result<std::sync::MutexGuard<'static, Connection>, StorageError> {
        static DB: std::sync::OnceLock<Result<std::sync::Mutex<Connection>, String>> =
            std::sync::OnceLock::new();
        let result = DB.get_or_init(|| {
            (|| {
                let dir = data_dir();
                std::fs::create_dir_all(&dir).map_err(|e| {
                    format!(
                        "open_db: failed to create data directory {}: {e}",
                        dir.display()
                    )
                })?;
                let path = db_path();
                let conn = Connection::open(&path).map_err(|e| {
                    format!(
                        "open_db: failed to open SQLite database at {}: {e}",
                        path.display()
                    )
                })?;
                apply_migration_if_needed(&conn)
                    .map_err(|e| format!("open_db: failed to apply schema migration: {e}"))?;
                Ok(std::sync::Mutex::new(conn))
            })()
        });
        match result {
            Ok(mutex) => Ok(mutex
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)),
            Err(e) => Err(StorageError::Io(std::io::Error::other(e.clone()))),
        }
    }
    /// Re-applies the schema migration using the shared long-lived connection.
    ///
    /// Only available in tests.  Use this after manually dropping tables to simulate
    /// a fresh-database migration without needing a separate `Connection`.
    #[cfg(test)]
    pub(crate) fn apply_migration_for_testing() -> Result<(), StorageError> {
        let conn = open_db()?;
        apply_migration_if_needed(&conn)
    }
    /// Reads all items from a store, deserialising each row's JSON `data` column.
    pub fn get_all<T: DeserializeOwned>(store_name: &str) -> Result<Vec<T>, StorageError> {
        let table = store_table(store_name)?;
        let conn = open_db()?;
        let query = format!("SELECT data FROM {table}");
        let mut stmt = conn.prepare(&query)?;
        let items = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .filter_map(Result::ok)
            .filter_map(|data| {
                serde_json::from_str::<T>(&data)
                    .inspect_err(|e| log::warn!("Skipping corrupt SQLite row: {e}"))
                    .ok()
            })
            .collect();
        Ok(items)
    }
    /// Reads completed sessions ordered by `start_time` descending, with
    /// database-level `LIMIT` / `OFFSET` pagination to avoid loading the
    /// entire history into memory.
    ///
    /// Uses the `end_time` and `start_time` generated columns (and their
    /// covering indices) so `SQLite` never needs to parse JSON for filtering or
    /// sorting.
    ///
    /// `limit` and `offset` are clamped to `i64::MAX` before being passed to
    /// `SQLite`; in practice both will always be tiny (tens to hundreds).
    pub fn get_completed_sessions_paged(
        limit: usize,
        offset: usize,
    ) -> Result<Vec<crate::models::WorkoutSession>, StorageError> {
        let conn = open_db()?;
        let mut stmt = conn.prepare(
            "SELECT data FROM sessions \
             WHERE end_time IS NOT NULL \
             ORDER BY start_time DESC \
             LIMIT ?1 OFFSET ?2",
        )?;
        let limit_i64 = i64::try_from(limit).unwrap_or(i64::MAX);
        let offset_i64 = i64::try_from(offset).unwrap_or(i64::MAX);
        let items = stmt
            .query_map(params![limit_i64, offset_i64], |row| {
                row.get::<_, String>(0)
            })?
            .filter_map(Result::ok)
            .filter_map(|data| {
                serde_json::from_str::<crate::models::WorkoutSession>(&data)
                    .inspect_err(|e| log::warn!("Skipping corrupt SQLite row: {e}"))
                    .ok()
            })
            .collect();
        Ok(items)
    }
    /// Replaces the entire contents of a store with `items` in a single transaction.
    ///
    /// JSON serialisation is performed **before** the `SQLite` mutex is acquired so
    /// that expensive serialisation work never blocks other threads waiting for the
    /// lock.
    ///
    /// Uses a RAII `Transaction` guard so that the database is automatically
    /// rolled back if an error or panic occurs before `commit()`.
    pub fn store_all<T: Serialize>(store_name: &str, items: &[T]) -> Result<(), StorageError> {
        let table = store_table(store_name)?;
        // Serialise every item to (id, JSON) *before* acquiring the database mutex.
        let rows: Vec<(String, String)> = items
            .iter()
            .map(|item| {
                let val = serde_json::to_value(item)?;
                let id = val
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let data = serde_json::to_string(item)?;
                Ok((id, data))
            })
            .collect::<Result<_, serde_json::Error>>()?;
        let mut conn = open_db()?;
        let tx = conn.transaction()?;
        let delete_sql = format!("DELETE FROM {table}");
        tx.execute(&delete_sql, [])?;
        let insert_sql = format!("INSERT OR REPLACE INTO {table} (id, data) VALUES (?1, ?2)");
        for (id, data) in &rows {
            tx.execute(&insert_sql, params![id, data])?;
        }
        tx.commit()?;
        Ok(())
    }
    /// Upserts one item (identified by `id`) into a store.
    ///
    /// JSON serialisation is performed **before** the `SQLite` mutex is acquired so
    /// that serialisation work never blocks other threads waiting for the lock.
    pub fn put_item<T: Serialize>(
        store_name: &str,
        id: &str,
        item: &T,
    ) -> Result<(), StorageError> {
        let table = store_table(store_name)?;
        // Serialise outside the lock to keep the critical section minimal.
        let data = serde_json::to_string(item)?;
        let conn = open_db()?;
        let insert_sql = format!("INSERT OR REPLACE INTO {table} (id, data) VALUES (?1, ?2)");
        conn.execute(&insert_sql, params![id, data])?;
        Ok(())
    }
    /// Deletes the item with `id` from a store (no-op if absent).
    pub fn delete_item(store_name: &str, id: &str) -> Result<(), StorageError> {
        let table = store_table(store_name)?;
        let conn = open_db()?;
        let delete_sql = format!("DELETE FROM {table} WHERE id = ?1");
        conn.execute(&delete_sql, params![id])?;
        Ok(())
    }
    /// Returns the total number of rows in the `sessions` table.
    pub fn get_session_count() -> Result<usize, StorageError> {
        let conn = open_db()?;
        let count: usize = conn.query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))?;
        Ok(count)
    }
    /// Returns the string value for `key`, or `None` if absent.
    pub fn get_config_value(key: &str) -> Option<String> {
        let conn = open_db().ok()?;
        conn.query_row(
            "SELECT value FROM config WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .ok()
    }
    /// Sets `key` to `value`.  Passing an empty `value` removes the key.
    pub fn set_config_value(key: &str, value: &str) -> Result<(), StorageError> {
        let conn = open_db()?;
        if value.is_empty() {
            conn.execute("DELETE FROM config WHERE key = ?1", params![key])?;
        } else {
            conn.execute(
                "INSERT OR REPLACE INTO config (key, value) VALUES (?1, ?2)",
                params![key, value],
            )?;
        }
        Ok(())
    }
    /// Removes `key` from the config (no-op if absent).
    pub fn remove_config_value(key: &str) -> Result<(), StorageError> {
        set_config_value(key, "")
    }
    /// Load only the active (in-progress) sessions by filtering at the SQL level.
    ///
    /// More memory-efficient than [`get_all`] because completed sessions, which
    /// can represent the bulk of history, are never deserialised into Rust.
    pub fn get_active_sessions() -> Result<Vec<crate::models::WorkoutSession>, StorageError> {
        let conn = open_db()?;
        let mut stmt = conn.prepare("SELECT data FROM sessions WHERE end_time IS NULL")?;
        let items = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .filter_map(Result::ok)
            .filter_map(|data| {
                serde_json::from_str::<crate::models::WorkoutSession>(&data)
                    .inspect_err(|e| log::warn!("Skipping corrupt active session row: {e}"))
                    .ok()
            })
            .collect();
        Ok(items)
    }
    /// Compute per-exercise all-time bests using a single SQL aggregation query.
    ///
    /// Uses `json_each` to iterate the `exercise_logs` array inside each
    /// completed session row, so **no session JSON is ever deserialised into a
    /// Rust struct**.  This is the most memory-efficient path available on
    /// native.
    ///
    /// Only completed logs (those whose `end_time` field is non-null) contribute
    /// to the aggregation, matching the behaviour of
    /// [`crate::services::app_state::merge_log_into_bests`].
    pub fn compute_bests_rows() -> Result<Vec<super::BestsRow>, StorageError> {
        bests_rows_query(None)
    }
    /// Same as [`compute_bests_rows`] but restricted to the given exercise IDs.
    ///
    /// Passes the IDs as a JSON array parameter and uses a sub-select to avoid
    /// aggregating exercises the caller does not need.
    pub fn compute_bests_rows_for(
        exercise_ids: &[String],
    ) -> Result<Vec<super::BestsRow>, StorageError> {
        bests_rows_query(Some(exercise_ids))
    }
    /// Shared implementation: if `ids` is `None` aggregates all exercises;
    /// if `Some`, adds a `json_each` IN-filter on the `exercise_id` column.
    fn bests_rows_query(ids: Option<&[String]>) -> Result<Vec<super::BestsRow>, StorageError> {
        let conn = open_db()?;
        let id_filter = if ids.is_some() {
            "AND json_extract(log.value, '$.exercise_id') \
             IN (SELECT value FROM json_each(?1))"
        } else {
            ""
        };
        // CTE-based query that computes both ATH (max) values and the values from
        // the most-recently completed log per exercise in a single pass.
        let sql = format!(
            "WITH all_logs AS ( \
                 SELECT \
                     json_extract(log.value, '$.exercise_id')                           AS exercise_id, \
                     CAST(json_extract(log.value, '$.weight_hg')   AS INTEGER)          AS weight, \
                     CAST(json_extract(log.value, '$.reps')        AS INTEGER)          AS reps, \
                     CAST(json_extract(log.value, '$.distance_m')  AS INTEGER)          AS dist, \
                     CAST(json_extract(log.value, '$.end_time')    AS INTEGER) \
                   - CAST(json_extract(log.value, '$.start_time')  AS INTEGER)          AS dur, \
                     CAST(json_extract(log.value, '$.end_time')    AS INTEGER)          AS end_ts \
                 FROM sessions \
                 CROSS JOIN json_each(json_extract(data, '$.exercise_logs')) AS log \
                 WHERE end_time IS NOT NULL \
                   AND json_extract(log.value, '$.end_time') IS NOT NULL \
                   {id_filter} \
             ), \
             bests AS ( \
                 SELECT exercise_id, \
                        MAX(weight) AS max_weight, \
                        MAX(reps)   AS max_reps, \
                        MAX(dist)   AS max_dist, \
                        MAX(dur)    AS max_dur \
                 FROM all_logs \
                 GROUP BY exercise_id \
             ), \
             ranked AS ( \
                 SELECT exercise_id, weight, reps, dist, end_ts, \
                        ROW_NUMBER() OVER ( \
                            PARTITION BY exercise_id \
                            ORDER BY end_ts DESC \
                        ) AS rn \
                 FROM all_logs \
             ), \
             lasts AS ( \
                 SELECT exercise_id, \
                        weight AS last_weight, \
                        reps   AS last_reps, \
                        dist   AS last_dist, \
                        end_ts AS last_ts \
                 FROM ranked WHERE rn = 1 \
             ) \
             SELECT b.exercise_id, \
                    b.max_weight, b.max_reps, b.max_dist, b.max_dur, \
                    l.last_weight, l.last_reps, l.last_dist, l.last_ts \
             FROM bests b \
             LEFT JOIN lasts l ON b.exercise_id = l.exercise_id"
        );
        let mut stmt = conn.prepare(&sql)?;
        let map_row = |row: &rusqlite::Row<'_>| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<i64>>(1)?,
                row.get::<_, Option<i64>>(2)?,
                row.get::<_, Option<i64>>(3)?,
                row.get::<_, Option<i64>>(4)?,
                row.get::<_, Option<i64>>(5)?,
                row.get::<_, Option<i64>>(6)?,
                row.get::<_, Option<i64>>(7)?,
                row.get::<_, Option<i64>>(8)?,
            ))
        };
        let rows: Vec<super::BestsRow> = if let Some(ids) = ids {
            let json = serde_json::to_string(ids).unwrap_or_else(|_| "[]".into());
            stmt.query_map(rusqlite::params![json], map_row)?
                .filter_map(Result::ok)
                .map(bests_row_from_tuple)
                .collect()
        } else {
            stmt.query_map([], map_row)?
                .filter_map(Result::ok)
                .map(bests_row_from_tuple)
                .collect()
        };
        Ok(rows)
    }
    /// Raw SQL projection tuple returned by `bests_rows_query`.
    type BestsSqlTuple = (
        String,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
    );
    /// Convert the raw SQL tuple into a [`BestsRow`].
    fn bests_row_from_tuple(
        (exercise_id, w, r, d, dur, lw, lr, ld, lts): BestsSqlTuple,
    ) -> super::BestsRow {
        super::BestsRow {
            exercise_id,
            max_weight_hg: w.and_then(|v| u16::try_from(v).ok()),
            max_reps: r.and_then(|v| u32::try_from(v).ok()),
            max_distance_m: d.and_then(|v| u32::try_from(v).ok()),
            max_duration_s: dur.and_then(|v| u64::try_from(v).ok()),
            last_weight_hg: lw.and_then(|v| u16::try_from(v).ok()),
            last_reps: lr.and_then(|v| u32::try_from(v).ok()),
            last_distance_m: ld.and_then(|v| u32::try_from(v).ok()),
            last_log_end_time: lts.and_then(|v| u64::try_from(v).ok()),
        }
    }
    /// Global mutex that serialises all tests touching native storage within a
    /// single process.
    ///
    /// With nextest each test runs in its own process, so cross-process
    /// isolation is handled by [`test_data_dir`]'s per-process directory.
    /// This mutex provides additional within-process serialisation for the
    /// (rare) case where multiple storage tests share a process.
    ///
    /// Recovers from a poisoned mutex so a previous test failure does not
    /// cascade into every subsequent test that needs storage isolation.
    #[cfg(test)]
    pub(crate) fn test_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
        let m = LOCK.get_or_init(|| std::sync::Mutex::new(()));
        m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
/// Zero-size marker type that binds [`AsyncStorageProvider`] to the `SQLite`
/// backend exposed by [`native_storage`].
#[cfg(not(target_arch = "wasm32"))]
pub struct NativeStorage;
#[cfg(not(target_arch = "wasm32"))]
impl AsyncStorageProvider for NativeStorage {
    async fn load_completed_sessions_page(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<crate::models::WorkoutSession>, StorageError> {
        tokio::task::spawn_blocking(move || {
            native_storage::get_completed_sessions_paged(limit, offset)
        })
        .await
        .map_err(|e| StorageError::TaskPanic(e.to_string()))?
        .map_err(StorageError::from)
    }
    async fn load_active_sessions(
        &self,
    ) -> Result<Vec<crate::models::WorkoutSession>, StorageError> {
        tokio::task::spawn_blocking(native_storage::get_active_sessions)
            .await
            .map_err(|e| StorageError::TaskPanic(e.to_string()))?
            .map_err(StorageError::from)
    }
    async fn load_custom_exercises(&self) -> Result<Vec<crate::models::Exercise>, StorageError> {
        tokio::task::spawn_blocking(|| {
            native_storage::get_all::<crate::models::Exercise>(
                native_storage::STORE_CUSTOM_EXERCISES,
            )
        })
        .await
        .map_err(|e| StorageError::TaskPanic(e.to_string()))?
        .map_err(StorageError::from)
    }
    async fn compute_all_bests_rows(&self) -> Result<Vec<BestsRow>, StorageError> {
        tokio::task::spawn_blocking(native_storage::compute_bests_rows)
            .await
            .map_err(|e| StorageError::TaskPanic(e.to_string()))?
            .map_err(StorageError::from)
    }
    async fn compute_bests_rows_for_exercises(
        &self,
        exercise_ids: Vec<String>,
    ) -> Result<Vec<BestsRow>, StorageError> {
        tokio::task::spawn_blocking(move || native_storage::compute_bests_rows_for(&exercise_ids))
            .await
            .map_err(|e| StorageError::TaskPanic(e.to_string()))?
            .map_err(StorageError::from)
    }
    async fn session_count(&self) -> Result<usize, StorageError> {
        tokio::task::spawn_blocking(native_storage::get_session_count)
            .await
            .map_err(|e| StorageError::TaskPanic(e.to_string()))?
            .map_err(StorageError::from)
    }
}
#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use super::native_exercises;
    use super::native_storage;
    use crate::models::{Category, Distance, Exercise, ExerciseLog, Force, Weight, WorkoutSession};
    /// All tests that touch native storage must hold this guard.
    fn lock() -> std::sync::MutexGuard<'static, ()> {
        native_storage::test_lock()
    }
    #[test]
    fn validate_store_accepts_known_stores() {
        let _g = lock();
        assert!(native_storage::get_all::<WorkoutSession>(native_storage::STORE_SESSIONS).is_ok(),);
        assert!(
            native_storage::get_all::<Exercise>(native_storage::STORE_CUSTOM_EXERCISES).is_ok(),
        );
        assert!(native_storage::get_all::<Exercise>(native_storage::STORE_EXERCISES).is_ok(),);
    }
    #[test]
    fn validate_store_rejects_unknown_store() {
        let _g = lock();
        let result = native_storage::get_all::<WorkoutSession>("unknown_store");
        assert!(result.is_err());
        assert!(
            matches!(
                result.unwrap_err(),
                native_storage::StorageError::UnknownStore(_)
            ),
            "expected StorageError::UnknownStore for an unknown store name",
        );
    }
    #[test]
    fn data_dir_returns_a_path() {
        let _g = lock();
        let p = native_storage::data_dir();
        assert!(p.to_str().is_some());
        assert!(p.is_absolute(), "data_dir must return an absolute path");
    }
    #[test]
    fn put_and_get_session() {
        let _g = lock();
        let session = WorkoutSession {
            id: "test_put_session".into(),
            start_time: 1_000,
            end_time: None,
            exercise_logs: vec![],
            pending_exercise_ids: vec![],
            rest_start_time: None,
            current_exercise_id: None,
            current_exercise_start: None,
            paused_at: None,
            total_paused_duration: 0,
            notes: String::new(),
        };
        native_storage::put_item(native_storage::STORE_SESSIONS, &session.id, &session).unwrap();
        let loaded: Vec<WorkoutSession> =
            native_storage::get_all(native_storage::STORE_SESSIONS).unwrap();
        assert!(
            loaded.iter().any(|s| s.id == session.id),
            "saved session must be present in get_all",
        );
        native_storage::delete_item(native_storage::STORE_SESSIONS, &session.id).unwrap();
    }
    #[test]
    fn put_item_overwrites_existing() {
        let _g = lock();
        let id = "test_overwrite_session";
        let s1 = WorkoutSession {
            id: id.into(),
            start_time: 1_000,
            end_time: None,
            exercise_logs: vec![],
            pending_exercise_ids: vec![],
            rest_start_time: None,
            current_exercise_id: None,
            current_exercise_start: None,
            paused_at: None,
            total_paused_duration: 0,
            notes: String::new(),
        };
        let s2 = WorkoutSession {
            id: id.into(),
            start_time: 2_000,
            end_time: Some(3_000),
            exercise_logs: vec![],
            pending_exercise_ids: vec![],
            rest_start_time: None,
            current_exercise_id: None,
            current_exercise_start: None,
            paused_at: None,
            total_paused_duration: 0,
            notes: String::new(),
        };
        native_storage::put_item(native_storage::STORE_SESSIONS, id, &s1).unwrap();
        native_storage::put_item(native_storage::STORE_SESSIONS, id, &s2).unwrap();
        let loaded: Vec<WorkoutSession> =
            native_storage::get_all(native_storage::STORE_SESSIONS).unwrap();
        let found: Vec<_> = loaded.iter().filter(|s| s.id == id).collect();
        assert_eq!(
            found.len(),
            1,
            "there should be exactly one record after overwrite"
        );
        assert_eq!(
            found[0].start_time, 2_000,
            "record must contain the latest values"
        );
        native_storage::delete_item(native_storage::STORE_SESSIONS, id).unwrap();
    }
    #[test]
    fn delete_item_removes_session() {
        let _g = lock();
        let id = "test_delete_session";
        let session = WorkoutSession {
            id: id.into(),
            start_time: 500,
            end_time: None,
            exercise_logs: vec![],
            pending_exercise_ids: vec![],
            rest_start_time: None,
            current_exercise_id: None,
            current_exercise_start: None,
            paused_at: None,
            total_paused_duration: 0,
            notes: String::new(),
        };
        native_storage::put_item(native_storage::STORE_SESSIONS, id, &session).unwrap();
        native_storage::delete_item(native_storage::STORE_SESSIONS, id).unwrap();
        let loaded: Vec<WorkoutSession> =
            native_storage::get_all(native_storage::STORE_SESSIONS).unwrap();
        assert!(
            !loaded.iter().any(|s| s.id == id),
            "deleted session must not appear in get_all",
        );
    }
    #[test]
    fn delete_item_nonexistent_is_noop() {
        let _g = lock();
        assert!(
            native_storage::delete_item(native_storage::STORE_SESSIONS, "nonexistent_id").is_ok(),
        );
    }
    #[test]
    fn store_all_replaces_existing_records() {
        let _g = lock();
        let ex1 = make_exercise("store_all_ex1", "Exercise One");
        let ex2 = make_exercise("store_all_ex2", "Exercise Two");
        let ex3 = make_exercise("store_all_ex3", "Exercise Three");
        native_storage::store_all(native_storage::STORE_EXERCISES, &[ex1, ex2]).unwrap();
        native_storage::store_all(native_storage::STORE_EXERCISES, std::slice::from_ref(&ex3))
            .unwrap();
        let loaded: Vec<Exercise> =
            native_storage::get_all(native_storage::STORE_EXERCISES).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, ex3.id);
        native_storage::store_all::<Exercise>(native_storage::STORE_EXERCISES, &[]).unwrap();
    }
    #[test]
    fn store_all_empty_clears_store() {
        let _g = lock();
        let ex = make_exercise("store_all_clear_ex", "Clear Exercise");
        native_storage::put_item(native_storage::STORE_EXERCISES, &ex.id, &ex).unwrap();
        native_storage::store_all::<Exercise>(native_storage::STORE_EXERCISES, &[]).unwrap();
        let loaded: Vec<Exercise> =
            native_storage::get_all(native_storage::STORE_EXERCISES).unwrap();
        assert!(loaded.is_empty(), "store must be empty after store_all([])");
    }
    #[test]
    fn config_set_get_remove() {
        let _g = lock();
        let key = "test_config_key";
        native_storage::set_config_value(key, "hello").unwrap();
        assert_eq!(native_storage::get_config_value(key), Some("hello".into()));
        native_storage::remove_config_value(key).unwrap();
        assert_eq!(native_storage::get_config_value(key), None);
    }
    #[test]
    fn config_set_empty_value_removes_key() {
        let _g = lock();
        let key = "test_config_empty";
        native_storage::set_config_value(key, "value").unwrap();
        native_storage::set_config_value(key, "").unwrap();
        assert_eq!(native_storage::get_config_value(key), None);
    }
    #[test]
    fn config_get_absent_key_returns_none() {
        let _g = lock();
        assert_eq!(
            native_storage::get_config_value("definitely_not_present_key_xyz"),
            None,
        );
    }
    #[test]
    fn config_overwrite_existing_value() {
        let _g = lock();
        let key = "test_config_overwrite";
        native_storage::set_config_value(key, "first").unwrap();
        native_storage::set_config_value(key, "second").unwrap();
        assert_eq!(native_storage::get_config_value(key), Some("second".into()));
        native_storage::remove_config_value(key).unwrap();
    }
    #[test]
    fn native_exercises_store_and_retrieve() {
        let _g = lock();
        let exercises = vec![
            make_exercise("ne_ex1", "Native Ex 1"),
            make_exercise("ne_ex2", "Native Ex 2"),
        ];
        native_exercises::store_all_exercises(&exercises);
        let loaded = native_exercises::get_all_exercises();
        assert_eq!(loaded.len(), 2);
        assert!(loaded.iter().any(|e| e.id == "ne_ex1"));
        assert!(loaded.iter().any(|e| e.id == "ne_ex2"));
        native_exercises::store_all_exercises(&[]);
    }
    #[test]
    fn native_exercises_get_all_returns_empty_when_store_empty() {
        let _g = lock();
        native_exercises::store_all_exercises(&[]);
        let loaded = native_exercises::get_all_exercises();
        assert!(loaded.is_empty());
    }
    #[test]
    fn find_last_exercise_log_returns_most_recent_completed() {
        use super::super::app_state::find_last_exercise_log;
        let log1 = make_exercise_log("run", 1_000, Some(1_060));
        let log2 = make_exercise_log("run", 2_000, Some(2_060));
        let sessions = vec![
            make_session("s1", vec![log1]),
            make_session("s2", vec![log2.clone()]),
        ];
        let found = find_last_exercise_log(&sessions, "run");
        assert!(found.is_some());
        assert_eq!(found.unwrap().start_time, log2.start_time);
    }
    #[test]
    fn find_last_exercise_log_skips_incomplete_logs() {
        use super::super::app_state::find_last_exercise_log;
        let incomplete = make_exercise_log("squat", 1_000, None);
        let complete = make_exercise_log("squat", 2_000, Some(2_120));
        let sessions = vec![make_session("s1", vec![incomplete, complete.clone()])];
        let found = find_last_exercise_log(&sessions, "squat");
        assert!(found.is_some());
        assert_eq!(found.unwrap().start_time, complete.start_time);
    }
    #[test]
    fn find_last_exercise_log_returns_none_when_not_found() {
        use super::super::app_state::find_last_exercise_log;
        let sessions: Vec<WorkoutSession> = vec![];
        assert!(find_last_exercise_log(&sessions, "bench_press").is_none());
    }
    #[test]
    fn find_last_exercise_log_returns_none_when_no_matching_id() {
        use super::super::app_state::find_last_exercise_log;
        let log = make_exercise_log("squat", 1_000, Some(1_060));
        let sessions = vec![make_session("s1", vec![log])];
        assert!(find_last_exercise_log(&sessions, "deadlift").is_none());
    }
    /// Verify that the schema migration creates all required tables and leaves
    /// them in a usable state.
    ///
    /// With the `OnceLock`-based connection the migration runs exactly once on
    /// first access.  This test simulates a "fresh database" by dropping all
    /// tables via the shared connection and then calling
    /// [`native_storage::apply_migration_for_testing`] to re-apply the DDL,
    /// which checks `user_version` and recreates the tables when it is below 2.
    #[test]
    fn schema_migration_runs_on_fresh_database() {
        let _g = lock();
        native_storage::get_all::<WorkoutSession>(native_storage::STORE_SESSIONS).ok();
        {
            let db_path = native_storage::data_dir().join(native_storage::DB_FILENAME);
            if db_path.exists() {
                let conn = rusqlite::Connection::open(&db_path).unwrap();
                conn.execute_batch(
                    "DROP TABLE IF EXISTS sessions;
                     DROP TABLE IF EXISTS custom_exercises;
                     DROP TABLE IF EXISTS exercises;
                     DROP TABLE IF EXISTS config;
                     PRAGMA user_version = 0;",
                )
                .unwrap();
            }
        }
        native_storage::apply_migration_for_testing()
            .expect("schema migration must succeed on fresh DB");
        let result = native_storage::get_all::<WorkoutSession>(native_storage::STORE_SESSIONS);
        assert!(
            result.is_ok(),
            "sessions table must be accessible after migration"
        );
        let session = WorkoutSession {
            id: "schema_migration_test".into(),
            start_time: 1_000,
            end_time: None,
            exercise_logs: vec![],
            pending_exercise_ids: vec![],
            rest_start_time: None,
            current_exercise_id: None,
            current_exercise_start: None,
            paused_at: None,
            total_paused_duration: 0,
            notes: String::new(),
        };
        native_storage::put_item(native_storage::STORE_SESSIONS, &session.id, &session).unwrap();
        let loaded: Vec<WorkoutSession> =
            native_storage::get_all(native_storage::STORE_SESSIONS).unwrap();
        assert!(loaded.iter().any(|s| s.id == session.id));
        native_storage::delete_item(native_storage::STORE_SESSIONS, &session.id).unwrap();
    }
    /// Insert a row with invalid JSON directly into `SQLite` and verify that
    /// `get_all` silently skips it rather than returning an error.
    #[test]
    fn get_all_skips_corrupt_rows() {
        let _g = lock();
        let db_path = native_storage::data_dir().join(native_storage::DB_FILENAME);
        native_storage::get_all::<WorkoutSession>(native_storage::STORE_SESSIONS).unwrap();
        {
            let conn = rusqlite::Connection::open(&db_path).unwrap();
            conn.execute(
                "INSERT OR REPLACE INTO sessions (id, data) VALUES (?1, ?2)",
                rusqlite::params!["corrupt_row_id", "not {{ valid json"],
            )
            .unwrap();
        }
        let result: Result<Vec<WorkoutSession>, _> =
            native_storage::get_all(native_storage::STORE_SESSIONS);
        assert!(result.is_ok(), "get_all must not error on corrupt rows");
        let loaded = result.unwrap();
        assert!(
            !loaded.iter().any(|s| s.id == "corrupt_row_id"),
            "corrupt row must be skipped",
        );
        {
            let conn = rusqlite::Connection::open(&db_path).unwrap();
            conn.execute(
                "DELETE FROM sessions WHERE id = ?1",
                rusqlite::params!["corrupt_row_id"],
            )
            .unwrap();
        }
    }
    #[test]
    fn completed_sessions_paged_returns_only_completed() {
        let _g = lock();
        let active = WorkoutSession {
            id: "paged_active".into(),
            start_time: 5_000,
            end_time: None,
            exercise_logs: vec![],
            pending_exercise_ids: vec![],
            rest_start_time: None,
            current_exercise_id: None,
            current_exercise_start: None,
            paused_at: None,
            total_paused_duration: 0,
            notes: String::new(),
        };
        let done = WorkoutSession {
            id: "paged_done".into(),
            start_time: 4_000,
            end_time: Some(5_000),
            exercise_logs: vec![],
            pending_exercise_ids: vec![],
            rest_start_time: None,
            current_exercise_id: None,
            current_exercise_start: None,
            paused_at: None,
            total_paused_duration: 0,
            notes: String::new(),
        };
        native_storage::put_item(native_storage::STORE_SESSIONS, &active.id, &active).unwrap();
        native_storage::put_item(native_storage::STORE_SESSIONS, &done.id, &done).unwrap();
        let page = native_storage::get_completed_sessions_paged(10, 0).expect("paged query failed");
        assert!(
            page.iter().any(|s| s.id == done.id),
            "completed session must appear"
        );
        assert!(
            !page.iter().any(|s| s.id == active.id),
            "active session must be excluded",
        );
        native_storage::delete_item(native_storage::STORE_SESSIONS, &active.id).unwrap();
        native_storage::delete_item(native_storage::STORE_SESSIONS, &done.id).unwrap();
    }
    #[test]
    fn completed_sessions_paged_respects_limit_and_offset() {
        let _g = lock();
        let ids: Vec<String> = (1u64..=5).map(|i| format!("paged_limit_s{i}")).collect();
        for (i, id) in ids.iter().enumerate() {
            let s = WorkoutSession {
                id: id.clone(),
                start_time: (i as u64 + 1) * 1_000,
                end_time: Some((i as u64 + 1) * 1_000 + 60),
                exercise_logs: vec![],
                pending_exercise_ids: vec![],
                rest_start_time: None,
                current_exercise_id: None,
                current_exercise_start: None,
                paused_at: None,
                total_paused_duration: 0,
                notes: String::new(),
            };
            native_storage::put_item(native_storage::STORE_SESSIONS, &s.id, &s).unwrap();
        }
        let page1 =
            native_storage::get_completed_sessions_paged(2, 0).expect("page 1 query failed");
        assert_eq!(page1.len(), 2, "limit 2 must return 2 sessions");
        let page2 =
            native_storage::get_completed_sessions_paged(2, 2).expect("page 2 query failed");
        assert_eq!(page2.len(), 2, "offset 2 must skip first 2 sessions");
        assert!(
            page1[0].start_time > page2[0].start_time,
            "results must be ordered newest-first",
        );
        for id in &ids {
            native_storage::delete_item(native_storage::STORE_SESSIONS, id).unwrap();
        }
    }
    fn make_exercise(id: &str, name: &str) -> Exercise {
        Exercise {
            id: id.into(),
            name: name.into(),
            name_lower: String::new(),
            force: None,
            level: None,
            mechanic: None,
            equipment: None,
            primary_muscles: vec![],
            secondary_muscles: vec![],
            instructions: vec![],
            category: Category::Strength,
            images: vec![],
            i18n: None,
        }
    }
    fn make_session(id: &str, logs: Vec<ExerciseLog>) -> WorkoutSession {
        WorkoutSession {
            id: id.into(),
            start_time: 1_000,
            end_time: None,
            exercise_logs: logs,
            pending_exercise_ids: vec![],
            rest_start_time: None,
            current_exercise_id: None,
            current_exercise_start: None,
            paused_at: None,
            total_paused_duration: 0,
            notes: String::new(),
        }
    }
    fn make_exercise_log(exercise_id: &str, start: u64, end: Option<u64>) -> ExerciseLog {
        ExerciseLog {
            exercise_id: exercise_id.into(),
            exercise_name: exercise_id.into(),
            category: Category::Strength,
            start_time: start,
            end_time: end,
            weight_hg: Weight(0),
            reps: None,
            distance_m: None,
            force: Some(Force::Push),
        }
    }
    #[test]
    fn get_active_sessions_returns_only_active() {
        let _g = lock();
        let id_active = "ga_active_s1";
        let id_done = "ga_done_s1";
        let active = WorkoutSession {
            id: id_active.into(),
            start_time: 100,
            end_time: None,
            exercise_logs: vec![],
            pending_exercise_ids: vec![],
            rest_start_time: None,
            current_exercise_id: None,
            current_exercise_start: None,
            paused_at: None,
            total_paused_duration: 0,
            notes: String::new(),
        };
        let done = WorkoutSession {
            id: id_done.into(),
            start_time: 200,
            end_time: Some(300),
            exercise_logs: vec![],
            pending_exercise_ids: vec![],
            rest_start_time: None,
            current_exercise_id: None,
            current_exercise_start: None,
            paused_at: None,
            total_paused_duration: 0,
            notes: String::new(),
        };
        native_storage::put_item(native_storage::STORE_SESSIONS, id_active, &active).unwrap();
        native_storage::put_item(native_storage::STORE_SESSIONS, id_done, &done).unwrap();
        let result = native_storage::get_active_sessions().expect("get_active_sessions failed");
        assert!(
            result.iter().any(|s| s.id == id_active),
            "active session must be present"
        );
        assert!(
            !result.iter().any(|s| s.id == id_done),
            "completed session must be excluded"
        );
        native_storage::delete_item(native_storage::STORE_SESSIONS, id_active).unwrap();
        native_storage::delete_item(native_storage::STORE_SESSIONS, id_done).unwrap();
    }
    #[test]
    fn compute_bests_rows_aggregates_correctly() {
        let _g = lock();
        let id = "cb_session1";
        let log1 = ExerciseLog {
            exercise_id: "cb_ex1".into(),
            exercise_name: "Ex1".into(),
            category: Category::Strength,
            start_time: 1_000,
            end_time: Some(1_060), // duration 60s
            weight_hg: Weight(1_000),
            reps: Some(10),
            distance_m: None,
            force: None,
        };
        let log2 = ExerciseLog {
            exercise_id: "cb_ex1".into(),
            exercise_name: "Ex1".into(),
            category: Category::Strength,
            start_time: 2_000,
            end_time: Some(2_090),  // duration 90s — should win
            weight_hg: Weight(800), // lower than log1, should not win
            reps: Some(12),         // higher reps
            distance_m: Some(Distance(500)),
            force: None,
        };
        let session = WorkoutSession {
            id: id.into(),
            start_time: 1_000,
            end_time: Some(3_000),
            exercise_logs: vec![log1, log2],
            pending_exercise_ids: vec![],
            rest_start_time: None,
            current_exercise_id: None,
            current_exercise_start: None,
            paused_at: None,
            total_paused_duration: 0,
            notes: String::new(),
        };
        native_storage::put_item(native_storage::STORE_SESSIONS, id, &session).unwrap();
        let rows = native_storage::compute_bests_rows().expect("compute_bests_rows failed");
        let row = rows.iter().find(|r| r.exercise_id == "cb_ex1");
        assert!(row.is_some(), "must have a row for cb_ex1");
        let row = row.unwrap();
        assert_eq!(row.max_weight_hg, Some(1_000), "max weight must be 1000");
        assert_eq!(row.max_reps, Some(12), "max reps must be 12");
        assert_eq!(row.max_distance_m, Some(500), "max distance must be 500");
        assert_eq!(row.max_duration_s, Some(90), "max duration must be 90s");
        native_storage::delete_item(native_storage::STORE_SESSIONS, id).unwrap();
    }
}
