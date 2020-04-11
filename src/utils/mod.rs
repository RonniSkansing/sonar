pub mod file {
    use std::fs::File;
    use std::fs::OpenOptions;
    use std::path::Path;

    pub trait Append {
        fn create_append<P: AsRef<Path>>(path: P) -> std::io::Result<File> {
            OpenOptions::new()
                .truncate(false)
                .append(true)
                .create(true)
                .open(path.as_ref())
        }
    }
    impl Append for std::fs::File {}
}

pub mod prometheus {
    pub fn normalize_name(s: String) -> String {
        s.replace('-', "_").replace('.', "_")
    }

    pub fn counter_success_name(s: String) -> String {
        normalize_name(s + "_success")
    }

    pub fn timer_name(name: String) -> String {
        normalize_name(name + "_time_ms")
    }
}

pub mod tokio_shutdown {
    use futures::future::{AbortHandle, AbortRegistration, Abortable};
    use mpsc::error::TryRecvError;
    use std::{future::Future, marker::Send, result::Result};
    use tokio::sync::mpsc;

    // Returns the AbortController for aborting, AbortRegistration for wrapping a task that should be able to forcefully stop
    // and the watchgroup itself for listening for graceful shutdown request and signal back that their done
    pub fn new() -> (AbortController, AbortRegistration, Syncronizer) {
        let (shutdown_tx, shutdown_rx) = mpsc::channel::<ShutdownEvent>(1);
        let (shutdown_is_done_tx, shutdown_is_done_rx) = mpsc::channel::<ShutdownEvent>(1);
        let (aborter, abort_reg) = AbortHandle::new_pair();

        (
            AbortController::new(shutdown_tx, shutdown_is_done_rx, aborter),
            abort_reg,
            Syncronizer::new(shutdown_rx, shutdown_is_done_tx),
        )
    }

    #[derive(Debug, PartialEq)]
    pub enum ShutdownEvent {
        Requested,
        Done,
    }

    type ShutdownSender = mpsc::Sender<ShutdownEvent>;
    type ShutdownReceiver = mpsc::Receiver<ShutdownEvent>;

    #[derive(Debug)]
    // Controller is stop a remote task
    // Forceful shutdown will happend at once. A graceful shutdown sends a signal to the task and the task sends back a signal when it's done
    pub struct AbortController {
        stop_signal_tx: ShutdownSender,
        is_done_signal_rx: ShutdownReceiver,
        forceful_stop_handle: AbortHandle,
    }

    impl AbortController {
        pub fn new(
            tx: ShutdownSender,
            is_done_tx: ShutdownReceiver,
            forceful_stop_handle: AbortHandle,
        ) -> Self {
            AbortController {
                stop_signal_tx: tx,
                is_done_signal_rx: is_done_tx,
                forceful_stop_handle,
            }
        }

        pub fn _shutdown_forcefully(self) {
            self.forceful_stop_handle.abort();
        }

        pub async fn shutdown_gracefully(mut self) -> Result<(), impl std::error::Error> {
            // send signal to stop
            match self.stop_signal_tx.send(ShutdownEvent::Requested).await {
                Ok(_) => {
                    // if OK then the signal is sent and we cant start listening for
                    // no matter what the answer is, the shutdown is complete
                    let _ = self.is_done_signal_rx.recv().await;
                    Ok(())
                }
                Err(err) => Err(err),
            }
        }
    }

    // The mediator is used inside the task to listen for when to stop and to signal back that it has stopped
    pub struct Syncronizer {
        shutdown_rx: ShutdownReceiver,
        shutdown_is_done_tx: ShutdownSender,
    }

    impl Syncronizer {
        pub fn new(shutdown_rx: ShutdownReceiver, shutdown_is_done_tx: ShutdownSender) -> Self {
            Self {
                shutdown_rx,
                shutdown_is_done_tx,
            }
        }

        pub fn should_stop(&mut self) -> bool {
            match self.shutdown_rx.try_recv() {
                Ok(_) => true,
                Err(err) => match err {
                    TryRecvError::Empty => false,
                    TryRecvError::Closed => true,
                },
            }
        }

        pub async fn done(mut self) {
            let _ = self.shutdown_is_done_tx.try_send(ShutdownEvent::Done);
        }
    }

    pub fn _to_abortable<T>(task: T) -> (Abortable<T>, AbortHandle)
    where
        T: 'static + Future + Send,
    {
        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        let future = Abortable::new(task, abort_registration);

        (future, abort_handle)
    }

    pub fn to_abortable_with_registration<T>(
        registration: AbortRegistration,
        task: T,
    ) -> Abortable<T>
    where
        T: 'static + std::future::Future + std::marker::Send,
    {
        Abortable::new(task, registration)
    }
}
