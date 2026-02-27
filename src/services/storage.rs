use crate::models::{Exercise, ExerciseLog, Workout, WorkoutSession};
use crate::ToastSignal;
use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

#[cfg(target_arch = "wasm32")]
use log::{error, info};

// ──────────────────────────────────────────
// Dioxus context-based state (replaces static Mutex)
// ──────────────────────────────────────────

/// Provide shared signals at the top of the component tree.
/// Call once inside the root `App` component.
pub fn provide_app_state() {
    use_context_provider(|| Signal::new(Vec::<Workout>::new()));
    use_context_provider(|| Signal::new(Vec::<WorkoutSession>::new()));
    use_context_provider(|| Signal::new(Vec::<Exercise>::new()));

    // Load persisted data into the signals via a resource (lifecycle-managed)
    use_resource(load_storage_data);
}

// ── helpers to obtain the signals from any component ──

#[allow(dead_code)]
pub fn use_workouts() -> Signal<Vec<Workout>> {
    consume_context::<Signal<Vec<Workout>>>()
}

pub fn use_sessions() -> Signal<Vec<WorkoutSession>> {
    consume_context::<Signal<Vec<WorkoutSession>>>()
}

pub fn use_custom_exercises() -> Signal<Vec<Exercise>> {
    consume_context::<Signal<Vec<Exercise>>>()
}

// ──────────────────────────────────────────
// IndexedDB persistence via rexie (wasm32 only)
// ──────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
pub(crate) mod idb {
    use rexie::{ObjectStore, Rexie, TransactionMode};
    use wasm_bindgen::JsValue;

    const DB_NAME: &str = "log_workout_db";
    const DB_VERSION: u32 = 2;

    pub const STORE_WORKOUTS: &str = "workouts";
    pub const STORE_SESSIONS: &str = "sessions";
    pub const STORE_CUSTOM_EXERCISES: &str = "custom_exercises";
    pub const STORE_EXERCISES: &str = "exercises";

