use dioxus::prelude::*;

mod components;
mod models;
mod services;
pub mod utils;

use components::{
    AddCustomExercisePage, AnalyticsPage, CreditsPage, EditCustomExercisePage, ExerciseListPage,
    HomePage,
};

/// Snackbar auto-dismiss delay in milliseconds
const SNACKBAR_DISMISS_MS: u32 = 3_000;

/// Global context signal for the congratulations toast shown after completing a session.
#[derive(Clone, Copy)]
pub struct CongratulationsSignal(pub Signal<bool>);

/// Global context signal for the notification permission warning toast.
#[derive(Clone, Copy)]
pub struct NotificationWarningSignal(pub Signal<bool>);

/// Global context signal for focusing an exercise in the exercise list (by name).
#[derive(Clone, Copy)]
pub struct ExerciseFocusSignal(pub Signal<Option<String>>);

#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
#[allow(clippy::enum_variant_names)]
enum Route {
    #[route("/")]
    HomePage {},
    #[route("/exercises")]
    ExerciseListPage {},
    #[route("/analytics")]
    AnalyticsPage {},
    #[route("/credits")]
    CreditsPage {},
    #[route("/add-exercise")]
    AddCustomExercisePage {},
    #[route("/edit-exercise/:id")]
    EditCustomExercisePage { id: String },
}

fn main() {
    // Initialize logger
    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).expect("failed to init logger");

    // Register service worker for offline image caching
    services::service_worker::register_service_worker();

    // Prevent the device screen from sleeping while the app is open
    services::wake_lock::enable_wake_lock();

    // Request notification permission so rest/duration alerts can be shown
    #[cfg(all(target_arch = "wasm32", feature = "web-platform"))]
    {
        use web_sys::NotificationPermission;
        if web_sys::Notification::permission() == NotificationPermission::Default {
            let _ = web_sys::Notification::request_permission();
        }
    }

    launch(App);
}

#[component]
fn App() -> Element {
    // Provide shared state signals via context
    services::storage::provide_app_state();
    services::exercise_db::provide_exercises();
    use_context_provider(|| CongratulationsSignal(Signal::new(false)));
    use_context_provider(|| NotificationWarningSignal(Signal::new(false)));
    use_context_provider(|| ExerciseFocusSignal(Signal::new(None)));

    // Show a warning toast if notification permission is denied on app start
    #[cfg(all(target_arch = "wasm32", feature = "web-platform"))]
    {
        let mut warn = use_context::<NotificationWarningSignal>().0;
        use_effect(move || {
            use web_sys::{Notification, NotificationPermission};
            if Notification::permission() == NotificationPermission::Denied {
                warn.set(true);
                spawn(async move {
                    gloo_timers::future::TimeoutFuture::new(SNACKBAR_DISMISS_MS).await;
                    warn.set(false);
                });
            }
        });
    }

    rsx! {
        Stylesheet { href: asset!("/assets/styles.css") }
        Router::<Route> {}
        CongratulationsToast {}
        NotificationWarningToast {}
    }
}

/// Renders the congratulations toast when a session is successfully completed.
#[component]
fn CongratulationsToast() -> Element {
    let show = use_context::<CongratulationsSignal>().0;
    if *show.read() {
        rsx! { Snackbar { text: "🎉 Great workout! Session complete!", signal: show } }
    } else {
        rsx! {}
    }
}

/// Renders a warning toast when notification permission is denied.
#[component]
fn NotificationWarningToast() -> Element {
    let show = use_context::<NotificationWarningSignal>().0;
    if *show.read() {
        rsx! {
            Snackbar {
                text: "⚠️ Notifications blocked – rest alerts won't be shown",
                signal: show,
                warning: true,
            }
        }
    } else {
        rsx! {}
    }
}

/// Common snackbar/toast component. Dismisses on click.
/// Auto-dismiss is handled by the caller via the shared signal.
#[component]
fn Snackbar(text: &'static str, signal: Signal<bool>, #[props(default)] warning: bool) -> Element {
    let mut sig = signal;
    rsx! {
        div {
            class: if warning { "snackbar snackbar--warning" } else { "snackbar" },
            onclick: move |_| sig.set(false),
            "{text}"
        }
    }
}
