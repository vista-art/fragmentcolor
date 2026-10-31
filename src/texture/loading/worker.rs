//! The worker thread that decodes and uploads textures,
//! so the caller's thread stays free. The web runs each job inline.

#[cfg(not(wasm))]
mod imp {
    use std::sync::OnceLock;
    use std::sync::mpsc::{Sender, channel};
    use std::thread;

    type Job = Box<dyn FnOnce() + Send + 'static>;

    static QUEUE: OnceLock<Sender<Job>> = OnceLock::new();

    fn queue() -> &'static Sender<Job> {
        QUEUE.get_or_init(|| {
            let (tx, rx) = channel::<Job>();
            thread::Builder::new()
                .name("fragmentcolor-bg".into())
                .spawn(move || {
                    while let Ok(job) = rx.recv() {
                        job();
                    }
                })
                .expect("SAFETY: spawning a single named worker thread on first texture create. Failure here means the OS denied thread creation, which is unrecoverable for any GPU work.");
            tx
        })
    }

    pub async fn run<F, T>(job: F) -> T
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = futures::channel::oneshot::channel::<T>();
        queue()
            .send(Box::new(move || {
                let _ = tx.send(job());
            }))
            .expect("SAFETY: process-wide worker is initialized on first call and never shut down; send only fails if it panicked, which is itself a bug we want to surface loudly.");
        rx.await
            .expect("SAFETY: the worker only drops the oneshot if the closure panicked; surface that panic instead of silently returning a placeholder.")
    }
}

#[cfg(wasm)]
mod imp {
    pub async fn run<F, T>(job: F) -> T
    where
        F: FnOnce() -> T,
    {
        job()
    }
}

pub use imp::run;
