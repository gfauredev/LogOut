use dioxus::prelude::*;

mod components;
mod models;
mod services;
pub mod utils;

use components::{ExerciseListPage, HomePage, AnalyticsPage, AddCustomExercisePage, EditCustomExercisePage};

#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    HomePage {},
    #[route("/exercises")]
    ExerciseListPage {},
    #[route("/analytics")]
    AnalyticsPage {},
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

    launch(App);
}

#[component]
fn App() -> Element {
    // Provide shared state signals via context
    services::storage::provide_app_state();
    services::exercise_db::provide_exercises();

    rsx! {
        Stylesheet { href: asset!("/assets/styles.css") }
        Router::<Route> {}
    }
}
