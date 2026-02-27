use dioxus::prelude::*;

mod components;
mod models;
mod services;
pub mod utils;

use components::{
    AddCustomExercisePage, AnalyticsPage, CreditsPage, EditCustomExercisePage, ExerciseListPage,
    HomePage,
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

/// Auto-dismiss delay for toasts in milliseconds.
const TOAST_DISMISS_MS: u32 = 3_000;

/// Global context signal for pre-filling the exercise list search query.
#[derive(Clone, Copy)]
pub struct ExerciseSearchSignal(pub Signal<Option<String>>);

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
        Stylesheet { href: asset!("/assets/styles.css") }
        Router::<Route> {}
        CongratulationsToast {}
        Toast {}
        NotificationPermissionToast {}
    }
}

/// Renders the congratulations toast when a session is successfully completed.
/// The auto-dismiss timer lives here (always mounted) so it is never cancelled
/// when the SessionView unmounts.
#[component]
fn CongratulationsToast() -> Element {
    let mut show = use_context::<CongratulationsSignal>().0;

    // Auto-dismiss: when `show` becomes true, schedule a reset after TOAST_DISMISS_MS.
    use_effect(move || {
        if *show.read() {
            spawn(async move {
                #[cfg(target_arch = "wasm32")]
                gloo_timers::future::TimeoutFuture::new(TOAST_DISMISS_MS).await;
                #[cfg(not(target_arch = "wasm32"))]
                tokio::time::sleep(std::time::Duration::from_millis(TOAST_DISMISS_MS as u64)).await;
                show.set(false);
            });
        }
    });

    if *show.read() {
        rsx! {
            div {
                class: "snackbar",
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

    use_effect(move || {
        if toast.read().is_some() {
            spawn(async move {
                #[cfg(target_arch = "wasm32")]
                gloo_timers::future::TimeoutFuture::new(TOAST_DISMISS_MS).await;
                #[cfg(not(target_arch = "wasm32"))]
                tokio::time::sleep(std::time::Duration::from_millis(TOAST_DISMISS_MS as u64)).await;
                toast.set(None);
            });
        }
    });

    let msg = toast.read().clone();
    if let Some(msg) = msg {
        rsx! {
            div {
                class: "snackbar",
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
    let show = use_context::<NotificationPermissionToastSignal>().0;

    if !*show.read() {
        return rsx! {};
    }

    #[cfg(all(target_arch = "wasm32", feature = "web-platform"))]
    {
        use web_sys::NotificationPermission;
        let msg = match web_sys::Notification::permission() {
            NotificationPermission::Denied => {
                "⚠️ Notifications blocked — re-enable in browser settings for timer alerts"
            }
            _ => "⚠️ Tap here to enable notifications for timer alerts",
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
