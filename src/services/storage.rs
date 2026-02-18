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

    // Load persisted data into the signals
    load_from_storage();
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
// IndexedDB persistence  (wasm32 only)
// ──────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
pub(crate) mod idb {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    use web_sys::{IdbDatabase, IdbOpenDbRequest, IdbRequest, IdbTransactionMode};
    use js_sys::Array;
    use std::cell::RefCell;
    use std::rc::Rc;

    const DB_NAME: &str = "log_workout_db";
    const DB_VERSION: u32 = 2;

    pub const STORE_WORKOUTS: &str = "workouts";
    pub const STORE_SESSIONS: &str = "sessions";
    pub const STORE_CUSTOM_EXERCISES: &str = "custom_exercises";
    pub const STORE_EXERCISES: &str = "exercises";

    /// Open (or create) the IndexedDB database and return it via a Future.
    pub async fn open_db() -> Result<IdbDatabase, JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
        let idb_factory = window
            .indexed_db()?
            .ok_or_else(|| JsValue::from_str("indexedDB not available"))?;

        let open_req: IdbOpenDbRequest = idb_factory.open_with_u32(DB_NAME, DB_VERSION)?;

        // Handle upgrade (create object stores)
        let on_upgrade = Closure::wrap(Box::new(move |event: web_sys::Event| {
            let target = event.target().unwrap();
            let req: IdbOpenDbRequest = target.unchecked_into();
            let db: IdbDatabase = req.result().unwrap().unchecked_into();

            for store_name in &[STORE_WORKOUTS, STORE_SESSIONS, STORE_CUSTOM_EXERCISES, STORE_EXERCISES] {
                let names = db.object_store_names();
                let mut found = false;
                for i in 0..names.length() {
                    if let Some(name) = names.get(i) {
                        if name == *store_name {
                            found = true;
                            break;
                        }
                    }
                }
                if !found {
                    let params = web_sys::IdbObjectStoreParameters::new();
                    params.set_key_path(&JsValue::from_str("id"));
                    let _ = db.create_object_store_with_optional_parameters(store_name, &params);
                }
            }
        }) as Box<dyn FnMut(_)>);
        open_req.set_onupgradeneeded(Some(on_upgrade.as_ref().unchecked_ref()));
        on_upgrade.forget();

        // Wrap open request into a Future
        let (tx, rx) = futures_channel::oneshot::channel::<Result<IdbDatabase, JsValue>>();
        let tx = Rc::new(RefCell::new(Some(tx)));

        let tx_ok = tx.clone();
        let on_success = Closure::once(Box::new(move |event: web_sys::Event| {
            let target = event.target().unwrap();
            let req: IdbRequest = target.unchecked_into();
            let db: IdbDatabase = req.result().unwrap().unchecked_into();
            if let Some(sender) = tx_ok.borrow_mut().take() {
                let _ = sender.send(Ok(db));
            }
        }));
        open_req.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        on_success.forget();

        let tx_err = tx;
        let on_error = Closure::once(Box::new(move |_event: web_sys::Event| {
            if let Some(sender) = tx_err.borrow_mut().take() {
                let _ = sender.send(Err(JsValue::from_str("IndexedDB open error")));
            }
        }));
        open_req.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        on_error.forget();

        rx.await.unwrap_or(Err(JsValue::from_str("channel closed")))
    }

    /// Put a single serialisable item into a store (upsert by key).
    pub async fn put_item<T: serde::Serialize>(store_name: &str, item: &T) -> Result<(), JsValue> {
        let db = open_db().await?;
        let tx = db.transaction_with_str_and_mode(store_name, IdbTransactionMode::Readwrite)?;
        let store = tx.object_store(store_name)?;

        let js_val = serde_wasm_bindgen::to_value(item)?;
        store.put(&js_val)?;

        Ok(())
    }

    /// Delete an item from a store by its key.
    pub async fn delete_item(store_name: &str, key: &str) -> Result<(), JsValue> {
        let db = open_db().await?;
        let tx = db.transaction_with_str_and_mode(store_name, IdbTransactionMode::Readwrite)?;
        let store = tx.object_store(store_name)?;
        store.delete(&JsValue::from_str(key))?;
        Ok(())
    }

    /// Load all items from a store.
    pub async fn get_all<T: serde::de::DeserializeOwned + 'static>(store_name: &str) -> Result<Vec<T>, JsValue> {
        let db = open_db().await?;
        let tx = db.transaction_with_str_and_mode(store_name, IdbTransactionMode::Readonly)?;
        let store = tx.object_store(store_name)?;
        let req = store.get_all()?;

        let (tx_ch, rx_ch) = futures_channel::oneshot::channel::<Result<Vec<T>, JsValue>>();
        let tx_ch = Rc::new(RefCell::new(Some(tx_ch)));

        let tx_ok = tx_ch.clone();
        let on_success = Closure::once(Box::new(move |event: web_sys::Event| {
            let target = event.target().unwrap();
            let req: IdbRequest = target.unchecked_into();
            let result = req.result().unwrap();
            let array: Array = result.unchecked_into();
            let mut items = Vec::new();
            for i in 0..array.length() {
                let js_val = array.get(i);
                match serde_wasm_bindgen::from_value::<T>(js_val) {
                    Ok(item) => items.push(item),
                    Err(e) => log::warn!("Skipping corrupt IndexedDB entry at index {}: {}", i, e),
                }
            }
            if let Some(sender) = tx_ok.borrow_mut().take() {
                let _ = sender.send(Ok(items));
            }
        }));
        req.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        on_success.forget();

        let tx_err = tx_ch;
        let on_error = Closure::once(Box::new(move |_event: web_sys::Event| {
            if let Some(sender) = tx_err.borrow_mut().take() {
                let _ = sender.send(Err(JsValue::from_str("getAll error")));
            }
        }));
        req.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        on_error.forget();

        rx_ch.await.unwrap_or(Err(JsValue::from_str("channel closed")))
    }
}

// ──────────────────────────────────────────
// Load persisted data into context signals
// ──────────────────────────────────────────

fn load_from_storage() {
    #[cfg(target_arch = "wasm32")]
    {
        let mut workouts_sig = use_workouts();
        let mut sessions_sig = use_sessions();
        let mut custom_sig = use_custom_exercises();

        // First try IndexedDB, then fall back to localStorage for migration
        spawn(async move {
            // Try IndexedDB first
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
        });
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
            error!("Failed to persist workout: {:?}", e);
        }
    });
}

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

    #[cfg(target_arch = "wasm32")]
    spawn(async move {
        if let Err(e) = idb::put_item(idb::STORE_SESSIONS, &session).await {
            error!("Failed to persist session: {:?}", e);
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
            error!("Failed to persist custom exercise: {:?}", e);
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

pub fn get_active_session() -> Option<WorkoutSession> {
    let sessions = use_sessions();
    let sessions = sessions.read();
    let result = sessions.iter().find(|s| s.is_active()).cloned();
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
        idb::get_all::<Exercise>(idb::STORE_EXERCISES)
            .await
            .map_err(|e| format!("{:?}", e))
    }

    pub async fn store_all_exercises(exercises: &[Exercise]) {
        for ex in exercises {
            if let Err(e) = idb::put_item(idb::STORE_EXERCISES, ex).await {
                log::error!("Failed to store exercise {}: {:?}", ex.id, e);
            }
        }
    }
}
