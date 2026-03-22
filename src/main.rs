//! **`LogOut`** – Turn off your computer, Log your workOut
//!
//! A simple, efficient and cross-platform workout logging application with
//! 800+ built-in exercises.  The app is built with [Dioxus] and targets both
//! PWA (web) and native Android / desktop platforms.
//!
//! [Dioxus]: https://dioxuslabs.com
use dioxus::prelude::*;
use dioxus_i18n::prelude::*;
use unic_langid::langid;
mod components;
mod models;
mod services;
/// Pure utility helpers (date formatting, URL resolution, timestamp helpers).
pub mod utils;
use components::{
    AddExercise, Analytics, EditExercise, Exercises, GlobalSessionHeader, Home, More,
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
/// Global context signal holding the configured rest duration (in seconds).
/// Shared between [`GlobalSessionHeader`] (which reads/displays it) and the
/// rest-duration input form that updates it.
#[derive(Clone, Copy)]
pub struct RestDurationSignal(pub Signal<u64>);
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
/// Global context signal for enum-value translations loaded from `i18n.json`.
/// Provides translated labels for category, force, equipment, level and muscle
/// names in the user's preferred language.
#[derive(Clone, Copy)]
pub struct DbI18nSignal(pub Signal<models::DbI18n>);
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
    #[route("/more")]
    More {},
    #[route("/add-exercise")]
    AddExercise {},
    #[route("/edit-exercise/:id")]
    EditExercise { id: String },
}
/// Detects the user's preferred language from the browser/system, returning a
/// `LanguageIdentifier`.  Falls back to English (`"en"`) when the language
/// cannot be determined or is not one the app supports.
fn detect_preferred_language() -> unic_langid::LanguageIdentifier {
    #[cfg(target_arch = "wasm32")]
    if let Some(lang_str) = web_sys::window().and_then(|w| w.navigator().language()) {
        if let Ok(id) = lang_str.parse() {
            return id;
        }
        if let Some(base) = lang_str.split('-').next() {
            if let Ok(id) = base.parse() {
                return id;
            }
        }
    }
    langid!("en")
}
fn main() {
    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).expect("failed to init logger");
    #[cfg(target_os = "android")]
    {
        services::android_notifications::setup_notification_channel();
    }
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
    {}
    services::service_worker::register_service_worker();
    services::wake_lock::enable_wake_lock();
    launch(App);
}
#[component]
fn App() -> Element {
    use_init_i18n(|| {
        let preferred_lang = detect_preferred_language();
        I18nConfig::new(preferred_lang)
            .with_locale((langid!("en"), include_str!("../assets/en.ftl")))
            .with_locale((langid!("fr"), include_str!("../assets/fr.ftl")))
            .with_fallback(langid!("en"))
    });
    services::storage::provide_app_state();
    #[cfg(target_arch = "wasm32")]
    use_hook(|| {
        services::storage::idb_queue::register_pagehide_flush();
    });
    use_context_provider(|| DbI18nSignal(Signal::new(models::DbI18n::default())));
    services::exercise_db::provide_exercises();
    use_context_provider(|| CongratulationsSignal(Signal::new(false)));
    use_context_provider(|| ToastSignal(Signal::new(None)));
    use_context_provider(|| NotificationPermissionToastSignal(Signal::new(false)));
    use_context_provider(|| ExerciseSearchSignal(Signal::new(None)));
    use_context_provider(|| PendingDeepLinkSignal(Signal::new(None)));
    use_context_provider(|| ShowRestInputSignal(Signal::new(false)));
    use_context_provider(|| RestDurationSignal(Signal::new(30u64)));
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
                    let url = utils::normalize_db_url(&url);
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
                    let toast = consume_context::<ToastSignal>().0;
                    spawn(async move {
                        services::exercise_db::reload_exercises(exercises_sig, toast).await;
                    });
                }
                DeepLinkAction::StartSession(exercise_ids) => {
                    pending.set(Some(DeepLinkAction::StartSession(exercise_ids)));
                }
                action @ DeepLinkAction::CreateSession(_) => {
                    pending.set(Some(action));
                }
            }
        });
        use_effect(move || {
            let exercises = exercises_sig.read();
            let action = { (*pending.read()).clone() };
            let Some(action) = action else {
                return;
            };
            if exercises.is_empty() {
                return;
            }
            pending.set(None);
            match action {
                DeepLinkAction::CreateSession(entries) => {
                    let session = build_session_from_entries(&entries, &exercises);
                    services::storage::save_session(session);
                }
                DeepLinkAction::StartSession(exercise_ids) => {
                    let known_ids: std::collections::HashSet<&str> =
                        exercises.iter().map(|e| e.id.as_str()).collect();
                    let valid_ids: Vec<String> = exercise_ids
                        .into_iter()
                        .filter(|id| known_ids.contains(id.as_str()))
                        .collect();
                    let mut session = models::WorkoutSession::new();
                    session.pending_exercise_ids = valid_ids;
                    services::storage::save_session(session);
                    nav.push(Route::Home {});
                }
                _ => {}
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
        "/credits" | "credits" | "/more" | "more" => Route::More {},
        "/add-exercise" | "add-exercise" => Route::AddExercise {},
        other => {
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
/// value is reinterpreted directly as a distance in metres, since cardio
/// deep-link params typically encode a distance rather than a repetition count.
/// Strength and static exercises use `reps` directly.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn build_session_from_entries(
    entries: &[utils::SessionExerciseEntry],
    exercises: &[models::Exercise],
) -> models::WorkoutSession {
    use models::{Category, Distance, ExerciseLog, Force, Weight, WorkoutSession};
    let base_time = models::get_current_timestamp().saturating_sub(3600);
    let mut session = WorkoutSession::new();
    session.start_time = base_time;
    for (i, entry) in entries.iter().enumerate() {
        let start = base_time + i as u64 * 120;
        let end = start + 60;
        let (name, category, force) = exercises
            .iter()
            .find(|e| e.id == entry.exercise_id)
            .map_or_else(
                || (entry.exercise_id.clone(), Category::Strength, None),
                |e| (e.name.clone(), e.category, e.force),
            );
        #[allow(clippy::cast_possible_truncation)]
        let weight_hg = entry
            .weight_hg
            .map(|w| Weight(w.min(u32::from(u16::MAX)) as u16));
        let reps = if force.is_some_and(Force::has_reps) {
            entry.reps
        } else {
            None
        };
        let distance_m = if category == Category::Cardio {
            entry.reps.map(Distance)
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
    let mut gen = use_signal(|| 0u32);
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
            div { class: "snackbar", onclick: move |_| show.set(false),
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
            div { class: "snackbar", onclick: move |_| toast.set(None), "{msg}" }
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
            div {
                class: "snackbar",
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
#[cfg(test)]
mod tests {
    use super::*;
    use models::{Category, Exercise, Force};
    fn sample_exercises() -> Vec<Exercise> {
        vec![
            Exercise {
                id: "Wide-Grip_Barbell_Bench_Press".into(),
                name: "Wide-Grip Barbell Bench Press".into(),
                name_lower: String::new(),
                force: Some(Force::Push),
                level: None,
                mechanic: None,
                equipment: None,
                primary_muscles: vec![],
                secondary_muscles: vec![],
                instructions: vec![],
                category: Category::Strength,
                images: vec![],
                i18n: None,
            },
            Exercise {
                id: "Barbell_Full_Squat".into(),
                name: "Barbell Full Squat".into(),
                name_lower: String::new(),
                force: Some(Force::Push),
                level: None,
                mechanic: None,
                equipment: None,
                primary_muscles: vec![],
                secondary_muscles: vec![],
                instructions: vec![],
                category: Category::Strength,
                images: vec![],
                i18n: None,
            },
            Exercise {
                id: "Running".into(),
                name: "Running".into(),
                name_lower: String::new(),
                force: None,
                level: None,
                mechanic: None,
                equipment: None,
                primary_muscles: vec![],
                secondary_muscles: vec![],
                instructions: vec![],
                category: Category::Cardio,
                images: vec![],
                i18n: None,
            },
        ]
    }
    #[test]
    fn build_session_from_dl_entries_strength() {
        let exercises = sample_exercises();
        let entries = utils::parse_session_exercises(
            "Wide-Grip_Barbell_Bench_Press:80:10,Barbell_Full_Squat:60:6",
        );
        let session = build_session_from_entries(&entries, &exercises);
        assert_eq!(session.exercise_logs.len(), 2);
        assert_eq!(
            session.exercise_logs[0].exercise_name,
            "Wide-Grip Barbell Bench Press",
        );
        assert_eq!(
            session.exercise_logs[0].weight_hg,
            Some(models::Weight(800))
        );
        assert_eq!(session.exercise_logs[0].reps, Some(10));
        assert_eq!(session.exercise_logs[1].exercise_name, "Barbell Full Squat");
        assert_eq!(
            session.exercise_logs[1].weight_hg,
            Some(models::Weight(600))
        );
        assert_eq!(session.exercise_logs[1].reps, Some(6));
        assert!(session.end_time.is_some());
    }
    #[test]
    fn build_session_from_dl_entries_cardio_uses_distance() {
        let exercises = sample_exercises();
        let entries = utils::parse_session_exercises("Running:-:5");
        let session = build_session_from_entries(&entries, &exercises);
        assert_eq!(session.exercise_logs.len(), 1);
        let log = &session.exercise_logs[0];
        assert_eq!(log.exercise_name, "Running");
        assert_eq!(log.category, Category::Cardio);
        assert_eq!(log.distance_m, Some(models::Distance(5)));
        assert_eq!(log.reps, None);
    }
    #[test]
    fn build_session_from_dl_entries_unknown_exercise_falls_back() {
        let exercises = sample_exercises();
        let entries = utils::parse_session_exercises("Unknown_Exercise:50:8");
        let session = build_session_from_entries(&entries, &exercises);
        assert_eq!(session.exercise_logs.len(), 1);
        assert_eq!(session.exercise_logs[0].exercise_name, "Unknown_Exercise");
        assert_eq!(session.exercise_logs[0].category, Category::Strength);
    }
    #[test]
    fn build_session_from_dl_entries_empty() {
        let exercises = sample_exercises();
        let entries = utils::parse_session_exercises("");
        let session = build_session_from_entries(&entries, &exercises);
        assert!(session.exercise_logs.is_empty());
        assert!(session.end_time.is_some());
    }
    #[test]
    fn build_session_from_dl_entries_duplicate_exercises() {
        let exercises = sample_exercises();
        let entries = utils::parse_session_exercises(
            "Wide-Grip_Barbell_Bench_Press:80:10,Wide-Grip_Barbell_Bench_Press:77.5:10,Barbell_Full_Squat:60:6",
        );
        let session = build_session_from_entries(&entries, &exercises);
        assert_eq!(session.exercise_logs.len(), 3);
        assert_eq!(
            session.exercise_logs[0].exercise_name,
            "Wide-Grip Barbell Bench Press",
        );
        assert_eq!(
            session.exercise_logs[0].weight_hg,
            Some(models::Weight(800))
        );
        assert_eq!(
            session.exercise_logs[1].exercise_name,
            "Wide-Grip Barbell Bench Press",
        );
        assert_eq!(
            session.exercise_logs[1].weight_hg,
            Some(models::Weight(775))
        );
    }
}
