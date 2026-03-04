//! Platform-specific storage backends for the LogOut application.
//!
//! This module provides two storage backends that share the same logical interface:
//!
//! - **Web** (`wasm32`): IndexedDB via the `rexie` crate, serialised through an
//!   async write queue so concurrent callers never fight over read-write
//!   transactions.
//! - **Native** (Android / desktop): SQLite via `rusqlite` stored in the OS
//!   app-data directory.
//!
//! All Dioxus reactive state (signals, context helpers, mutation functions) lives
//! in the sibling [`app_state`](super::app_state) module and is re-exported here
//! for backward compatibility.

// ── Re-exports of Dioxus context / mutation helpers ──────────────────────────
// These live in `app_state` to keep that module free of storage backend code and
// to keep this module free of Dioxus hooks so its storage logic is unit-testable.
pub use super::app_state::{
    add_custom_exercise, delete_session, get_last_exercise_log, provide_app_state, save_session,
    update_custom_exercise, use_custom_exercises, use_sessions,
};

#[cfg(target_arch = "wasm32")]
pub(crate) mod idb {
    use rexie::{ObjectStore, Rexie, TransactionMode};
    use wasm_bindgen::JsValue;

    const DB_NAME: &str = "log_workout_db";
    const DB_VERSION: u32 = 2;

    pub const STORE_SESSIONS: &str = "sessions";
    pub const STORE_CUSTOM_EXERCISES: &str = "custom_exercises";
    pub const STORE_EXERCISES: &str = "exercises";

    /// Open (or create) the IndexedDB database via rexie.
    async fn open_db() -> Result<Rexie, rexie::Error> {
        Rexie::builder(DB_NAME)
            .version(DB_VERSION)
            .add_object_store(ObjectStore::new(STORE_SESSIONS).key_path("id"))
            .add_object_store(ObjectStore::new(STORE_CUSTOM_EXERCISES).key_path("id"))
            .add_object_store(ObjectStore::new(STORE_EXERCISES).key_path("id"))
            .build()
            .await
    }

    /// Put a single serialisable item into a store (upsert by key).
    pub async fn put_item<T: serde::Serialize>(store_name: &str, item: &T) -> Result<(), String> {
        let db = open_db().await.map_err(|e| format!("{e}"))?;
        let tx = db
            .transaction(&[store_name], TransactionMode::ReadWrite)
            .map_err(|e| format!("{e}"))?;
        let store = tx.store(store_name).map_err(|e| format!("{e}"))?;
        let js_val = serde_wasm_bindgen::to_value(item).map_err(|e| format!("{e}"))?;
        store.put(&js_val, None).await.map_err(|e| format!("{e}"))?;
        tx.done().await.map_err(|e| format!("{e}"))?;
        Ok(())
    }

    /// Put many serialisable items into a store in a single transaction.
    /// More efficient than calling [`put_item`] in a loop because only one
    /// database connection and one transaction are opened.
    pub async fn put_all<T: serde::Serialize>(store_name: &str, items: &[T]) -> Result<(), String> {
        let db = open_db().await.map_err(|e| format!("{e}"))?;
        let tx = db
            .transaction(&[store_name], TransactionMode::ReadWrite)
            .map_err(|e| format!("{e}"))?;
        let store = tx.store(store_name).map_err(|e| format!("{e}"))?;
        for item in items {
            let js_val = serde_wasm_bindgen::to_value(item).map_err(|e| format!("{e}"))?;
            store.put(&js_val, None).await.map_err(|e| format!("{e}"))?;
        }
        tx.done().await.map_err(|e| format!("{e}"))?;
        Ok(())
    }

    /// Delete an item from a store by its key.
    pub async fn delete_item(store_name: &str, key: &str) -> Result<(), String> {
        let db = open_db().await.map_err(|e| format!("{e}"))?;
        let tx = db
            .transaction(&[store_name], TransactionMode::ReadWrite)
            .map_err(|e| format!("{e}"))?;
        let store = tx.store(store_name).map_err(|e| format!("{e}"))?;
        store
            .delete(JsValue::from_str(key))
            .await
            .map_err(|e| format!("{e}"))?;
        tx.done().await.map_err(|e| format!("{e}"))?;
        Ok(())
    }

