use super::session_exercise_form::ExerciseInputForm;
use crate::models::{
    format_time, parse_distance_km, parse_duration_seconds, parse_weight_kg, Category, ExerciseLog,
    Force, Weight, WorkoutSession, HG_PER_KG, M_PER_KM,
};
use crate::services::{exercise_db, storage};
use crate::utils::sleep_ms;
use dioxus::prelude::*;
use dioxus_i18n::prelude::i18n;
use dioxus_i18n::t;

/// Horizontal distance in pixels required to trigger edit on swipe-right.
const SWIPE_EDIT_PX: f64 = 56.0;
/// Horizontal distance in pixels required to arm delete on swipe-left.
const SWIPE_DELETE_PX: f64 = 56.0;
/// Maximum visual drag offset applied to the tile while swiping.
const SWIPE_VISUAL_MAX_PX: f64 = 96.0;
/// Maximum horizontal movement still considered a tap.
const TAP_SLOP_PX: f64 = 14.0;
/// Delete hold duration in 100 ms ticks (30 × 100 ms = 3 s).
const DELETE_HOLD_STEPS: u32 = 30;
/// `DELETE_HOLD_STEPS` as `f32` for progress computations.
const DELETE_HOLD_STEPS_F32: f32 = 30.0;
/// Duration of each hold tick in milliseconds.
const DELETE_HOLD_TICK_MS: u32 = 100;
/// A single completed exercise log entry with inline edit support.
#[component]
pub fn CompletedExerciseLog(
    idx: usize,
    log: ExerciseLog,
    session: Memo<WorkoutSession>,
    /// Called when the user taps the tile to start another set.
    #[props(default)]
    on_replay: EventHandler<()>,
    /// Whether tap-to-replay is enabled (only in an active session with no exercise in progress).
    #[props(default)]
    show_replay: bool,
) -> Element {
    let mut is_editing = use_signal(|| false);
    let mut pointer_start_x = use_signal(|| None::<f64>);
    let mut pointer_down = use_signal(|| false);
    let mut drag_delta_x = use_signal(|| 0.0f64);
    let mut delete_armed = use_signal(|| false);
    let mut delete_progress = use_signal(|| 0.0f32);
    // Gesture generation token used to cancel in-flight hold tasks.
    let mut delete_hold_gen = use_signal(|| 0u32);
    let mut edit_weight_input = use_signal(String::new);
    let mut edit_reps_input = use_signal(String::new);
    let mut edit_distance_input = use_signal(String::new);
    let mut edit_time_input = use_signal(String::new);
    let mut toast = consume_context::<crate::ToastSignal>().0;
    let mut start_edit = {
        let log = log.clone();
        move |()| {
            edit_weight_input.set(if log.weight_hg.0 == 0 {
                String::new()
            } else {
                format!("{:.1}", f64::from(log.weight_hg.0) / HG_PER_KG)
            });
            edit_reps_input.set(log.reps.map(|r| r.to_string()).unwrap_or_default());
            edit_distance_input.set(
                log.distance_m
                    .map(|d| format!("{:.2}", f64::from(d.0) / M_PER_KM))
                    .unwrap_or_default(),
            );
            edit_time_input.set(log.duration_seconds().map(format_time).unwrap_or_default());
            is_editing.set(true);
        }
    };
    let all_exercises = exercise_db::use_exercises();
    let custom_exercises = storage::use_custom_exercises();
    let lang_str = use_memo(move || i18n().language().to_string());
    let exercise_id_for_name = log.exercise_id.clone();
    let fallback_name = log.exercise_name.clone();
    let display_name = use_memo(move || {
        let all = all_exercises.read();
        let custom = custom_exercises.read();
        let lang = lang_str.read();
        exercise_db::resolve_exercise(&all, &custom, &exercise_id_for_name).map_or_else(
            || fallback_name.clone(),
            |ex| ex.name_for_lang(&lang).to_owned(),
        )
    });
    let force = log.force;
    let category = log.category;
    let exercise_id = log.exercise_id.clone();
    let display_dx = drag_delta_x
        .read()
        .clamp(-SWIPE_VISUAL_MAX_PX, SWIPE_VISUAL_MAX_PX);
    rsx! {
        article {
            class: "log log-tile",
            style: "transform: translateX({display_dx}px);",
            onpointerdown: move |evt| {
                if *is_editing.read() {
                    return;
                }
                let next = delete_hold_gen.peek().wrapping_add(1);
                delete_hold_gen.set(next);
                pointer_down.set(true);
                pointer_start_x.set(Some(evt.client_coordinates().x));
                drag_delta_x.set(0.0);
                delete_armed.set(false);
                delete_progress.set(0.0);
            },
            onpointermove: move |evt| {
                if *is_editing.read() || !*pointer_down.read() {
                    return;
                }
                let Some(start_x) = *pointer_start_x.read() else {
                    return;
                };
                let dx = evt.client_coordinates().x - start_x;
                drag_delta_x.set(dx);
                if dx <= -SWIPE_DELETE_PX && !*delete_armed.read() {
                    delete_armed.set(true);
                    let gen = delete_hold_gen.peek().wrapping_add(1);
                    delete_hold_gen.set(gen);
                    spawn(async move {
                        let step = 1.0_f32 / DELETE_HOLD_STEPS_F32;
                        let mut cur = 0.0_f32;
                        for _ in 0..DELETE_HOLD_STEPS {
                            sleep_ms(DELETE_HOLD_TICK_MS).await;
                            if *delete_hold_gen.peek() != gen
                                || !*pointer_down.peek()
                                || *drag_delta_x.peek() > -SWIPE_DELETE_PX
                            {
                                delete_progress.set(0.0);
                                return;
                            }
                            cur += step;
                            delete_progress.set(cur);
                        }
                        if *delete_hold_gen.peek() == gen
                            && *pointer_down.peek()
                            && *drag_delta_x.peek() <= -SWIPE_DELETE_PX
                        {
                            toast.write().push_back(t!("toast-log-deleted").to_string());
                            let mut current_session = session.read().clone();
                            current_session.exercise_logs.remove(idx);
                            storage::save_session(current_session);
                        }
                        delete_progress.set(0.0);
                    });
                } else if dx > -SWIPE_DELETE_PX && *delete_armed.read() {
                    delete_armed.set(false);
                    delete_progress.set(0.0);
                    let next = delete_hold_gen.peek().wrapping_add(1);
                    delete_hold_gen.set(next);
                }
            },
            onpointerup: move |_| {
                if *is_editing.read() {
                    return;
                }
                let dx = *drag_delta_x.read();
                let armed_delete = *delete_armed.read();
                let completed_delete = *delete_progress.read() >= 1.0;
                pointer_down.set(false);
                pointer_start_x.set(None);
                drag_delta_x.set(0.0);
                delete_armed.set(false);
                delete_progress.set(0.0);
                let next = delete_hold_gen.peek().wrapping_add(1);
                delete_hold_gen.set(next);
                if completed_delete {
                    return;
                }
                if armed_delete {
                    toast.write().push_back(t!("hold-to-delete-hint").to_string());
                    return;
                }
                if dx >= SWIPE_EDIT_PX {
                    start_edit(());
                    return;
                }
                if show_replay && dx.abs() <= TAP_SLOP_PX {
                    on_replay.call(());
                }
            },
            onpointerleave: move |_| {
                if *is_editing.read() {
                    return;
                }
                pointer_down.set(false);
                pointer_start_x.set(None);
                drag_delta_x.set(0.0);
                delete_armed.set(false);
                delete_progress.set(0.0);
                let next = delete_hold_gen.peek().wrapping_add(1);
                delete_hold_gen.set(next);
            },
            onpointercancel: move |_| {
                if *is_editing.read() {
                    return;
                }
                pointer_down.set(false);
                pointer_start_x.set(None);
                drag_delta_x.set(0.0);
                delete_armed.set(false);
                delete_progress.set(0.0);
                let next = delete_hold_gen.peek().wrapping_add(1);
                delete_hold_gen.set(next);
            },
            title: t!("log-replay-title"),
            "aria-label": t!("log-replay-title"),
            header {
                h4 { "{display_name}" }
                ul { class: "log-stats",
                    if log.weight_hg.0 > 0 {
                        li { "{log.weight_hg}" }
                    }
                    if let Some(reps) = log.reps {
                        li { "{reps} reps" }
                    }
                    if let Some(d) = log.distance_m {
                        li { "{d}" }
                    }
                    if let Some(duration) = log.duration_seconds() {
                        li { "{crate::models::format_time(duration)}" }
                    }
                }
            }
            if !*is_editing.read() {
                if *delete_armed.read() || *delete_progress.read() > 0.0 {
                    div {
                        class: "log-delete-progress",
                        style: "width: {(*delete_progress.read() * 100.0).clamp(0.0, 100.0)}%;",
                    }
                }
            }
            if *is_editing.read() {
                ExerciseInputForm {
                    exercise_id,
                    exercise_name: log.exercise_name.clone(),
                    weight_input: edit_weight_input,
                    reps_input: edit_reps_input,
                    distance_input: edit_distance_input,
                    force,
                    category,
                    time_input: Some(edit_time_input),
                    on_complete: move |()| {
                        let mut current_session = session.read().clone();
                        if let Some(log) = current_session.exercise_logs.get_mut(idx) {
                            log.weight_hg = if category == Category::Stretching {
                                Weight::default()
                            } else {
                                parse_weight_kg(&edit_weight_input.read()).unwrap_or_default()
                            };
                            log.reps = if category != Category::Cardio
                                && force.is_some_and(Force::has_reps)
                            {
                                edit_reps_input.read().parse().ok()
                            } else {
                                None
                            };
                            if log.category == Category::Cardio {
                                log.distance_m = parse_distance_km(&edit_distance_input.read());
                            }
                            let time_str = edit_time_input.read();
                            if !time_str.is_empty() {
                                if let Some(dur) = parse_duration_seconds(&time_str) {
                                    log.end_time = Some(log.start_time + dur);
                                }
                            }
                        }
                        storage::save_session(current_session);
                        is_editing.set(false);
                        edit_weight_input.set(String::new());
                        edit_reps_input.set(String::new());
                        edit_distance_input.set(String::new());
                        edit_time_input.set(String::new());
                    },
                    on_cancel: move |()| is_editing.set(false),
                }
            }
        }
    }
}
