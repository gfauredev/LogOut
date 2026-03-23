use super::storage::native_storage;
use crate::models::{Exercise, WorkoutSession};
use dioxus::prelude::WritableExt;
use dioxus::signals::Signal;
use std::cell::RefCell;
use std::collections::VecDeque;
/// A pending write operation, including the toast signal for error reporting.
pub enum NativeOp {
    /// Upsert a session.  On write failure the sessions signal is reverted to
    /// `previous` (the value before the optimistic update).
    PutSession {
        session: WorkoutSession,
        toast: Signal<std::collections::VecDeque<String>>,
        sessions_sig: Signal<Vec<WorkoutSession>>,
        /// `None` means the session was newly inserted; reverting removes it.
        /// `Some(old)` means it was an update; reverting restores `old`.
        previous: Option<WorkoutSession>,
    },
    /// Delete a session by ID.  On failure the sessions signal is restored
    /// using `snapshot` (if the session was present in the signal).
    DeleteSession {
        id: String,
        toast: Signal<std::collections::VecDeque<String>>,
        sessions_sig: Signal<Vec<WorkoutSession>>,
        /// The session that was removed from the signal, for revert on failure.
        snapshot: Option<WorkoutSession>,
    },
    PutExercise(Exercise, Signal<std::collections::VecDeque<String>>),
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
#[allow(clippy::too_many_lines)]
async fn drain() {
    loop {
        let op = QUEUE.with(|q| q.borrow_mut().1.pop_front());
        match op {
            None => {
                QUEUE.with(|q| q.borrow_mut().0 = false);
                break;
            }
            Some(NativeOp::PutSession {
                session: s,
                mut toast,
                mut sessions_sig,
                previous,
            }) => {
                let id = s.id.clone();
                let result = tokio::task::spawn_blocking(move || {
                    native_storage::put_item(native_storage::STORE_SESSIONS, &s.id, &s)
                })
                .await;
                match result {
                    Ok(Err(e)) => {
                        log::error!("Native queue: failed to put session {id}: {e}");
                        toast
                            .write()
                            .push_back(format!("⚠️ Failed to save session: {e}"));
                        // Revert the optimistic signal update.
                        let mut sessions = sessions_sig.write();
                        match previous {
                            None => sessions.retain(|x| x.id != id),
                            Some(old) => {
                                if let Some(pos) = sessions.iter().position(|x| x.id == id) {
                                    sessions[pos] = old;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Native queue: spawn_blocking panicked for session {id}: {e}");
                        toast
                            .write()
                            .push_back("⚠️ Failed to save session (internal error)".into());
                        // Revert optimistic update even on panic.
                        let mut sessions = sessions_sig.write();
                        match previous {
                            None => sessions.retain(|x| x.id != id),
                            Some(old) => {
                                if let Some(pos) = sessions.iter().position(|x| x.id == id) {
                                    sessions[pos] = old;
                                }
                            }
                        }
                    }
                    Ok(Ok(())) => {}
                }
            }
            Some(NativeOp::DeleteSession {
                id,
                mut toast,
                mut sessions_sig,
                snapshot,
            }) => {
                let id2 = id.clone();
                let result = tokio::task::spawn_blocking(move || {
                    native_storage::delete_item(native_storage::STORE_SESSIONS, &id)
                })
                .await;
                match result {
                    Ok(Err(e)) => {
                        log::error!("Native queue: failed to delete session {id2}: {e}");
                        toast
                            .write()
                            .push_back(format!("⚠️ Failed to delete session: {e}"));
                        if let Some(session) = snapshot {
                            sessions_sig.write().push(session);
                        }
                    }
                    Err(e) => {
                        log::error!("Native queue: spawn_blocking panicked for delete {id2}: {e}");
                        toast
                            .write()
                            .push_back("⚠️ Failed to delete session (internal error)".into());
                        if let Some(session) = snapshot {
                            sessions_sig.write().push(session);
                        }
                    }
                    Ok(Ok(())) => {}
                }
            }
            Some(NativeOp::PutExercise(ex, mut toast)) => {
                let ex_id = ex.id.clone();
                let result = tokio::task::spawn_blocking(move || {
                    native_storage::put_item(native_storage::STORE_CUSTOM_EXERCISES, &ex.id, &ex)
                })
                .await;
                match result {
                    Ok(Err(e)) => {
                        log::error!("Native queue: failed to put exercise {ex_id}: {e}");
                        toast
                            .write()
                            .push_back(format!("⚠️ Failed to save exercise: {e}"));
                    }
                    Err(e) => {
                        log::error!(
                            "Native queue: spawn_blocking panicked for exercise {ex_id}: {e}"
                        );
                        toast
                            .write()
                            .push_back("⚠️ Failed to save exercise (internal error)".into());
                    }
                    Ok(Ok(())) => {}
                }
            }
        }
        tokio::task::yield_now().await;
    }
}
