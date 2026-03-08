//! **`LogOut`** – Turn off your computer, Log your workOut
//!
//! A simple, efficient and cross-platform workout logging application with
//! 800+ built-in exercises.  The app is built with [Dioxus] and targets both
//! PWA (web) and native Android / desktop platforms.
//!
//! [Dioxus]: https://dioxuslabs.com

use dioxus::prelude::*;

mod components;
mod models;
mod services;
/// Pure utility helpers (date formatting, URL resolution, timestamp helpers).
pub mod utils;

use components::{
    AddExercise, Analytics, Credits, EditExercise, Exercises, GlobalSessionHeader, Home,
};

/// Global context signal for the congratulations toast shown after completing a session.
#[derive(Clone, Copy)]
pub struct CongratulationsSignal(pub Signal<bool>);

/// Global context signal for a general-purpose toast message.
#[derive(Clone, Copy)]
pub struct ToastSignal(pub Signal<Option<String>>);

/// Global context signal that, when `true`, shows a persistent notification-
/// permission warning toast.  The toast prompts the user to click it in order
/// to trigger the browser permission dialog.
#[derive(Clone, Copy)]
pub struct NotificationPermissionToastSignal(pub Signal<bool>);

/// Global context signal used to show/hide the rest-duration input form in
/// the active [`SessionView`].  The form is toggled by clicking the timer in
/// the [`GlobalSessionHeader`] which lives in the layout and is shared across
/// all pages.
#[derive(Clone, Copy)]
pub struct ShowRestInputSignal(pub Signal<bool>);

/// Auto-dismiss delay for toasts in milliseconds.
const TOAST_DISMISS_MS: u32 = 3_000;

/// Global context signal for pre-filling the exercise list search query.
#[derive(Clone, Copy)]
pub struct ExerciseSearchSignal(pub Signal<Option<String>>);

/// Global context signal holding a pending deep-link action that requires the
/// exercise list to be loaded before it can be executed (e.g. creating a past
/// session with specific exercises).
#[derive(Clone, Copy)]
pub struct PendingDeepLinkSignal(pub Signal<Option<utils::DeepLinkAction>>);

#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(DeepLinkLayout)]
    #[route("/")]
    Home {},
    #[route("/exercises")]
    Exercises {},
    #[route("/analytics")]
    Analytics {},
    #[route("/credits")]
    Credits {},
    #[route("/add-exercise")]
    AddExercise {},
    #[route("/edit-exercise/:id")]
    EditExercise { id: String },
}

fn main() {
    // Initialize logger
    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).expect("failed to init logger");

    // Initialize Android-specific paths and channels
    #[cfg(target_os = "android")]
    {
        // Try to get the internal data directory from the environment or system properties.
        // Dioxus/Tao on Android typically sets some environment variables or we can
        // rely on the JNI bridge `setDataDir` to be called by the Java side.
        services::android_notifications::setup_notification_channel();
    }
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
    {
        // Desktop notifications or other setup
    }

    // Register service worker for offline image caching
    services::service_worker::register_service_worker();

    // Prevent the device screen from sleeping while the app is open
    services::wake_lock::enable_wake_lock();

    launch(App);
}

#[component]
fn App() -> Element {
    // Provide shared state signals via context
    services::storage::provide_app_state();
    services::exercise_db::provide_exercises();
    use_context_provider(|| CongratulationsSignal(Signal::new(false)));
    use_context_provider(|| ToastSignal(Signal::new(None)));
    use_context_provider(|| NotificationPermissionToastSignal(Signal::new(false)));
    use_context_provider(|| ExerciseSearchSignal(Signal::new(None)));
    use_context_provider(|| PendingDeepLinkSignal(Signal::new(None)));
    use_context_provider(|| ShowRestInputSignal(Signal::new(false)));

    // Show the notification permission warning toast when permission has not yet
    // been granted.  The toast prompts the user to click it — respecting browsers
    // that require a user gesture before the permission dialog can be shown.
    #[cfg(all(target_arch = "wasm32", feature = "web-platform"))]
    {
        let mut notif_toast = use_context::<NotificationPermissionToastSignal>().0;
        use_hook(move || {
            use web_sys::NotificationPermission;
            match web_sys::Notification::permission() {
                NotificationPermission::Default | NotificationPermission::Denied => {
                    notif_toast.set(true);
                }
                _ => {}
            }
        });
    }

    rsx! {
        Stylesheet { href: asset!("/assets/style.scss") }
        Router::<Route> {}
        CongratulationsToast {}
        Toast {}
        NotificationPermissionToast {}
    }
}

