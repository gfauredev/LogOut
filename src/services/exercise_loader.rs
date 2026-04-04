/// Dioxus integration layer for the exercise database.
///
/// Provides and consumes the Dioxus context that holds the exercise list signal,
/// and drives the async load / background-refresh cycle.  Kept separate from
/// `exercise_db` so the data-access module stays unit-testable without a full
/// Dioxus virtual-DOM.
use crate::models::Exercise;
use crate::services::exercise_db;
use crate::{DbEmptyToastSignal, DbI18nSignal, ToastSignal};
use dioxus::prelude::*;
use std::sync::Arc;
/// Provides the exercises signal and kicks off the background load from cache.
/// Never auto-downloads; if the cache is empty a toast is shown instead.
/// Call once inside the root `App` component.
pub fn provide_exercises() {
    let wrapper = use_context_provider(|| exercise_db::AllExercisesSignal(Signal::new(Vec::new())));
    let sig = wrapper.0;
    let mut i18n_sig = use_context::<DbI18nSignal>().0;
    let mut toast = use_context::<ToastSignal>().0;
    let db_empty_toast = use_context::<DbEmptyToastSignal>().0;
    #[cfg(not(target_arch = "wasm32"))]
    let img_progress = use_context::<crate::ImageDownloadProgressSignal>().0;

    // Load cached exercises immediately (no network call), then download any
    // missing images in the background.
    spawn(async move {
        load_exercises(sig, db_empty_toast).await;
        // After loading from cache, download any images that are missing on
        // disk.  This handles the case where a previous image download was
        // interrupted (e.g. by the screen locking).  A separate Dioxus task
        // is spawned so the download runs concurrently without blocking the
        // rest of the startup sequence.
        #[cfg(not(target_arch = "wasm32"))]
        {
            let exercises: Vec<Exercise> = sig.read().iter().map(|e| e.as_ref().clone()).collect();
            if !exercises.is_empty() {
                spawn(async move {
                    exercise_db::download_db_images(&exercises, img_progress).await;
                });
            }
        }
    });

    // Download i18n data in background
    spawn(async move {
        match exercise_db::download_db_i18n().await {
            Ok(i18n_data) if !i18n_data.is_empty() => {
                i18n_sig.set(i18n_data);
            }
            Ok(_) => {
                // Empty i18n map is normal for offline mode; app falls back to English labels.
            }
            Err(e) => {
                log::warn!("Failed to download i18n data: {e}");
                toast
                    .write()
                    .push_back(format!("⚠️ Failed to load i18n data: {e}"));
            }
        }
    });
}
/// Consumes the exercises signal from the Dioxus context.
pub fn use_exercises() -> Signal<Vec<Arc<Exercise>>> {
    use_context::<exercise_db::AllExercisesSignal>().0
}
/// Clears the current exercise list and immediately re-downloads from the
/// configured URL.  Intended to be called after saving a new database URL so
/// the app reflects the change without requiring a full reload.
///
/// On success the toast shows a confirmation message; on error (network,
/// empty response, JSON parse) it shows an appropriate error message so the
/// user knows the URL change did not take effect.
pub async fn reload_exercises(
    mut sig: Signal<Vec<Arc<Exercise>>>,
    mut toast: Signal<std::collections::VecDeque<String>>,
    img_progress: Signal<Option<(usize, usize)>>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        use crate::services::storage::idb_exercises;
        toast
            .write()
            .push_back("⬇️ Downloading exercise database…".to_string());
        idb_exercises::clear_all_exercises().await;
        match exercise_db::download_exercises().await {
            Ok(Some(exercises)) if !exercises.is_empty() => {
                log::info!(
                    "Reloaded {} exercises from new URL, storing in IndexedDB",
                    exercises.len()
                );
                idb_exercises::store_all_exercises(&exercises).await;
                sig.set(
                    exercises
                        .into_iter()
                        .map(|e| Arc::new(Exercise::with_lowercase(e)))
                        .collect(),
                );
                toast
                    .write()
                    .push_back("💾 Exercise database reloaded successfully".to_string());
            }
            Ok(Some(_)) => {
                log::warn!("Reloaded exercises file was empty");
                toast
                    .write()
                    .push_back("⚠️ exercises.json was empty — check the database URL".to_string());
            }
            Ok(None) => {
                log::info!("exercises.json unchanged (304) — no reload needed");
                toast
                    .write()
                    .push_back("ℹ️ Exercise database is already up to date".to_string());
            }
            Err(e) => {
                log::warn!("Failed to reload exercises: {e:?}");
                toast
                    .write()
                    .push_back(format!("❌ Failed to reload exercises: {e}"));
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use crate::services::storage::native_exercises;
        toast
            .write()
            .push_back("⬇️ Downloading exercise database…".to_string());
        native_exercises::clear_all_exercises();
        match exercise_db::download_exercises().await {
            Ok(Some(exercises)) if !exercises.is_empty() => {
                log::info!(
                    "Reloaded {} exercises from new URL, storing in local file",
                    exercises.len()
                );
                native_exercises::store_all_exercises(&exercises);
                // Clone for the background image download before consuming exercises.
                let exercises_for_download = exercises.clone();
                // Show exercises immediately — do not block on image download.
                sig.set(
                    exercises
                        .into_iter()
                        .map(|e| Arc::new(Exercise::with_lowercase(e)))
                        .collect(),
                );
                toast
                    .write()
                    .push_back("💾 Exercise database reloaded successfully".to_string());
                // Spawn image download as a separate Dioxus task so that it
                // continues running after reload_exercises returns and so that
                // exercises are visible immediately without waiting for all
                // images to download first.
                spawn(async move {
                    exercise_db::download_db_images(&exercises_for_download, img_progress).await;
                });
            }
            Ok(Some(_)) => {
                log::warn!("Reloaded exercises file was empty");
                toast
                    .write()
                    .push_back("⚠️ exercises.json was empty — check the database URL".to_string());
            }
            Ok(None) => {
                log::info!("exercises.json unchanged (304) — no reload needed");
                toast
                    .write()
                    .push_back("ℹ️ Exercise database is already up to date".to_string());
            }
            Err(e) => {
                log::warn!("Failed to reload exercises: {e:?}");
                toast
                    .write()
                    .push_back(format!("❌ Failed to reload exercises: {e}"));
            }
        }
    }
}
/// Loads exercises from the local cache into the signal.
/// If the cache is empty the `db_empty_toast` signal is set to `true` so the
/// UI can prompt the user to download the database.
async fn load_exercises(mut sig: Signal<Vec<Arc<Exercise>>>, mut db_empty_toast: Signal<bool>) {
    #[cfg(target_arch = "wasm32")]
    {
        use crate::services::storage::idb_exercises;
        let cached = idb_exercises::get_all_exercises().await.unwrap_or_default();
        if cached.is_empty() {
            log::info!("Exercise cache empty — showing download prompt");
            db_empty_toast.set(true);
        } else {
            log::info!("Loaded {} exercises from IndexedDB", cached.len());
            sig.set(
                cached
                    .into_iter()
                    .map(|e| Arc::new(Exercise::with_lowercase(e)))
                    .collect(),
            );
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use crate::services::storage::native_exercises;
        // Use spawn_blocking so the function is properly async on native.
        let cached = match tokio::task::spawn_blocking(native_exercises::get_all_exercises).await {
            Ok(exercises) => exercises,
            Err(e) => {
                log::warn!("Failed to load exercises from local file: {e}");
                Vec::new()
            }
        };
        if cached.is_empty() {
            log::info!("Exercise cache empty — showing download prompt");
            db_empty_toast.set(true);
        } else {
            log::info!("Loaded {} exercises from local file", cached.len());
            sig.set(
                cached
                    .into_iter()
                    .map(|e| Arc::new(Exercise::with_lowercase(e)))
                    .collect(),
            );
        }
    }
}
