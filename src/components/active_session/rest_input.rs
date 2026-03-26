use dioxus::prelude::*;

/// Collapsible form for configuring the rest duration between sets.
#[component]
pub fn RestDurationInput(
    mut show_rest_input: Signal<bool>,
    mut rest_input_value: Signal<String>,
    mut rest_duration: Signal<u64>,
) -> Element {
    rsx! {
        form {
            class: "inputs",
            aria_label: "Set rest duration",
            onsubmit: move |evt| {
                evt.prevent_default();
                if let Ok(val) = rest_input_value.read().parse::<u64>() {
                    rest_duration.set(val);
                }
                show_rest_input.set(false);
            },
            label { r#for: "rest-duration-field", "Rest duration" }
            input {
                id: "rest-duration-field",
                r#type: "number",
                inputmode: "numeric",
                value: "{rest_input_value}",
                oninput: move |evt| rest_input_value.set(evt.value()),
            }
            button { class: "yes", r#type: "submit", "💾" }
        }
    }
}