    /// Open (or create) the IndexedDB database via rexie.
    async fn open_db() -> Result<Rexie, rexie::Error> {
        Rexie::builder(DB_NAME)
            .version(DB_VERSION)
            .add_object_store(ObjectStore::new(STORE_WORKOUTS).key_path("id"))
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

// ──────────────────────────────────────────
// Load persisted data into context signals (via use_resource)
// ──────────────────────────────────────────

async fn load_storage_data() {
    // ── Web platform (wasm32 + IndexedDB) ────────────────────────────────────
    #[cfg(target_arch = "wasm32")]
    {
        let mut workouts_sig = use_workouts();
        let mut sessions_sig = use_sessions();
        let mut custom_sig = use_custom_exercises();
        let mut toast = consume_context::<ToastSignal>().0;

        // First try IndexedDB, then fall back to localStorage for migration
        let mut from_idb = false;

        match idb::get_all::<Workout>(idb::STORE_WORKOUTS).await {
            Ok(workouts) if !workouts.is_empty() => {
                info!("Loaded {} workouts from IndexedDB", workouts.len());
                workouts_sig.set(workouts);
                from_idb = true;
            }
            Err(e) => {
                error!("Failed to load workouts from IndexedDB: {e}");
                toast.set(Some(format!("⚠️ Failed to load workouts: {e}")));
            }
            _ => {}
        }
        match idb::get_all::<WorkoutSession>(idb::STORE_SESSIONS).await {
            Ok(sessions) if !sessions.is_empty() => {
                info!("Loaded {} sessions from IndexedDB", sessions.len());
                sessions_sig.set(sessions);
                from_idb = true;
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
                from_idb = true;
            }
            Err(e) => {
                error!("Failed to load custom exercises from IndexedDB: {e}");
                toast.set(Some(format!("⚠️ Failed to load custom exercises: {e}")));
            }
            _ => {}
        }

        // Fall back to localStorage (one-time migration)
        if !from_idb {
            migrate_from_local_storage(workouts_sig, sessions_sig, custom_sig).await;
        }
    }

    // ── Native platform (Android / desktop) ──────────────────────────────────
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut workouts_sig = use_workouts();
        let mut sessions_sig = use_sessions();
        let mut custom_sig = use_custom_exercises();
        let mut toast = consume_context::<ToastSignal>().0;

        match native_storage::get_all::<Workout>(native_storage::STORE_WORKOUTS) {
            Ok(workouts) if !workouts.is_empty() => {
                log::info!("Loaded {} workouts from storage", workouts.len());
                workouts_sig.set(workouts);
            }
            Err(e) => {
                log::error!("Failed to load workouts: {e}");
                toast.set(Some(format!("⚠️ Failed to load workouts: {e}")));
            }
            _ => {}
        }
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

#[cfg(target_arch = "wasm32")]
async fn migrate_from_local_storage(
    mut workouts_sig: Signal<Vec<Workout>>,
    mut sessions_sig: Signal<Vec<WorkoutSession>>,
    mut custom_sig: Signal<Vec<Exercise>>,
) {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let storage = match window.local_storage() {
        Ok(Some(s)) => s,
        _ => return,
    };

    // Workouts
    if let Ok(Some(data)) = storage.get_item("log_workout_workouts") {
        if let Ok(workouts) = serde_json::from_str::<Vec<Workout>>(&data) {
            info!(
                "Migrating {} workouts from localStorage → IndexedDB",
                workouts.len()
            );
            for w in &workouts {
                let _ = idb::put_item(idb::STORE_WORKOUTS, w).await;
            }
            workouts_sig.set(workouts);
            let _ = storage.remove_item("log_workout_workouts");
        }
    }

    // Sessions
    if let Ok(Some(data)) = storage.get_item("log_workout_sessions") {
        if let Ok(sessions) = serde_json::from_str::<Vec<WorkoutSession>>(&data) {
            info!(
                "Migrating {} sessions from localStorage → IndexedDB",
                sessions.len()
            );
            for s in &sessions {
                let _ = idb::put_item(idb::STORE_SESSIONS, s).await;
            }
            sessions_sig.set(sessions);
            let _ = storage.remove_item("log_workout_sessions");
        }
    }

    // Custom exercises
    if let Ok(Some(data)) = storage.get_item("log_workout_custom_exercises") {
        if let Ok(custom) = serde_json::from_str::<Vec<Exercise>>(&data) {
            info!(
                "Migrating {} custom exercises from localStorage → IndexedDB",
                custom.len()
            );
            for c in &custom {
                let _ = idb::put_item(idb::STORE_CUSTOM_EXERCISES, c).await;
            }
            custom_sig.set(custom);
            let _ = storage.remove_item("log_workout_custom_exercises");
        }
    }
}

// ──────────────────────────────────────────
// Public mutation helpers (granular IDB writes)
// ──────────────────────────────────────────

/// Appends a completed workout to the workout list and persists it to IndexedDB.
/// Note: the WorkoutLog page currently has no route; this function is kept for
/// future use when the workout-planning flow is wired up.
#[allow(dead_code)]
pub fn add_workout(workout: Workout) {
    let mut sig = use_workouts();
    sig.write().push(workout.clone());

    #[cfg(target_arch = "wasm32")]
    {
        let mut toast = consume_context::<ToastSignal>().0;
        spawn_local(async move {
            if let Err(e) = idb::put_item(idb::STORE_WORKOUTS, &workout).await {
                error!("Failed to persist workout: {e}");
                toast.set(Some(format!("⚠️ Failed to save workout: {e}")));
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    if let Err(e) = native_storage::put_item(native_storage::STORE_WORKOUTS, &workout.id, &workout)
    {
        log::error!("Failed to persist workout: {e}");
        consume_context::<ToastSignal>()
            .0
            .set(Some(format!("⚠️ Failed to save workout: {e}")));
    }
}

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
    #[cfg(target_arch = "wasm32")]
    {
        let mut toast = consume_context::<ToastSignal>().0;
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) = idb::put_item(idb::STORE_SESSIONS, &session).await {
                error!("Failed to persist session: {e}");
                toast.set(Some(format!("⚠️ Failed to save session: {e}")));
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    if let Err(e) = native_storage::put_item(native_storage::STORE_SESSIONS, &session.id, &session)
    {
        log::error!("Failed to persist session: {e}");
        consume_context::<ToastSignal>()
            .0
            .set(Some(format!("⚠️ Failed to save session: {e}")));
    }
}

pub fn delete_session(id: &str) {
    let mut sig = use_sessions();
    sig.write().retain(|s| s.id != id);

    #[cfg(target_arch = "wasm32")]
    {
        let id = id.to_owned();
        let mut toast = consume_context::<ToastSignal>().0;
        spawn_local(async move {
            if let Err(e) = idb::delete_item(idb::STORE_SESSIONS, &id).await {
                error!("Failed to delete session: {e}");
                toast.set(Some(format!("⚠️ Failed to delete session: {e}")));
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    if let Err(e) = native_storage::delete_item(native_storage::STORE_SESSIONS, id) {
        log::error!("Failed to delete session: {e}");
        consume_context::<ToastSignal>()
            .0
            .set(Some(format!("⚠️ Failed to delete session: {e}")));
    }
}

pub fn add_custom_exercise(exercise: Exercise) {
    let mut sig = use_custom_exercises();
    sig.write().push(exercise.clone());

    #[cfg(target_arch = "wasm32")]
    {
        let mut toast = consume_context::<ToastSignal>().0;
        spawn_local(async move {
            if let Err(e) = idb::put_item(idb::STORE_CUSTOM_EXERCISES, &exercise).await {
                error!("Failed to persist custom exercise: {e}");
                toast.set(Some(format!("⚠️ Failed to save exercise: {e}")));
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    if let Err(e) = native_storage::put_item(
        native_storage::STORE_CUSTOM_EXERCISES,
        &exercise.id,
        &exercise,
    ) {
        log::error!("Failed to persist custom exercise: {e}");
        consume_context::<ToastSignal>()
            .0
            .set(Some(format!("⚠️ Failed to save exercise: {e}")));
    }
}

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
        let mut toast = consume_context::<ToastSignal>().0;
        spawn_local(async move {
            if let Err(e) = idb::put_item(idb::STORE_CUSTOM_EXERCISES, &exercise).await {
                error!("Failed to persist updated custom exercise: {e}");
                toast.set(Some(format!("⚠️ Failed to update exercise: {e}")));
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    if let Err(e) = native_storage::put_item(
        native_storage::STORE_CUSTOM_EXERCISES,
        &exercise.id,
        &exercise,
    ) {
        log::error!("Failed to persist updated custom exercise: {e}");
        consume_context::<ToastSignal>()
            .0
            .set(Some(format!("⚠️ Failed to update exercise: {e}")));
    }
}

// Helper to get last values for an exercise (for prefilling)
pub fn get_last_exercise_log(exercise_id: &str) -> Option<ExerciseLog> {
    let sessions = use_sessions();
    let sessions = sessions.read();
    for session in sessions.iter().rev() {
        for log in session.exercise_logs.iter().rev() {
            if log.exercise_id == exercise_id && log.is_complete() {
                return Some(log.clone());
            }
        }
    }
    None
}

// ──────────────────────────────────────────
// Exercise storage helpers (used by exercise_db)
// ──────────────────────────────────────────

/// IndexedDB-backed exercise storage for the web platform.
#[cfg(target_arch = "wasm32")]
pub mod idb_exercises {
    use super::idb;
    use crate::models::Exercise;

    pub async fn get_all_exercises() -> Result<Vec<Exercise>, String> {
        idb::get_all::<Exercise>(idb::STORE_EXERCISES).await
    }

    pub async fn store_all_exercises(exercises: &[Exercise]) {
        for ex in exercises {
            if let Err(e) = idb::put_item(idb::STORE_EXERCISES, ex).await {
                log::error!("Failed to store exercise {}: {e}", ex.id);
            }
        }
    }
}

/// File-backed exercise storage for native platforms (Android / desktop).
#[cfg(not(target_arch = "wasm32"))]
pub mod native_exercises {
    use super::native_storage;
    use crate::models::Exercise;

    pub fn get_all_exercises() -> Vec<Exercise> {
        native_storage::get_all::<Exercise>(native_storage::STORE_EXERCISES).unwrap_or_default()
    }

    pub fn store_all_exercises(exercises: &[Exercise]) {
        if let Err(e) = native_storage::store_all(native_storage::STORE_EXERCISES, exercises) {
            log::error!("Failed to store exercises: {e}");
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

    pub const STORE_WORKOUTS: &str = "workouts";
    pub const STORE_SESSIONS: &str = "sessions";
    pub const STORE_CUSTOM_EXERCISES: &str = "custom_exercises";
    pub const STORE_EXERCISES: &str = "exercises";

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
    fn open_db() -> Result<Connection, String> {
        std::fs::create_dir_all(data_dir()).map_err(|e| e.to_string())?;
        let conn = Connection::open(db_path()).map_err(|e| e.to_string())?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS workouts (id TEXT PRIMARY KEY, data TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS sessions (id TEXT PRIMARY KEY, data TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS custom_exercises (id TEXT PRIMARY KEY, data TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS exercises (id TEXT PRIMARY KEY, data TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS config (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
        )
        .map_err(|e| e.to_string())?;
        // Run one-time migration from old JSON files (no-op if already migrated).
        migrate_from_json(&conn);
        Ok(conn)
    }

    /// One-time migration: reads any existing `<store>.json` files and inserts
    /// their contents into SQLite, then deletes the JSON files.
    fn migrate_from_json(conn: &Connection) {
        for store in &[
            STORE_WORKOUTS,
            STORE_SESSIONS,
            STORE_CUSTOM_EXERCISES,
            STORE_EXERCISES,
        ] {
            let json_path = data_dir().join(format!("{store}.json"));
            if !json_path.exists() {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&json_path) else {
                continue;
            };
            let Ok(items) = serde_json::from_str::<Vec<serde_json::Value>>(&text) else {
                continue;
            };
            for item in &items {
                let Some(id) = item.get("id").and_then(|v| v.as_str()) else {
                    continue;
                };
                let Ok(data) = serde_json::to_string(item) else {
                    continue;
                };
                let _ = conn.execute(
                    &format!("INSERT OR IGNORE INTO {store} (id, data) VALUES (?1, ?2)"),
                    params![id, data],
                );
            }
            let _ = std::fs::remove_file(&json_path);
            log::info!("Migrated {} from JSON to SQLite", store);
        }
        // Migrate config.json
        let config_path = data_dir().join("config.json");
        if config_path.exists() {
            if let Ok(text) = std::fs::read_to_string(&config_path) {
                if let Ok(serde_json::Value::Object(map)) = serde_json::from_str(&text) {
                    for (key, val) in &map {
                        if let Some(value) = val.as_str() {
                            let _ = conn.execute(
                                "INSERT OR IGNORE INTO config (key, value) VALUES (?1, ?2)",
                                params![key, value],
                            );
                        }
                    }
                }
            }
            let _ = std::fs::remove_file(&config_path);
            log::info!("Migrated config.json to SQLite");
        }
    }

    /// Reads all items from a store, deserialising each row's JSON `data` column.
    pub fn get_all<T: DeserializeOwned>(store_name: &str) -> Result<Vec<T>, String> {
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
                    .map_err(|e| log::warn!("Skipping corrupt SQLite row: {e}"))
                    .ok()
            })
            .collect();
        Ok(items)
    }

    /// Replaces the entire contents of a store with `items` in a single transaction.
    pub fn store_all<T: Serialize>(store_name: &str, items: &[T]) -> Result<(), String> {
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
}
