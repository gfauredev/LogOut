use dioxus::prelude::*;
use crate::models::{Workout, WorkoutSession, ExerciseLog, CustomExercise};

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
    use_context_provider(|| Signal::new(Vec::<CustomExercise>::new()));

    // Load persisted data into the signals via a resource (lifecycle-managed)
    use_resource(load_storage_data);
}

// ── helpers to obtain the signals from any component ──

pub fn use_workouts() -> Signal<Vec<Workout>> {
    use_context::<Signal<Vec<Workout>>>()
}

pub fn use_sessions() -> Signal<Vec<WorkoutSession>> {
    use_context::<Signal<Vec<WorkoutSession>>>()
}

pub fn use_custom_exercises() -> Signal<Vec<CustomExercise>> {
    use_context::<Signal<Vec<CustomExercise>>>()
}

// ──────────────────────────────────────────
// IndexedDB persistence via rexie (wasm32 only)
// ──────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
pub(crate) mod idb {
    use rexie::{Rexie, ObjectStore, TransactionMode};
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
        let tx = db.transaction(&[store_name], TransactionMode::ReadWrite)
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
        let tx = db.transaction(&[store_name], TransactionMode::ReadWrite)
            .map_err(|e| format!("{e}"))?;
        let store = tx.store(store_name).map_err(|e| format!("{e}"))?;
        store.delete(JsValue::from_str(key)).await.map_err(|e| format!("{e}"))?;
        tx.done().await.map_err(|e| format!("{e}"))?;
        Ok(())
    }

    /// Load all items from a store.
    pub async fn get_all<T: serde::de::DeserializeOwned>(store_name: &str) -> Result<Vec<T>, String> {
        let db = open_db().await.map_err(|e| format!("{e}"))?;
        let tx = db.transaction(&[store_name], TransactionMode::ReadOnly)
            .map_err(|e| format!("{e}"))?;
        let store = tx.store(store_name).map_err(|e| format!("{e}"))?;
        let js_values = store.get_all(None, None).await.map_err(|e| format!("{e}"))?;

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
    #[cfg(target_arch = "wasm32")]
    {
        let mut workouts_sig = use_workouts();
        let mut sessions_sig = use_sessions();
        let mut custom_sig = use_custom_exercises();

        // First try IndexedDB, then fall back to localStorage for migration
        let mut from_idb = false;

        if let Ok(workouts) = idb::get_all::<Workout>(idb::STORE_WORKOUTS).await {
            if !workouts.is_empty() {
                info!("Loaded {} workouts from IndexedDB", workouts.len());
                workouts_sig.set(workouts);
                from_idb = true;
            }
        }
        if let Ok(sessions) = idb::get_all::<WorkoutSession>(idb::STORE_SESSIONS).await {
            if !sessions.is_empty() {
                info!("Loaded {} sessions from IndexedDB", sessions.len());
                sessions_sig.set(sessions);
                from_idb = true;
            }
        }
        if let Ok(custom) = idb::get_all::<CustomExercise>(idb::STORE_CUSTOM_EXERCISES).await {
            if !custom.is_empty() {
                info!("Loaded {} custom exercises from IndexedDB", custom.len());
                custom_sig.set(custom);
                from_idb = true;
            }
        }

        // Fall back to localStorage (one-time migration)
        if !from_idb {
            migrate_from_local_storage(workouts_sig, sessions_sig, custom_sig).await;
        }
    }
}

#[cfg(target_arch = "wasm32")]
async fn migrate_from_local_storage(
    mut workouts_sig: Signal<Vec<Workout>>,
    mut sessions_sig: Signal<Vec<WorkoutSession>>,
    mut custom_sig: Signal<Vec<CustomExercise>>,
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
            info!("Migrating {} workouts from localStorage → IndexedDB", workouts.len());
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
            info!("Migrating {} sessions from localStorage → IndexedDB", sessions.len());
            for s in &sessions {
                let _ = idb::put_item(idb::STORE_SESSIONS, s).await;
            }
            sessions_sig.set(sessions);
            let _ = storage.remove_item("log_workout_sessions");
        }
    }

    // Custom exercises
    if let Ok(Some(data)) = storage.get_item("log_workout_custom_exercises") {
        if let Ok(custom) = serde_json::from_str::<Vec<CustomExercise>>(&data) {
            info!("Migrating {} custom exercises from localStorage → IndexedDB", custom.len());
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

pub fn add_workout(workout: Workout) {
    let mut sig = use_workouts();
    sig.write().push(workout.clone());

    #[cfg(target_arch = "wasm32")]
    spawn(async move {
        if let Err(e) = idb::put_item(idb::STORE_WORKOUTS, &workout).await {
            error!("Failed to persist workout: {e}");
        }
    });
}

#[allow(dead_code)]
pub fn delete_workout(id: &str) {
    let mut sig = use_workouts();
    sig.write().retain(|w| w.id != id);

    #[cfg(target_arch = "wasm32")]
    {
        let id = id.to_owned();
        spawn(async move {
            let _ = idb::delete_item(idb::STORE_WORKOUTS, &id).await;
        });
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
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = idb::put_item(idb::STORE_SESSIONS, &session).await {
            error!("Failed to persist session: {e}");
        }
    });
}

pub fn delete_session(id: &str) {
    let mut sig = use_sessions();
    sig.write().retain(|s| s.id != id);

    #[cfg(target_arch = "wasm32")]
    {
        let id = id.to_owned();
        spawn(async move {
            let _ = idb::delete_item(idb::STORE_SESSIONS, &id).await;
        });
    }
}

pub fn add_custom_exercise(exercise: CustomExercise) {
    let mut sig = use_custom_exercises();
    sig.write().push(exercise.clone());

    #[cfg(target_arch = "wasm32")]
    spawn(async move {
        if let Err(e) = idb::put_item(idb::STORE_CUSTOM_EXERCISES, &exercise).await {
            error!("Failed to persist custom exercise: {e}");
        }
    });
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

#[allow(dead_code)]
pub fn get_active_session() -> Option<WorkoutSession> {
    let sessions = use_sessions();
    let result = sessions.read().iter().find(|s| s.is_active()).cloned();
    result
}

// ──────────────────────────────────────────
// Exercise IndexedDB helpers (used by exercise_db)
// ──────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
pub mod idb_exercises {
    use crate::models::Exercise;
    use super::idb;

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