    /// Load all items from a store.
    pub async fn get_all<T: serde::de::DeserializeOwned>(
        store_name: &str,
    ) -> Result<Vec<T>, String> {
        let db = open_db().await.map_err(|e| format!("{e}"))?;
        let tx = db
            .transaction(&[store_name], TransactionMode::ReadOnly)
            .map_err(|e| format!("{e}"))?;
        let store = tx.store(store_name).map_err(|e| format!("{e}"))?;
        let js_values = store
            .get_all(None, None)
            .await
            .map_err(|e| format!("{e}"))?;

        let mut items = Vec::new();
        for (i, js_val) in js_values.into_iter().enumerate() {
            match serde_wasm_bindgen::from_value::<T>(js_val) {
                Ok(item) => items.push(item),
                Err(e) => log::warn!("Skipping corrupt IndexedDB entry at index {}: {}", i, e),
            }
        }
        Ok(items)
    }
}

// Async write queue for IndexedDB (wasm32 only)
//
// Serialises all write operations (put / delete) so that concurrent callers
// never open competing read-write transactions on the same object store, which
// IndexedDB would otherwise serialise at the browser level anyway but could
// cause transaction aborts under some circumstances
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
        PutSession(WorkoutSession, Signal<Option<String>>),
        DeleteSession(String, Signal<Option<String>>),
        PutExercise(Exercise, Signal<Option<String>>),
        // DeleteExercise(String, Signal<Option<String>>), // Not supported for now
    }

    thread_local! {
        /// (draining, pending_ops)
        static QUEUE: RefCell<(bool, VecDeque<IdbOp>)> =
            RefCell::new((false, VecDeque::new()));
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

    async fn drain() {
        loop {
            let op = QUEUE.with(|q| q.borrow_mut().1.pop_front());
            match op {
                None => {
                    QUEUE.with(|q| q.borrow_mut().0 = false);
                    break;
                }
                Some(IdbOp::PutSession(s, mut toast)) => {
                    if let Err(e) = idb::put_item(idb::STORE_SESSIONS, &s).await {
                        log::error!("IDB queue: failed to put session {}: {e}", s.id);
                        toast.set(Some(format!("⚠️ Failed to save session: {e}")));
                    }
                }
                Some(IdbOp::DeleteSession(id, mut toast)) => {
                    if let Err(e) = idb::delete_item(idb::STORE_SESSIONS, &id).await {
                        log::error!("IDB queue: failed to delete session {id}: {e}");
                        toast.set(Some(format!("⚠️ Failed to delete session: {e}")));
                    }
                }
                Some(IdbOp::PutExercise(ex, mut toast)) => {
                    if let Err(e) = idb::put_item(idb::STORE_CUSTOM_EXERCISES, &ex).await {
                        log::error!("IDB queue: failed to put exercise {}: {e}", ex.id);
                        toast.set(Some(format!("⚠️ Failed to save exercise: {e}")));
                    }
                }
            }
        }
    }
}

// ──────────────────────────────────────────
// Exercise storage helpers (used by exercise_db)
// ──────────────────────────────────────────

/// IndexedDB-backed exercise storage for the web platform.
#[cfg(target_arch = "wasm32")]
pub mod idb_exercises {
    use super::idb;
    use crate::models::Exercise;

    /// Retrieve all cached exercises from the IndexedDB exercises store.
    pub async fn get_all_exercises() -> Result<Vec<Exercise>, String> {
        idb::get_all::<Exercise>(idb::STORE_EXERCISES).await
    }

    /// Persist `exercises` to the IndexedDB exercises store in a single transaction.
    pub async fn store_all_exercises(exercises: &[Exercise]) {
        if let Err(e) = idb::put_all(idb::STORE_EXERCISES, exercises).await {
            log::error!("Failed to store exercises in IndexedDB: {e}");
        }
    }
}

