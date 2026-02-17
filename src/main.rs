use dioxus::prelude::*;

mod components;
mod models;
mod services;

use components::{ExerciseListPage, HomePage, WorkoutLogPage, ActiveSessionPage, AddCustomExercisePage};

#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    HomePage {},
    #[route("/exercises")]
    ExerciseListPage {},
    #[route("/workout")]
    WorkoutLogPage {},
    #[route("/session")]
    ActiveSessionPage {},
    #[route("/add-exercise")]
    AddCustomExercisePage {},
}

fn main() {
    // Initialize storage
    services::storage::init_storage();

    // Initialize logger
    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).expect("failed to init logger");

    // Register service worker for offline image caching
    services::service_worker::register_service_worker();

    launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}