/// Layout component rendered inside the Router context for all routes.
///
/// Handles `logworkout://` deep links (and their web equivalents via `?dl_*`
/// URL query parameters) on first mount.  Navigation links require the Router
/// context, so this component is the right place to call `use_navigator()`.
///
/// **Immediate actions** (URL storage, exercise search pre-fill, navigation)
/// are executed inside `use_hook` which runs once per component mount.
///
/// **Deferred actions** (creating a past session) are stored in
/// [`PendingDeepLinkSignal`] and executed via `use_effect` once the exercise
/// list has been loaded from the network/cache.
#[component]
fn DeepLinkLayout() -> Element {
    #[cfg(target_arch = "wasm32")]
    {
        use utils::DeepLinkAction;

        let nav = use_navigator();
        let exercises_sig = services::exercise_db::use_exercises();
        let mut search_signal = consume_context::<ExerciseSearchSignal>().0;
        let mut pending = consume_context::<PendingDeepLinkSignal>().0;

        // ── First-mount: parse URL params and execute immediate actions ──────
        use_hook(move || {
            let Some(action) = utils::parse_web_deep_link() else {
                return;
            };
            match action {
                DeepLinkAction::Navigate(path) => {
                    let route = path_to_route(&path);
                    nav.push(route);
                }
                DeepLinkAction::SearchExercises(q) => {
                    search_signal.set(Some(q));
                    nav.push(Route::Exercises {});
                }
                DeepLinkAction::SetDbUrl(url) => {
                    // Normalise the URL before persisting so it is always
                    // ready to be used as a base URL (scheme + trailing slash).
                    let url = utils::normalize_db_url(&url);
                    // Persist the new URL in localStorage immediately so that
                    // `get_exercise_db_url()` picks it up when exercises reload.
                    if let Some(window) = web_sys::window() {
                        if let Ok(Some(storage)) = window.local_storage() {
                            if url.is_empty() || url == utils::EXERCISE_DB_BASE_URL {
                                let _ = storage.remove_item(utils::EXERCISE_DB_URL_STORAGE_KEY);
                            } else {
                                let _ = storage.set_item(utils::EXERCISE_DB_URL_STORAGE_KEY, &url);
                            }
                        }
                    }
                    services::exercise_db::clear_fetch_cache();
                    // No navigation needed — the reload will happen via provide_exercises
                }
                DeepLinkAction::StartSession(exercise_ids) => {
                    let mut session = models::WorkoutSession::new();
                    session.pending_exercise_ids = exercise_ids;
                    services::storage::save_session(session);
                    nav.push(Route::Home {});
                }
                // Deferred: needs exercises to be loaded first
                action @ DeepLinkAction::CreateSession(_) => {
                    pending.set(Some(action));
                }
            }
        });

        // ── Deferred: create past session once exercises are available ───────
        //
        // Both `exercises_sig` and `pending` are READ here so that the effect
        // re-fires when either changes.  Using `.write().take()` for `pending`
        // would only write (no reactive subscription), causing the effect to
        // miss the stored action when exercises finish loading.
        use_effect(move || {
            let exercises = exercises_sig.read();
            // Clone the pending action to release the read lock before any write
            let action = { (*pending.read()).clone() };
            let Some(action) = action else {
                return; // nothing pending
            };
            if exercises.is_empty() {
                return; // exercises not loaded yet; will retry when they load
            }
            // Clear the pending action to avoid reprocessing on future re-runs
            pending.set(None);
            if let DeepLinkAction::CreateSession(entries) = action {
                let session = build_session_from_entries(&entries, &exercises);
                services::storage::save_session(session);
            }
        });
    }

    rsx! {
        GlobalSessionHeader {}
        Outlet::<Route> {}
    }
}

/// Convert a deep-link path string such as `"/"` or `"/exercises"` to a [`Route`].
#[cfg(target_arch = "wasm32")]
fn path_to_route(path: &str) -> Route {
    match path {
        "/" | "home" => Route::Home {},
        "/exercises" | "exercises" => Route::Exercises {},
        "/analytics" | "analytics" => Route::Analytics {},
        "/credits" | "credits" => Route::Credits {},
        "/add-exercise" | "add-exercise" => Route::AddExercise {},
        other => {
            // Try to parse /edit-exercise/:id
            if let Some(id) = other.strip_prefix("/edit-exercise/") {
                Route::EditExercise { id: id.to_string() }
            } else {
                Route::Home {}
            }
        }
    }
}

