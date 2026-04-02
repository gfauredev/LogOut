/// Dioxus integration layer for the exercise database.
///
/// Provides and consumes the Dioxus context that holds the exercise list signal,
/// and drives the async load / background-refresh cycle.  Kept separate from
/// `exercise_db` so the data-access module stays unit-testable without a full
/// Dioxus virtual-DOM.
use crate::models::Exercise;
use crate::services::exercise_db;
use crate::{DbI18nSignal, ToastSignal};
use dioxus::prelude::*;
use std::sync::Arc;
/// Provides the exercises signal and kicks off the background load.
/// Call once inside the root `App` component.
pub fn provide_exercises() {
    let wrapper = use_context_provider(|| exercise_db::AllExercisesSignal(Signal::new(Vec::new())));
    let sig = wrapper.0;
    let mut i18n_sig = use_context::<DbI18nSignal>().0;
    let mut toast = use_context::<ToastSignal>().0;
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
        load_exercises(sig).await;
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
) {
    #[cfg(target_arch = "wasm32")]
    {
        use crate::services::storage::idb_exercises;
        idb_exercises::clear_all_exercises().await;
        match exercise_db::download_exercises().await {
            Ok(Some(exercises)) if !exercises.is_empty() => {
                log::info!(
                    "Reloaded {} exercises from new URL, storing in IndexedDB",
                    exercises.len()
                );
                idb_exercises::store_all_exercises(&exercises).await;
                exercise_db::record_fetch_timestamp();
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
        native_exercises::clear_all_exercises();
        match exercise_db::download_exercises().await {
            Ok(Some(exercises)) if !exercises.is_empty() => {
                log::info!(
                    "Reloaded {} exercises from new URL, storing in local file",
                    exercises.len()
                );
                native_exercises::store_all_exercises(&exercises);
                exercise_db::record_fetch_timestamp();
                exercise_db::download_db_images(&exercises).await;
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
}
async fn load_exercises(mut sig: Signal<Vec<Arc<Exercise>>>) {
    #[cfg(target_arch = "wasm32")]
    {
        use crate::services::storage::idb_exercises;
        let cached = idb_exercises::get_all_exercises().await.unwrap_or_default();
        let needs_refresh = !cached.is_empty() && exercise_db::is_refresh_due();
        if !cached.is_empty() {
            log::info!("Loaded {} exercises from IndexedDB", cached.len());
            sig.set(
                cached
                    .into_iter()
                    .map(|e| Arc::new(Exercise::with_lowercase(e)))
                    .collect(),
            );
            if !needs_refresh {
                return;
            }
            log::info!("Exercise database is stale – refreshing in background");
        }
        match exercise_db::download_exercises().await {
            Ok(Some(exercises)) if !exercises.is_empty() => {
                log::info!(
                    "Downloaded {} exercises, storing in IndexedDB",
                    exercises.len()
                );
                idb_exercises::store_all_exercises(&exercises).await;
                exercise_db::record_fetch_timestamp();
                sig.set(
                    exercises
                        .into_iter()
                        .map(|e| Arc::new(Exercise::with_lowercase(e)))
                        .collect(),
                );
                return;
            }
            Ok(Some(_)) => log::warn!("Downloaded exercises file was empty"),
            Ok(None) => {
                // 304 Not Modified: cached copy in IndexedDB is still current.
                log::info!("exercises.json unchanged (304) – using IndexedDB cache");
                exercise_db::record_fetch_timestamp();
                return;
            }
            Err(e) => log::warn!("Failed to download exercises: {e:?}"),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use crate::services::storage::native_exercises;
        let cached = native_exercises::get_all_exercises();
        let needs_refresh = !cached.is_empty() && exercise_db::is_refresh_due();
        if !cached.is_empty() {
            log::info!("Loaded {} exercises from local file", cached.len());
            sig.set(
                cached
                    .into_iter()
                    .map(|e| Arc::new(Exercise::with_lowercase(e)))
                    .collect(),
            );
            if !needs_refresh {
                return;
            }
            log::info!("Exercise database is stale – refreshing in background");
        }
        match exercise_db::download_exercises().await {
            Ok(Some(exercises)) if !exercises.is_empty() => {
                log::info!(
                    "Downloaded {} exercises, storing in local file",
                    exercises.len()
                );
                native_exercises::store_all_exercises(&exercises);
                exercise_db::record_fetch_timestamp();
                exercise_db::download_db_images(&exercises).await;
                sig.set(
                    exercises
                        .into_iter()
                        .map(|e| Arc::new(Exercise::with_lowercase(e)))
                        .collect(),
                );
                return;
            }
            Ok(Some(_)) => log::warn!("Downloaded exercises file was empty"),
            Ok(None) => {
                // 304 Not Modified: cached copy on disk is still current.
                log::info!("exercises.json unchanged (304) – using local cache");
                exercise_db::record_fetch_timestamp();
                return;
            }
            Err(e) => log::warn!("Failed to download exercises: {e:?}"),
        }
    }
    log::warn!("No exercises available: failed to load from cache and download from API");
}