/// File-backed exercise storage for native platforms (Android / desktop).
#[cfg(not(target_arch = "wasm32"))]
pub mod native_exercises {
    use super::native_storage;
    use crate::models::Exercise;

    /// Retrieve all cached exercises from the SQLite exercises store.
    pub fn get_all_exercises() -> Vec<Exercise> {
        native_storage::get_all::<Exercise>(native_storage::STORE_EXERCISES).unwrap_or_default()
    }

    /// Persist `exercises` to the SQLite exercises store.
    pub fn store_all_exercises(exercises: &[Exercise]) {
        if let Err(e) = native_storage::store_all(native_storage::STORE_EXERCISES, exercises) {
            log::error!("Failed to store exercises: {e}");
        }
    }
}

// Async write queue for SQLite (native only)
//
// Mirrors the wasm32 `idb_queue` pattern: serialises all write operations
// so that concurrent Dioxus tasks never fight over the same DB file.
// Uses Dioxus `spawn` (backed by the single-threaded tokio runtime on native)
// so that in-flight writes are not cancelled when a component unmounts.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod native_queue {
    use super::native_storage;
    use crate::models::{Exercise, WorkoutSession};
    use dioxus::prelude::WritableExt;
    use dioxus::signals::Signal;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    /// A pending write operation, including the toast signal for error reporting.
    pub enum NativeOp {
        PutSession(WorkoutSession, Signal<Option<String>>),
        DeleteSession(String, Signal<Option<String>>),
        PutExercise(Exercise, Signal<Option<String>>),
    }

    thread_local! {
        /// (draining, pending_ops)
        static QUEUE: RefCell<(bool, VecDeque<NativeOp>)> =
            const { RefCell::new((false, VecDeque::new())) };
    }

    /// Enqueue a write operation. If no drain is currently running, starts one.
    pub fn enqueue(op: NativeOp) {
        QUEUE.with(|q| {
            let mut q = q.borrow_mut();
            q.1.push_back(op);
            if !q.0 {
                q.0 = true;
                dioxus::prelude::spawn(drain());
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
                Some(NativeOp::PutSession(s, mut toast)) => {
                    if let Err(e) =
                        native_storage::put_item(native_storage::STORE_SESSIONS, &s.id, &s)
                    {
                        log::error!("Native queue: failed to put session {}: {e}", s.id);
                        toast.set(Some(format!("⚠️ Failed to save session: {e}")));
                    }
                }
                Some(NativeOp::DeleteSession(id, mut toast)) => {
                    if let Err(e) =
                        native_storage::delete_item(native_storage::STORE_SESSIONS, &id)
                    {
                        log::error!("Native queue: failed to delete session {id}: {e}");
                        toast.set(Some(format!("⚠️ Failed to delete session: {e}")));
                    }
                }
                Some(NativeOp::PutExercise(ex, mut toast)) => {
                    if let Err(e) = native_storage::put_item(
                        native_storage::STORE_CUSTOM_EXERCISES,
                        &ex.id,
                        &ex,
                    ) {
                        log::error!("Native queue: failed to put exercise {}: {e}", ex.id);
                        toast.set(Some(format!("⚠️ Failed to save exercise: {e}")));
                    }
                }
            }
        }
    }
}

// ──────────────────────────────────────────
// Native SQLite-based storage (non-wasm platforms: Android / desktop)
// ──────────────────────────────────────────