/// Build a completed [`models::WorkoutSession`] from deep-link entries,
/// looking up exercise metadata (name, category, force) from the loaded list.
///
/// The [`utils::SessionExerciseEntry`] URL format uses `weight_hg` for the weight
/// in hectograms and `reps` for repetitions.  For cardio exercises, the `reps`
/// value is reinterpreted as a distance in kilometres (multiplied by 1000 to get
/// metres), since cardio deep-link params typically encode a distance rather than
/// a repetition count.  Strength and static exercises use `reps` directly.
#[cfg(target_arch = "wasm32")]
fn build_session_from_entries(
    entries: &[utils::SessionExerciseEntry],
    exercises: &[models::Exercise],
) -> models::WorkoutSession {
    use models::{Category, Distance, ExerciseLog, Force, Weight, WorkoutSession};

    let base_time = models::get_current_timestamp().saturating_sub(3600); // 1 h ago
    let mut session = WorkoutSession::new();
    session.start_time = base_time;

    for (i, entry) in entries.iter().enumerate() {
        let start = base_time + i as u64 * 120;
        let end = start + 60;

        // Look up exercise metadata; fall back to minimal defaults if not found
        let (name, category, force) = exercises
            .iter()
            .find(|e| e.id == entry.exercise_id)
            .map(|e| (e.name.clone(), e.category, e.force))
            .unwrap_or_else(|| (entry.exercise_id.clone(), Category::Strength, None));

        let weight_hg = entry
            .weight_hg
            .map(|w| Weight(w.min(u16::MAX as u32) as u16));
        let reps = if force.is_some_and(Force::has_reps) {
            entry.reps
        } else {
            None
        };
        let distance_m = if category == Category::Cardio {
            entry.reps.map(|r| Distance(r * 1000))
        } else {
            None
        };

        session.exercise_logs.push(ExerciseLog {
            exercise_id: entry.exercise_id.clone(),
            exercise_name: name,
            category,
            force,
            start_time: start,
            end_time: Some(end),
            weight_hg,
            reps,
            distance_m,
        });
    }

    session.end_time = Some(base_time + entries.len() as u64 * 120 + 60);
    session
}

/// The auto-dismiss timer lives here (always mounted) so it is never cancelled
/// when the `SessionView` unmounts.
#[component]
fn CongratulationsToast() -> Element {
    let mut show = use_context::<CongratulationsSignal>().0;
    // Each time the toast is shown a new generation is stamped.  The dismiss
    // timer captures that generation and only hides the toast if no newer
    // toast has replaced it in the meantime.
    let mut gen = use_signal(|| 0u32);

    // Auto-dismiss: when `show` becomes true, schedule a reset after TOAST_DISMISS_MS.
    use_effect(move || {
        if *show.read() {
            let next = *gen.peek() + 1;
            gen.set(next);
            spawn(async move {
                #[cfg(target_arch = "wasm32")]
                gloo_timers::future::TimeoutFuture::new(TOAST_DISMISS_MS).await;
                #[cfg(not(target_arch = "wasm32"))]
                tokio::time::sleep(std::time::Duration::from_millis(u64::from(
                    TOAST_DISMISS_MS,
                )))
                .await;
                if *gen.peek() == next {
                    show.set(false);
                }
            });
        }
    });

    if *show.read() {
        rsx! {
            div { class: "snackbar",
                onclick: move |_| show.set(false),
                "🎉 Great workout! Session complete!"
            }
        }
    } else {
        rsx! {}
    }
}

/// General-purpose toast component that auto-dismisses after [`TOAST_DISMISS_MS`].
#[component]
fn Toast() -> Element {
    let mut toast = use_context::<ToastSignal>().0;
    // Generation counter: incremented with each new message so that a stale
    // dismiss timer from a previous message cannot hide a newer one.
    let mut gen = use_signal(|| 0u32);

    use_effect(move || {
        if toast.read().is_some() {
            let next = *gen.peek() + 1;
            gen.set(next);
            spawn(async move {
                #[cfg(target_arch = "wasm32")]
                gloo_timers::future::TimeoutFuture::new(TOAST_DISMISS_MS).await;
                #[cfg(not(target_arch = "wasm32"))]
                tokio::time::sleep(std::time::Duration::from_millis(u64::from(
                    TOAST_DISMISS_MS,
                )))
                .await;
                if *gen.peek() == next {
                    toast.set(None);
                }
            });
        }
    });

    let guard = toast.read();
    if let Some(msg) = guard.as_deref() {
        rsx! {
            div { class: "snackbar",
                onclick: move |_| toast.set(None),
                "{msg}"
            }
        }
    } else {
        rsx! {}
    }
}

/// Persistent notification-permission warning toast.
///
/// Shown when notification permission is `default` or `denied`.  Clicking the
/// toast triggers the browser permission dialog (user gesture required by spec).
/// The toast does **not** auto-dismiss so the user can act on it at their pace.
#[component]
fn NotificationPermissionToast() -> Element {
    #[allow(unused_mut)]
    let mut show = use_context::<NotificationPermissionToastSignal>().0;
    if !*show.read() {
        return rsx! {};
    }
    #[cfg(all(target_arch = "wasm32", feature = "web-platform"))]
    {
        use web_sys::NotificationPermission;
        let msg = match web_sys::Notification::permission() {
            NotificationPermission::Denied => "⚠️ Notifications blocked",
            _ => "⚠️ Tap here to enable notifications",
        };
        rsx! {
            div { class: "snackbar",
                onclick: move |_| {
                    show.set(false);
                    if let Ok(promise) = web_sys::Notification::request_permission() {
                        wasm_bindgen_futures::spawn_local(async move {
                            let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                        });
                    }
                },
                "{msg}"
            }
        }
    }

    #[cfg(not(all(target_arch = "wasm32", feature = "web-platform")))]
    rsx! {}
}
