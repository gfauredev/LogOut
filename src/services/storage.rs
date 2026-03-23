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
/// Returns an empty `Vec` and logs an error when storage access fails.
#[cfg_attr(not(target_arch = "wasm32"), allow(clippy::unused_async))]
pub async fn load_completed_sessions_page(
    limit: usize,
    offset: usize,
) -> Vec<crate::models::WorkoutSession> {
    #[cfg(target_arch = "wasm32")]
    {
        match idb::get_all::<crate::models::WorkoutSession>(idb::STORE_SESSIONS).await {
            Ok(mut sessions) => {
                sessions.retain(|s| !s.is_active());
                sessions.sort_by(|a, b| b.start_time.cmp(&a.start_time));
                sessions.into_iter().skip(offset).take(limit).collect()
            }
            Err(e) => {
                log::error!("Failed to load sessions from IDB: {e}");
                vec![]
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        match native_storage::get_completed_sessions_paged(limit, offset) {
            Ok(sessions) => sessions,
            Err(e) => {
                log::error!("Failed to load completed sessions: {e}");
                vec![]
            }
        }
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
    /// Remove all items from a store.
    pub async fn clear_all(store_name: &str) -> Result<(), String> {
        let db = open_db().await.map_err(|e| format!("{e}"))?;
        let tx = db
            .transaction(&[store_name], TransactionMode::ReadWrite)
            .map_err(|e| format!("{e}"))?;
        let store = tx.store(store_name).map_err(|e| format!("{e}"))?;
        store.clear().await.map_err(|e| format!("{e}"))?;
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
                Err(e) => {
                    log::warn!("Skipping corrupt IndexedDB entry at index {i}: {e}")
                }
            }
        }
        Ok(items)
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
        PutSession(WorkoutSession, Signal<Option<String>>),
        DeleteSession(String, Signal<Option<String>>),
        PutExercise(Exercise, Signal<Option<String>>),
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
    pub async fn get_all_exercises() -> Result<Vec<Exercise>, String> {
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
    pub async fn store_image(image_key: &str, bytes: &[u8]) -> Result<(), String> {
        let db = super::idb::open_db().await.map_err(|e| format!("{e}"))?;
        let tx = db
            .transaction(&[super::idb::STORE_IMAGES], TransactionMode::ReadWrite)
            .map_err(|e| format!("{e}"))?;
        let store = tx
            .store(super::idb::STORE_IMAGES)
            .map_err(|e| format!("{e}"))?;
        let arr = Uint8Array::from(bytes);
        let key = JsValue::from_str(image_key);
        store
            .put(&arr.into(), Some(&key))
            .await
            .map_err(|e| format!("{e}"))?;
        tx.done().await.map_err(|e| format!("{e}"))?;
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
/// specific data directory (`dirs::data_local_dir()/log-out/`).
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
    #[cfg(target_os = "android")]
    static ANDROID_DATA_DIR: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    #[cfg(target_os = "android")]
    #[no_mangle]
    pub extern "C" fn Java_dev_dioxus_main_MainActivity_setDataDir(
        mut env: jni::JNIEnv,
        _class: jni::objects::JClass,
        data_dir: jni::objects::JString,
    ) {
        let dir: String = match env.get_string(&data_dir) {
            Ok(s) => s.into(),
            Err(e) => {
                log::error!("setDataDir: failed to read Java data_dir string: {e:?}");
                return;
            }
        };
        let _ = ANDROID_DATA_DIR.set(dir);
    }
    pub const STORE_SESSIONS: &str = "sessions";
    pub const STORE_CUSTOM_EXERCISES: &str = "custom_exercises";
    pub const STORE_EXERCISES: &str = "exercises";
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
    pub fn data_dir() -> PathBuf {
        #[cfg(target_os = "android")]
        {
            if let Some(dir) = ANDROID_DATA_DIR.get() {
                return PathBuf::from(dir);
            }
        }
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("log-out")
    }
    fn db_path() -> PathBuf {
        data_dir().join("log-out.db")
    }
    /// Runs incremental schema migrations to bring the database up to the current version.
    ///
    /// | version | change |
    /// |---------|--------|
    /// | 0 → 2  | fresh install: create all tables with `start_time`/`end_time` generated columns and covering indices on `sessions` |
    /// | 1 → 2  | existing install: recreate `sessions` with generated columns and indices (rename → create → copy → drop) |
    ///
    /// Separated from [`open_db`] so it can be called in tests after a manual schema
    /// reset without needing to re-create the long-lived connection.
    fn apply_migration_if_needed(conn: &Connection) -> Result<(), StorageError> {
        let schema_version: u32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if schema_version == 0 {
            // Fresh install: create all tables with generated columns from the start.
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS sessions (
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
                 CREATE TABLE IF NOT EXISTS custom_exercises (id TEXT PRIMARY KEY, data TEXT NOT NULL);
                 CREATE TABLE IF NOT EXISTS exercises         (id TEXT PRIMARY KEY, data TEXT NOT NULL);
                 CREATE TABLE IF NOT EXISTS config            (key TEXT PRIMARY KEY, value TEXT NOT NULL);
                 PRAGMA user_version = 2;",
            )?;
        }
        if schema_version == 1 {
            // Existing install: add generated columns to sessions via rename→create→copy→drop.
            conn.execute_batch(
                "ALTER TABLE sessions RENAME TO sessions_v1;
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
                 INSERT INTO sessions(id, data) SELECT id, data FROM sessions_v1;
                 DROP TABLE sessions_v1;
                 CREATE INDEX IF NOT EXISTS idx_sessions_end_time   ON sessions(end_time)   WHERE end_time   IS NOT NULL;
                 CREATE INDEX IF NOT EXISTS idx_sessions_start_time ON sessions(start_time) WHERE start_time IS NOT NULL;
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
    /// # Panics
    ///
    /// Panics on the first call if the data directory cannot be created or the database
    /// file cannot be opened.  These are considered fatal, unrecoverable errors.
    fn open_db() -> std::sync::MutexGuard<'static, Connection> {
        static DB: std::sync::OnceLock<std::sync::Mutex<Connection>> = std::sync::OnceLock::new();
        let mutex = DB.get_or_init(|| {
            std::fs::create_dir_all(data_dir()).expect("open_db: failed to create data directory");
            let conn =
                Connection::open(db_path()).expect("open_db: failed to open SQLite database");
            apply_migration_if_needed(&conn).expect("open_db: failed to apply schema migration");
            std::sync::Mutex::new(conn)
        });
        mutex
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    /// Re-applies the schema migration using the shared long-lived connection.
    ///
    /// Only available in tests.  Use this after manually dropping tables to simulate
    /// a fresh-database migration without needing a separate `Connection`.
    #[cfg(test)]
    pub(crate) fn apply_migration_for_testing() -> Result<(), StorageError> {
        let conn = open_db();
        apply_migration_if_needed(&conn)
    }
    /// Reads all items from a store, deserialising each row's JSON `data` column.
    pub fn get_all<T: DeserializeOwned>(store_name: &str) -> Result<Vec<T>, StorageError> {
        let table = store_table(store_name)?;
        let conn = open_db();
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
        let conn = open_db();
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
    /// Uses a RAII `Transaction` guard so that the database is automatically
    /// rolled back if an error or panic occurs before `commit()`.
    pub fn store_all<T: Serialize>(store_name: &str, items: &[T]) -> Result<(), StorageError> {
        let table = store_table(store_name)?;
        let mut conn = open_db();
        let tx = conn.transaction()?;
        let delete_sql = format!("DELETE FROM {table}");
        tx.execute(&delete_sql, [])?;
        let insert_sql = format!("INSERT OR REPLACE INTO {table} (id, data) VALUES (?1, ?2)");
        for item in items {
            let val = serde_json::to_value(item)?;
            let id = val
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let data = serde_json::to_string(item)?;
            tx.execute(&insert_sql, params![id, data])?;
        }
        tx.commit()?;
        Ok(())
    }
    /// Upserts one item (identified by `id`) into a store.
    pub fn put_item<T: Serialize>(
        store_name: &str,
        id: &str,
        item: &T,
    ) -> Result<(), StorageError> {
        let table = store_table(store_name)?;
        let conn = open_db();
        let data = serde_json::to_string(item)?;
        let insert_sql = format!("INSERT OR REPLACE INTO {table} (id, data) VALUES (?1, ?2)");
        conn.execute(&insert_sql, params![id, data])?;
        Ok(())
    }
    /// Deletes the item with `id` from a store (no-op if absent).
    pub fn delete_item(store_name: &str, id: &str) -> Result<(), StorageError> {
        let table = store_table(store_name)?;
        let conn = open_db();
        let delete_sql = format!("DELETE FROM {table} WHERE id = ?1");
        conn.execute(&delete_sql, params![id])?;
        Ok(())
    }
    /// Returns the string value for `key`, or `None` if absent.
    pub fn get_config_value(key: &str) -> Option<String> {
        let conn = open_db();
        conn.query_row(
            "SELECT value FROM config WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .ok()
    }
    /// Sets `key` to `value`.  Passing an empty `value` removes the key.
    pub fn set_config_value(key: &str, value: &str) -> Result<(), StorageError> {
        let conn = open_db();
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
        m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
/// Trait that abstracts synchronous key-value storage operations.
///
/// Both the `get_all` / `put_item` / `delete_item` / `store_all` family of
/// operations on the native (`SQLite`) backend implement this interface.
/// Defining a shared trait decouples business logic from the concrete backend
/// and makes the storage layer straightforward to substitute in tests or to
/// extend with new implementations in the future.
///
/// The web (`IndexedDB`) backend is inherently asynchronous and therefore does
/// not implement this synchronous trait; it exposes an equivalent async API
/// through the [`idb`] module.
#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub trait StorageProvider {
    /// The error type returned by all operations on this provider.
    type Error: std::fmt::Display + std::fmt::Debug;
    /// Load all items from `store_name`, skipping rows that fail to deserialise.
    fn get_all<T: serde::de::DeserializeOwned>(store_name: &str) -> Result<Vec<T>, Self::Error>;
    /// Upsert one item (identified by `id`) into `store_name`.
    fn put_item<T: serde::Serialize>(
        store_name: &str,
        id: &str,
        item: &T,
    ) -> Result<(), Self::Error>;
    /// Delete the item with `id` from `store_name` (no-op if absent).
    fn delete_item(store_name: &str, id: &str) -> Result<(), Self::Error>;
    /// Replace all items in `store_name` with `items` in a single transaction.
    fn store_all<T: serde::Serialize>(store_name: &str, items: &[T]) -> Result<(), Self::Error>;
}
/// Zero-size marker type that binds [`StorageProvider`] to the `SQLite`
/// backend exposed by [`native_storage`].
#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub struct NativeStorage;
#[cfg(not(target_arch = "wasm32"))]
impl StorageProvider for NativeStorage {
    type Error = native_storage::StorageError;
    fn get_all<T: serde::de::DeserializeOwned>(store_name: &str) -> Result<Vec<T>, Self::Error> {
        native_storage::get_all(store_name)
    }
    fn put_item<T: serde::Serialize>(
        store_name: &str,
        id: &str,
        item: &T,
    ) -> Result<(), Self::Error> {
        native_storage::put_item(store_name, id, item)
    }
    fn delete_item(store_name: &str, id: &str) -> Result<(), Self::Error> {
        native_storage::delete_item(store_name, id)
    }
    fn store_all<T: serde::Serialize>(store_name: &str, items: &[T]) -> Result<(), Self::Error> {
        native_storage::store_all(store_name, items)
    }
}
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
        assert!(p.ends_with("log-out"));
    }
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
            paused_at: None,
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
            version: DATA_VERSION,
            pending_exercise_ids: vec![],
            rest_start_time: None,
            current_exercise_id: None,
            current_exercise_start: None,
            paused_at: None,
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
            paused_at: None,
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
            version: DATA_VERSION,
            pending_exercise_ids: vec![],
            rest_start_time: None,
            current_exercise_id: None,
            current_exercise_start: None,
            paused_at: None,
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
    /// which checks `user_version` and recreates the tables when it is 0.
    #[test]
    fn schema_migration_runs_on_fresh_database() {
        let _g = lock();
        native_storage::get_all::<WorkoutSession>(native_storage::STORE_SESSIONS).ok();
        {
            let db_path = native_storage::data_dir().join("log-out.db");
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
            version: DATA_VERSION,
            pending_exercise_ids: vec![],
            rest_start_time: None,
            current_exercise_id: None,
            current_exercise_start: None,
            paused_at: None,
        };
        native_storage::put_item(native_storage::STORE_SESSIONS, &session.id, &session).unwrap();
        let loaded: Vec<WorkoutSession> =
            native_storage::get_all(native_storage::STORE_SESSIONS).unwrap();
        assert!(loaded.iter().any(|s| s.id == session.id));
        native_storage::delete_item(native_storage::STORE_SESSIONS, &session.id).unwrap();
    }
    /// Verify that the v1→v2 migration (adding generated columns to an existing
    /// sessions table) preserves data and leaves the paged query working.
    #[test]
    fn schema_migration_v1_to_v2_preserves_data() {
        let _g = lock();
        let db_path = native_storage::data_dir().join("log-out.db");
        // Force the database into schema version 1 by recreating sessions without
        // generated columns, copying any live rows, then setting user_version = 1.
        {
            let conn = rusqlite::Connection::open(&db_path).unwrap();
            conn.execute_batch(
                "DROP TABLE IF EXISTS sessions;
                 DROP TABLE IF EXISTS sessions_v1;
                 CREATE TABLE sessions (id TEXT PRIMARY KEY, data TEXT NOT NULL);
                 INSERT INTO sessions(id, data)
                     VALUES ('v1_s1', '{\"id\":\"v1_s1\",\"start_time\":1000,\"end_time\":2000,\
                              \"exercise_logs\":[],\"version\":1,\"pending_exercise_ids\":[]}');
                 PRAGMA user_version = 1;",
            )
            .unwrap();
        }
        native_storage::apply_migration_for_testing().expect("v1→v2 migration must succeed");
        let page = native_storage::get_completed_sessions_paged(10, 0)
            .expect("paged query must work after v1→v2 migration");
        assert!(
            page.iter().any(|s| s.id == "v1_s1"),
            "row inserted before migration must survive v1→v2"
        );
        {
            let conn = rusqlite::Connection::open(&db_path).unwrap();
            conn.execute(
                "DELETE FROM sessions WHERE id = ?1",
                rusqlite::params!["v1_s1"],
            )
            .unwrap();
        }
    }
    /// Insert a row with invalid JSON directly into `SQLite` and verify that
    /// `get_all` silently skips it rather than returning an error.
    #[test]
    fn get_all_skips_corrupt_rows() {
        let _g = lock();
        let db_path = native_storage::data_dir().join("log-out.db");
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
            version: DATA_VERSION,
            pending_exercise_ids: vec![],
            rest_start_time: None,
            current_exercise_id: None,
            current_exercise_start: None,
            paused_at: None,
        };
        let done = WorkoutSession {
            id: "paged_done".into(),
            start_time: 4_000,
            end_time: Some(5_000),
            exercise_logs: vec![],
            version: DATA_VERSION,
            pending_exercise_ids: vec![],
            rest_start_time: None,
            current_exercise_id: None,
            current_exercise_start: None,
            paused_at: None,
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
                version: DATA_VERSION,
                pending_exercise_ids: vec![],
                rest_start_time: None,
                current_exercise_id: None,
                current_exercise_start: None,
                paused_at: None,
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
            version: DATA_VERSION,
            pending_exercise_ids: vec![],
            rest_start_time: None,
            current_exercise_id: None,
            current_exercise_start: None,
            paused_at: None,
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