/// SQLite-backed storage for Android and desktop builds.
///
/// A single `log-workout.db` SQLite database file is kept inside the app-
/// specific data directory (`dirs::data_local_dir()/log-workout/`).
/// Each "store" maps to a table with columns `id TEXT PRIMARY KEY, data TEXT`.
/// A separate `config` table holds arbitrary key/value string pairs.
///
/// On first launch, existing JSON files from the old file-based backend are
/// automatically migrated into the database and then deleted.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod native_storage {
    use rusqlite::{params, Connection};
    use serde::{de::DeserializeOwned, Serialize};
    use std::path::PathBuf;

    pub const STORE_SESSIONS: &str = "sessions";
    pub const STORE_CUSTOM_EXERCISES: &str = "custom_exercises";
    pub const STORE_EXERCISES: &str = "exercises";

    const KNOWN_STORES: &[&str] = &[STORE_SESSIONS, STORE_CUSTOM_EXERCISES, STORE_EXERCISES];

    /// Validates `store_name` against the known store constants to prevent SQL
    /// injection from unexpected callers.  Returns `Err` when the name is unknown.
    fn validate_store(store_name: &str) -> Result<(), String> {
        if KNOWN_STORES.contains(&store_name) {
            Ok(())
        } else {
            Err(format!("Unknown store: {store_name}"))
        }
    }

    /// Returns the application data directory, creating it if necessary.
    pub fn data_dir() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("log-workout")
    }

    fn db_path() -> PathBuf {
        data_dir().join("log-workout.db")
    }

    /// Opens (or creates) the SQLite database and ensures all required tables exist.
    /// Uses `PRAGMA user_version` to run the schema DDL only once (when the DB is
    /// first created), rather than on every operation.
    fn open_db() -> Result<Connection, String> {
        std::fs::create_dir_all(data_dir()).map_err(|e| e.to_string())?;
        let conn = Connection::open(db_path()).map_err(|e| e.to_string())?;
        let schema_version: u32 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        if schema_version == 0 {
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS sessions (id TEXT PRIMARY KEY, data TEXT NOT NULL);
                 CREATE TABLE IF NOT EXISTS custom_exercises (id TEXT PRIMARY KEY, data TEXT NOT NULL);
                 CREATE TABLE IF NOT EXISTS exercises (id TEXT PRIMARY KEY, data TEXT NOT NULL);
                 CREATE TABLE IF NOT EXISTS config (key TEXT PRIMARY KEY, value TEXT NOT NULL);
                 PRAGMA user_version = 1;",
            )
            .map_err(|e| e.to_string())?;
        }
        Ok(conn)
    }

    /// Reads all items from a store, deserialising each row's JSON `data` column.
    pub fn get_all<T: DeserializeOwned>(store_name: &str) -> Result<Vec<T>, String> {
        validate_store(store_name)?;
        let conn = open_db()?;
        let mut stmt = conn
            .prepare(&format!("SELECT data FROM {store_name}"))
            .map_err(|e| e.to_string())?;
        let items = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .filter_map(|data| {
                serde_json::from_str::<T>(&data)
                    .inspect_err(|e| log::warn!("Skipping corrupt SQLite row: {e}"))
                    .ok()
            })
            .collect();
        Ok(items)
    }

    /// Replaces the entire contents of a store with `items` in a single transaction.
    pub fn store_all<T: Serialize>(store_name: &str, items: &[T]) -> Result<(), String> {
        validate_store(store_name)?;
        let conn = open_db()?;
        let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
        tx.execute(&format!("DELETE FROM {store_name}"), [])
            .map_err(|e| e.to_string())?;
        for item in items {
            let val = serde_json::to_value(item).map_err(|e| e.to_string())?;
            let id = val
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let data = serde_json::to_string(item).map_err(|e| e.to_string())?;
            tx.execute(
                &format!("INSERT OR REPLACE INTO {store_name} (id, data) VALUES (?1, ?2)"),
                params![id, data],
            )
            .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())
    }

    /// Upserts one item (identified by `id`) into a store.
    pub fn put_item<T: Serialize>(store_name: &str, id: &str, item: &T) -> Result<(), String> {
        validate_store(store_name)?;
        let conn = open_db()?;
        let data = serde_json::to_string(item).map_err(|e| e.to_string())?;
        conn.execute(
            &format!("INSERT OR REPLACE INTO {store_name} (id, data) VALUES (?1, ?2)"),
            params![id, data],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Deletes the item with `id` from a store (no-op if absent).
    pub fn delete_item(store_name: &str, id: &str) -> Result<(), String> {
        validate_store(store_name)?;
        let conn = open_db()?;
        conn.execute(
            &format!("DELETE FROM {store_name} WHERE id = ?1"),
            params![id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ── Config (key/value pairs) ──────────────────────────────────────────────

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
    pub fn set_config_value(key: &str, value: &str) -> Result<(), String> {
        let conn = open_db()?;
        if value.is_empty() {
            conn.execute("DELETE FROM config WHERE key = ?1", params![key])
                .map_err(|e| e.to_string())?;
        } else {
            conn.execute(
                "INSERT OR REPLACE INTO config (key, value) VALUES (?1, ?2)",
                params![key, value],
            )
            .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Removes `key` from the config (no-op if absent).
    pub fn remove_config_value(key: &str) -> Result<(), String> {
        set_config_value(key, "")
    }

    /// Global mutex that serialises all tests touching native storage.
    ///
    /// Tests in any module that read or write native-storage config or data
    /// should hold this guard for their duration to prevent data races.
    /// Recovers from a poisoned mutex so a previous test failure does not
    /// cascade into every subsequent test that needs storage isolation.
    #[cfg(test)]
    pub(crate) fn test_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
        let m = LOCK.get_or_init(|| std::sync::Mutex::new(()));
        m.lock().unwrap_or_else(|e| e.into_inner())
    }
}

// ──────────────────────────────────────────
// Unit tests for native_storage and native_exercises
// ──────────────────────────────────────────

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use super::native_exercises;
    use super::native_storage;
    use crate::models::{Category, Exercise, ExerciseLog, Force, WorkoutSession, DATA_VERSION};

    /// All tests that touch native storage must hold this guard.
    fn lock() -> std::sync::MutexGuard<'static, ()> {
        native_storage::test_lock()
    }

    // ── validate_store ────────────────────────────────────────────────────────

    #[test]
    fn validate_store_accepts_known_stores() {
        let _g = lock();
        assert!(native_storage::get_all::<WorkoutSession>(native_storage::STORE_SESSIONS).is_ok());
        assert!(
            native_storage::get_all::<Exercise>(native_storage::STORE_CUSTOM_EXERCISES).is_ok()
        );
        assert!(native_storage::get_all::<Exercise>(native_storage::STORE_EXERCISES).is_ok());
    }

    #[test]
    fn validate_store_rejects_unknown_store() {
        let _g = lock();
        let result = native_storage::get_all::<WorkoutSession>("unknown_store");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown store"));
    }

    // ── data_dir ──────────────────────────────────────────────────────────────

    #[test]
    fn data_dir_returns_a_path() {
        let _g = lock();
        let p = native_storage::data_dir();
        assert!(p.to_str().is_some());
        assert!(p.ends_with("log-workout"));
    }

    // ── put_item / get_all / delete_item ─────────────────────────────────────

    #[test]
    fn put_and_get_session() {
        let _g = lock();
        let session = WorkoutSession {
            id: "test_put_session".into(),
            start_time: 1_000,
            end_time: None,
            exercise_logs: vec![],
            version: DATA_VERSION,
            pending_exercise_ids: vec![],
            rest_start_time: None,
            current_exercise_id: None,
            current_exercise_start: None,
        };
        native_storage::put_item(native_storage::STORE_SESSIONS, &session.id, &session).unwrap();
        let loaded: Vec<WorkoutSession> =
            native_storage::get_all(native_storage::STORE_SESSIONS).unwrap();
        assert!(
            loaded.iter().any(|s| s.id == session.id),
            "saved session must be present in get_all"
        );
        // Clean up
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
            version: DATA_VERSION,
            pending_exercise_ids: vec![],
            rest_start_time: None,
            current_exercise_id: None,
            current_exercise_start: None,
        };
        let s2 = WorkoutSession {
            id: id.into(),
            start_time: 2_000,
            end_time: Some(3_000),
            exercise_logs: vec![],
            version: DATA_VERSION,
            pending_exercise_ids: vec![],
            rest_start_time: None,
            current_exercise_id: None,
            current_exercise_start: None,
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
        // Clean up
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
            version: DATA_VERSION,
            pending_exercise_ids: vec![],
            rest_start_time: None,
            current_exercise_id: None,
            current_exercise_start: None,
        };
        native_storage::put_item(native_storage::STORE_SESSIONS, id, &session).unwrap();
        native_storage::delete_item(native_storage::STORE_SESSIONS, id).unwrap();
        let loaded: Vec<WorkoutSession> =
            native_storage::get_all(native_storage::STORE_SESSIONS).unwrap();
        assert!(
            !loaded.iter().any(|s| s.id == id),
            "deleted session must not appear in get_all"
        );
    }

    #[test]
    fn delete_item_nonexistent_is_noop() {
        let _g = lock();
        // Deleting a key that doesn't exist must not return an error.
        assert!(
            native_storage::delete_item(native_storage::STORE_SESSIONS, "nonexistent_id").is_ok()
        );
    }

    // ── store_all ─────────────────────────────────────────────────────────────

    #[test]
    fn store_all_replaces_existing_records() {
        let _g = lock();
        let ex1 = make_exercise("store_all_ex1", "Exercise One");
        let ex2 = make_exercise("store_all_ex2", "Exercise Two");
        let ex3 = make_exercise("store_all_ex3", "Exercise Three");

        native_storage::store_all(native_storage::STORE_EXERCISES, &[ex1, ex2]).unwrap();
        native_storage::store_all(native_storage::STORE_EXERCISES, &[ex3.clone()]).unwrap();

        let loaded: Vec<Exercise> =
            native_storage::get_all(native_storage::STORE_EXERCISES).unwrap();
        // Only ex3 should remain – store_all deletes and replaces.
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, ex3.id);
        // Clean up
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

    // ── config key/value ─────────────────────────────────────────────────────

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
        // Setting to "" is equivalent to remove.
        native_storage::set_config_value(key, "").unwrap();
        assert_eq!(native_storage::get_config_value(key), None);
    }

    #[test]
    fn config_get_absent_key_returns_none() {
        let _g = lock();
        assert_eq!(
            native_storage::get_config_value("definitely_not_present_key_xyz"),
            None
        );
    }

    #[test]
    fn config_overwrite_existing_value() {
        let _g = lock();
        let key = "test_config_overwrite";
        native_storage::set_config_value(key, "first").unwrap();
        native_storage::set_config_value(key, "second").unwrap();
        assert_eq!(native_storage::get_config_value(key), Some("second".into()));
        // Clean up
        native_storage::remove_config_value(key).unwrap();
    }

    // ── native_exercises ─────────────────────────────────────────────────────

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
        // Clean up
        native_exercises::store_all_exercises(&[]);
    }

    #[test]
    fn native_exercises_get_all_returns_empty_when_store_empty() {
        let _g = lock();
        native_exercises::store_all_exercises(&[]);
        let loaded = native_exercises::get_all_exercises();
        assert!(loaded.is_empty());
    }

    // ── find_last_exercise_log (pure helper) ──────────────────────────────────

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
        // Should find log2 (most recent) because sessions are iterated in reverse.
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

    // ── helpers ───────────────────────────────────────────────────────────────

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
        }
    }

    fn make_session(id: &str, logs: Vec<ExerciseLog>) -> WorkoutSession {
        WorkoutSession {
            id: id.into(),
            start_time: 1_000,
            end_time: None,
            exercise_logs: logs,
            version: DATA_VERSION,
            pending_exercise_ids: vec![],
            rest_start_time: None,
            current_exercise_id: None,
            current_exercise_start: None,
        }
    }

    fn make_exercise_log(exercise_id: &str, start: u64, end: Option<u64>) -> ExerciseLog {
        ExerciseLog {
            exercise_id: exercise_id.into(),
            exercise_name: exercise_id.into(),
            category: Category::Strength,
            start_time: start,
            end_time: end,
            weight_hg: None,
            reps: None,
            distance_m: None,
            force: Some(Force::Push),
        }
    }
}
