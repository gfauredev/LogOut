use crate::models::{Exercise, WorkoutSession};
use dioxus::prelude::*;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, OnceLock};

/// A pending write operation. Data only, no signals.
pub enum NativeOp {
    PutSession {
        session: WorkoutSession,
        previous: Option<WorkoutSession>,
    },
    DeleteSession {
        id: String,
        snapshot: Option<WorkoutSession>,
    },
    PutExercise(Exercise),
}

/// Result of a native operation, to be sent back to the UI.
pub enum NativeResult {
    PutSession {
        id: String,
        result: Result<(), String>,
        previous: Option<WorkoutSession>,
    },
    DeleteSession {
        id: String,
        result: Result<(), String>,
        snapshot: Option<WorkoutSession>,
    },
    PutExercise {
        id: String,
        result: Result<(), String>,
    },
}

struct QueueState {
    draining: bool,
    pending: VecDeque<NativeOp>,
}

fn queue() -> &'static Arc<Mutex<QueueState>> {
    static QUEUE: OnceLock<Arc<Mutex<QueueState>>> = OnceLock::new();
    QUEUE.get_or_init(|| {
        Arc::new(Mutex::new(QueueState {
            draining: false,
            pending: VecDeque::new(),
        }))
    })
}

type ResultChannel = (
    tokio::sync::mpsc::UnboundedSender<NativeResult>,
    Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<NativeResult>>>,
);

/// Global channel for reporting results back to the UI.
static RESULT_CHANNEL: OnceLock<ResultChannel> = OnceLock::new();

fn get_result_channel() -> &'static ResultChannel {
    RESULT_CHANNEL.get_or_init(|| {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        (tx, Mutex::new(Some(rx)))
    })
}

pub fn enqueue(op: NativeOp) {
    let mut q = queue().lock().unwrap();
    q.pending.push_back(op);
    if !q.draining {
        q.draining = true;
        tokio::spawn(drain());
    }
}

/// Hook to listen for native operation results and update signals.
pub fn use_native_results() {
    let mut toast = use_context::<crate::ToastSignal>().0;
    let mut sessions_sig = use_context::<Signal<Vec<WorkoutSession>>>();

    use_resource(move || async move {
        let rx = {
            let mut lock = get_result_channel().1.lock().unwrap();
            lock.take()
        };

        if let Some(mut rx) = rx {
            while let Some(res) = rx.recv().await {
                match res {
                    NativeResult::PutSession {
                        id,
                        result,
                        previous,
                    } => match result {
                        Ok(()) => {
                            log::info!("Successfully saved session {id}");
                        }
                        Err(e) => {
                            log::error!("Failed to save session {id}: {e}");
                            toast
                                .write()
                                .push_back(format!("⚠️ Failed to save session: {e}"));
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
                    },
                    NativeResult::DeleteSession {
                        id,
                        result,
                        snapshot,
                    } => match result {
                        Ok(()) => {
                            log::info!("Successfully deleted session {id}");
                        }
                        Err(e) => {
                            log::error!("Failed to delete session {id}: {e}");
                            toast
                                .write()
                                .push_back(format!("⚠️ Failed to delete session: {e}"));
                            if let Some(session) = snapshot {
                                sessions_sig.write().push(session);
                            }
                        }
                    },
                    NativeResult::PutExercise { id, result } => match result {
                        Ok(()) => {
                            log::info!("Successfully saved exercise {id}");
                        }
                        Err(e) => {
                            log::error!("Failed to save exercise {id}: {e}");
                            toast
                                .write()
                                .push_back(format!("⚠️ Failed to save exercise: {e}"));
                        }
                    },
                }
            }
            // Put it back if we ever exit the loop (though we shouldn't)
            let mut lock = get_result_channel().1.lock().unwrap();
            *lock = Some(rx);
        }
    });
}

async fn drain() {
    let tx = &get_result_channel().0;
    loop {
        let op = {
            let mut q = queue().lock().unwrap();
            if let Some(op) = q.pending.pop_front() {
                op
            } else {
                q.draining = false;
                break;
            }
        };

        match op {
            NativeOp::PutSession {
                session: s,
                previous,
            } => {
                let id = s.id.clone();
                let res = tokio::task::spawn_blocking(move || {
                    super::storage::native_storage::put_item(
                        super::storage::native_storage::STORE_SESSIONS,
                        &s.id,
                        &s,
                    )
                })
                .await;

                let result = match res {
                    Ok(Ok(())) => Ok(()),
                    Ok(Err(e)) => Err(e.to_string()),
                    Err(e) => Err(format!("Task panicked: {e}")),
                };
                let _ = tx.send(NativeResult::PutSession {
                    id,
                    result,
                    previous,
                });
            }
            NativeOp::DeleteSession { id, snapshot } => {
                let id2 = id.clone();
                let res = tokio::task::spawn_blocking(move || {
                    super::storage::native_storage::delete_item(
                        super::storage::native_storage::STORE_SESSIONS,
                        &id,
                    )
                })
                .await;
                let result = match res {
                    Ok(Ok(())) => Ok(()),
                    Ok(Err(e)) => Err(e.to_string()),
                    Err(e) => Err(format!("Task panicked: {e}")),
                };
                let _ = tx.send(NativeResult::DeleteSession {
                    id: id2,
                    result,
                    snapshot,
                });
            }
            NativeOp::PutExercise(ex) => {
                let id = ex.id.clone();
                let res = tokio::task::spawn_blocking(move || {
                    super::storage::native_storage::put_item(
                        super::storage::native_storage::STORE_CUSTOM_EXERCISES,
                        &ex.id,
                        &ex,
                    )
                })
                .await;
                let result = match res {
                    Ok(Ok(())) => Ok(()),
                    Ok(Err(e)) => Err(e.to_string()),
                    Err(e) => Err(format!("Task panicked: {e}")),
                };
                let _ = tx.send(NativeResult::PutExercise { id, result });
            }
        }
        tokio::task::yield_now().await;
    }
}
