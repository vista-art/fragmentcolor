//! Internal background executor for CPU-only prep work.
//!
//! Texture creation, image decoding, and mipmap generation are pure CPU work
//! that doesn't touch the `Surface`. We offload them to a worker thread so the
//! caller (typically the GPU/event-loop thread) stays responsive. Public API
//! does not change — callers still `await` the result.
//!
//! - **Native:** a single process-wide worker thread receives `FnOnce` jobs
//!   over an mpsc channel and replies via a `futures::channel::oneshot`. The
//!   `Arc<RenderContext>` captured by the closure is `Send + Sync`, and
//!   `wgpu::Queue::write_texture` only needs `&self`, so the worker can do the
//!   GPU writes without bouncing back to the main thread.
//! - **Wasm:** `wgpu` types are `!Send` (they hold JS objects bound to the
//!   page's main thread). Real off-thread work would need a Web Worker with
//!   its own GPU adapter, which is out of scope here. The wasm path runs the
//!   job inline and returns immediately. Behavior matches the pre-refactor
//!   code on the web — no regression — and the public API stays identical.
//!
//! Single-worker on native is intentional for v0.11.0: it frees the main
//! thread (the primary ask) without adding a thread pool we don't yet have
//! evidence we need. If batch throughput becomes a bottleneck, scale to N
//! workers without changing the public API.

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
