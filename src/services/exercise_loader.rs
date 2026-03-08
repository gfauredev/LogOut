/// Dioxus integration layer for the exercise database.
///
/// Provides and consumes the Dioxus context that holds the exercise list signal,
/// and drives the async load / background-refresh cycle.  Kept separate from
/// `exercise_db` so the data-access module stays unit-testable without a full
/// Dioxus virtual-DOM.
use crate::models::Exercise;
use crate::services::exercise_db;
use dioxus::prelude::*;

/// Provides the exercises signal and kicks off the background load.
/// Call once inside the root `App` component.
pub fn provide_exercises() {
    let wrapper = use_context_provider(|| exercise_db::AllExercisesSignal(Signal::new(Vec::new())));
    let sig = wrapper.0;

    spawn(async move {
        load_exercises(sig).await;
    });
}

/// Consumes the exercises signal from the Dioxus context.
pub fn use_exercises() -> Signal<Vec<Exercise>> {
    use_context::<exercise_db::AllExercisesSignal>().0
}

/// Clears the current exercise list and immediately re-downloads from the
/// configured URL.  Intended to be called after saving a new database URL so
/// the app reflects the change without requiring a full reload.
///
/// On success the toast shows a confirmation message; on error (network,
/// empty response, JSON parse) it shows an appropriate error message so the
/// user knows the URL change did not take effect.
pub async fn reload_exercises(mut sig: Signal<Vec<Exercise>>, mut toast: Signal<Option<String>>) {
    // Clear immediately so the UI does not show stale data from the old URL
    sig.set(Vec::new());

    #[cfg(target_arch = "wasm32")]
    {
        use crate::services::storage::idb_exercises;
        match exercise_db::download_exercises().await {
            Ok(exercises) if !exercises.is_empty() => {
                log::info!(
                    "Reloaded {} exercises from new URL, storing in IndexedDB",
                    exercises.len()
                );
                idb_exercises::store_all_exercises(&exercises).await;
                exercise_db::record_fetch_timestamp();
                sig.set(exercises.into_iter().map(Exercise::with_lowercase).collect());
                toast.set(Some(
                    "✅ Exercise database reloaded successfully".to_string(),
                ));
            }
            Ok(_) => {
                log::warn!("Reloaded exercises file was empty");
                toast.set(Some(
                    "⚠️ exercises.json was empty — check the database URL".to_string(),
                ));
            }
            Err(e) => {
                log::warn!("Failed to reload exercises: {e:?}");
                toast.set(Some(format!("❌ Failed to reload exercises: {e}")));
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use crate::services::storage::native_exercises;
        match exercise_db::download_exercises().await {
            Ok(exercises) if !exercises.is_empty() => {
                log::info!(
                    "Reloaded {} exercises from new URL, storing in local file",
                    exercises.len()
                );
                native_exercises::store_all_exercises(&exercises);
                exercise_db::record_fetch_timestamp();
                sig.set(exercises.into_iter().map(Exercise::with_lowercase).collect());
                toast.set(Some(
                    "✅ Exercise database reloaded successfully".to_string(),
                ));
            }
            Ok(_) => {
                log::warn!("Reloaded exercises file was empty");
                toast.set(Some(
                    "⚠️ exercises.json was empty — check the database URL".to_string(),
                ));
            }
            Err(e) => {
                log::warn!("Failed to reload exercises: {e:?}");
                toast.set(Some(format!("❌ Failed to reload exercises: {e}")));
            }
        }
    }
}

#[allow(unused_mut, unused_variables)]
async fn load_exercises(mut sig: Signal<Vec<Exercise>>) {
    // ── Web platform (wasm32 + IndexedDB) ────────────────────────────────────
    #[cfg(target_arch = "wasm32")]
    {
        use crate::services::storage::idb_exercises;

        let cached = idb_exercises::get_all_exercises().await.unwrap_or_default();
        let needs_refresh = !cached.is_empty() && exercise_db::is_refresh_due();

        if !cached.is_empty() {
            log::info!("Loaded {} exercises from IndexedDB", cached.len());
            sig.set(cached.into_iter().map(Exercise::with_lowercase).collect());

            if !needs_refresh {
                return;
            }

            // Re-fetch in the background to keep exercises up to date
            log::info!("Exercise database is stale – refreshing in background");
        }

        // Download from the network (first run or periodic refresh)
        match exercise_db::download_exercises().await {
            Ok(exercises) if !exercises.is_empty() => {
                log::info!(
                    "Downloaded {} exercises, storing in IndexedDB",
                    exercises.len()
                );
                idb_exercises::store_all_exercises(&exercises).await;
                exercise_db::record_fetch_timestamp();
                sig.set(exercises.into_iter().map(Exercise::with_lowercase).collect());
                return;
            }
            Ok(_) => log::warn!("Downloaded exercises file was empty"),
            Err(e) => log::warn!("Failed to download exercises: {e:?}"),
        }
    }

    // ── Native platform (Android / desktop) ──────────────────────────────────
    #[cfg(not(target_arch = "wasm32"))]
    {
        use crate::services::storage::native_exercises;

        let cached = native_exercises::get_all_exercises();
        let needs_refresh = !cached.is_empty() && exercise_db::is_refresh_due();

        if !cached.is_empty() {
            log::info!("Loaded {} exercises from local file", cached.len());
            sig.set(cached.into_iter().map(Exercise::with_lowercase).collect());

            if !needs_refresh {
                return;
            }

            log::info!("Exercise database is stale – refreshing in background");
        }

        match exercise_db::download_exercises().await {
            Ok(exercises) if !exercises.is_empty() => {
                log::info!(
                    "Downloaded {} exercises, storing in local file",
                    exercises.len()
                );
                native_exercises::store_all_exercises(&exercises);
                exercise_db::record_fetch_timestamp();
                sig.set(exercises.into_iter().map(Exercise::with_lowercase).collect());
                return;
            }
            Ok(_) => log::warn!("Downloaded exercises file was empty"),
            Err(e) => log::warn!("Failed to download exercises: {e:?}"),
        }
    }

    // No exercises available: database will remain empty until next launch or network becomes available
    log::warn!("No exercises available: failed to load from cache and download from API");
}
