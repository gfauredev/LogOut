use crate::utils::sleep_ms;
use crate::ToastSignal;
use dioxus::prelude::*;
use dioxus_i18n::t;

/// Number of 100 ms ticks that must elapse while the button is held before the
/// delete action fires (30 × 100 ms = 3 s).
const HOLD_STEPS: u32 = 30;
/// `HOLD_STEPS` as `f32` for progress computations; no precision loss for this
/// small value.
const HOLD_STEPS_F32: f32 = 30.0;
/// Duration of each tick in milliseconds.
const HOLD_TICK_MS: u32 = 100;
/// SVG viewBox half-side (the SVG is `RING_SIZE × RING_SIZE`).
const RING_SIZE: f32 = 44.0;
/// Circle radius used for the progress ring (leaves room for the stroke).
const RING_RADIUS: f32 = 19.0;
/// Stroke-dasharray / full circumference of the progress ring.
const RING_CIRC: f32 = 2.0 * std::f32::consts::PI * RING_RADIUS; // ≈ 119.4

/// A delete button that requires the user to hold it for 3 seconds before
/// firing `on_delete`.  While the button is held a circular SVG progress ring
/// fills around it.  If the button is released early a toast hint is shown.
#[component]
pub fn HoldDeleteButton(on_delete: EventHandler<()>, title: String) -> Element {
    let mut progress = use_signal(|| 0.0f32);
    // Generation counter: incremented on each press and on each early release.
    // The spawned task captures its generation and exits as soon as it drifts.
    let mut gen = use_signal(|| 0u32);

    let hint_msg = t!("hold-to-delete-hint").to_string();

    let offset = RING_CIRC * (1.0 - *progress.read());

    rsx! {
        div { class: "hold-del",
            svg {
                class: "hold-del-ring",
                view_box: "0 0 {RING_SIZE} {RING_SIZE}",
                "aria-hidden": "true",
                circle {
                    cx: "{RING_SIZE / 2.0}",
                    cy: "{RING_SIZE / 2.0}",
                    r: "{RING_RADIUS}",
                    "stroke-dasharray": "{RING_CIRC}",
                    "stroke-dashoffset": "{offset}",
                }
            }
            button {
                class: "del",
                title,
                onpointerdown: move |_| {
                    let next = gen.peek().wrapping_add(1);
                    gen.set(next);
                    let hint = hint_msg.clone();
                    let mut toast = consume_context::<ToastSignal>().0;
                    spawn(async move {
                        let increment = 1.0_f32 / HOLD_STEPS_F32;
                        let mut cur_progress = 0.0_f32;
                        for _ in 0..HOLD_STEPS {
                            sleep_ms(HOLD_TICK_MS).await;
                            if *gen.peek() != next {
                                // Released early – show the hint toast.
                                toast.write().push_back(hint);
                                progress.set(0.0);
                                return;
                            }
                            cur_progress += increment;
                            progress.set(cur_progress);
                        }
                        // Full 3 s elapsed – fire the delete action.
                        if *gen.peek() == next {
                            on_delete.call(());
                        }
                        progress.set(0.0);
                    });
                },
                onpointerup: move |_| {
                    let next = gen.peek().wrapping_add(1);
                    gen.set(next);
                },
                onpointerleave: move |_| {
                    let next = gen.peek().wrapping_add(1);
                    gen.set(next);
                },
                "🗑️"
            }
        }
    }
}
