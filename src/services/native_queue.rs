use super::storage::native_storage;
use crate::models::{Exercise, WorkoutSession};
use dioxus::prelude::WritableExt;
use dioxus::signals::Signal;
use std::cell::RefCell;
use std::collections::VecDeque;
/// A pending write operation, including the toast signal for error reporting.
pub enum NativeOp {
    PutSession(WorkoutSession, Signal<Option<String>>),
    DeleteSession(String, Signal<Option<String>>),
    PutExercise(Exercise, Signal<Option<String>>),
}
thread_local! {
    /// (draining, pending_ops)
    static QUEUE: RefCell<(bool, VecDeque<NativeOp>)> = const {
        RefCell::new((false, VecDeque::new()))
    };
}
/// Enqueue a write operation. If no drain is currently running, starts one.
pub fn enqueue(op: NativeOp) {
    QUEUE.with(|q| {
        let mut q = q.borrow_mut();
        q.1.push_back(op);
        if !q.0 {
            q.0 = true;
            dioxus::prelude::spawn(drain());
        }
    });
}
async fn drain() {
    loop {
        let op = QUEUE.with(|q| q.borrow_mut().1.pop_front());
        match op {
            None => {
                QUEUE.with(|q| q.borrow_mut().0 = false);
                break;
            }
            Some(NativeOp::PutSession(s, mut toast)) => {
                if let Err(e) = native_storage::put_item(native_storage::STORE_SESSIONS, &s.id, &s)
                {
                    log::error!("Native queue: failed to put session {}: {e}", s.id);
                    toast.set(Some(format!("⚠️ Failed to save session: {e}")));
                }
            }
            Some(NativeOp::DeleteSession(id, mut toast)) => {
                if let Err(e) = native_storage::delete_item(native_storage::STORE_SESSIONS, &id) {
                    log::error!("Native queue: failed to delete session {id}: {e}");
                    toast.set(Some(format!("⚠️ Failed to delete session: {e}")));
                }
            }
            Some(NativeOp::PutExercise(ex, mut toast)) => {
                if let Err(e) =
                    native_storage::put_item(native_storage::STORE_CUSTOM_EXERCISES, &ex.id, &ex)
                {
                    log::error!("Native queue: failed to put exercise {}: {e}", ex.id);
                    toast.set(Some(format!("⚠️ Failed to save exercise: {e}")));
                }
            }
        }
        tokio::task::yield_now().await;
    }
}
