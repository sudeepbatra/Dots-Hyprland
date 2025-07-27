// Take a look at the license at the top of the repository in the LICENSE file.

use futures_channel::oneshot;
use futures_core::task::{Context, Poll};
use std::future::Future;
use std::pin::{self, Pin};

use crate::prelude::*;
use crate::Cancellable;

pub struct GioFuture<F, O, T, E> {
    obj: O,
    schedule_operation: Option<F>,
    cancellable: Option<Cancellable>,
    receiver: Option<oneshot::Receiver<Result<T, E>>>,
}

pub struct GioFutureResult<T, E> {
    sender: ThreadGuard<oneshot::Sender<Result<T, E>>>,
}

unsafe impl<T, E> Send for GioFutureResult<T, E> {}

impl<T, E> GioFutureResult<T, E> {
    pub fn resolve(self, res: Result<T, E>) {
        let _ = self.sender.into_inner().send(res);
    }
}

impl<F, O, T: 'static, E: 'static> GioFuture<F, O, T, E>
where
    O: Clone + 'static,
    F: FnOnce(&O, &Cancellable, GioFutureResult<T, E>) + 'static,
{
    pub fn new(obj: &O, schedule_operation: F) -> GioFuture<F, O, T, E> {
        GioFuture {
            obj: obj.clone(),
            schedule_operation: Some(schedule_operation),
            cancellable: Some(Cancellable::new()),
            receiver: None,
        }
    }
}

impl<F, O, T, E> Future for GioFuture<F, O, T, E>
where
    O: Clone + 'static,
    F: FnOnce(&O, &Cancellable, GioFutureResult<T, E>) + 'static,
{
    type Output = Result<T, E>;

    fn poll(mut self: pin::Pin<&mut Self>, ctx: &mut Context) -> Poll<Result<T, E>> {
        let GioFuture {
            ref obj,
            ref mut schedule_operation,
            ref mut cancellable,
            ref mut receiver,
            ..
        } = *self;

        if let Some(schedule_operation) = schedule_operation.take() {
            let main_context = glib::MainContext::ref_thread_default();
            assert!(
                main_context.is_owner(),
                "Spawning futures only allowed if the thread is owning the MainContext"
            );

            // Channel for sending back the GIO async operation
            // result to our future here.
            //
            // In theory, we could directly continue polling the
            // corresponding task from the GIO async operation
            // callback, however this would break at the very
            // least the g_main_current_source() API.
            let (send, recv) = oneshot::channel();

            schedule_operation(
                obj,
                cancellable.as_ref().unwrap(),
                GioFutureResult {
                    sender: ThreadGuard::new(send),
                },
            );

            *receiver = Some(recv);
        }

        // At this point we must have a receiver
        let res = {
            let receiver = receiver.as_mut().unwrap();
            Pin::new(receiver).poll(ctx)
        };

        match res {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(_)) => panic!("Async operation sender was unexpectedly closed"),
            Poll::Ready(Ok(v)) => {
                // Get rid of the reference to the cancellable and receiver
                let _ = cancellable.take();
                let _ = receiver.take();
                Poll::Ready(v)
            }
        }
    }
}

impl<F, O, T, E> Drop for GioFuture<F, O, T, E> {
    fn drop(&mut self) {
        if let Some(cancellable) = self.cancellable.take() {
            cancellable.cancel();
        }
        let _ = self.receiver.take();
    }
}

impl<F, O, T, E> Unpin for GioFuture<F, O, T, E> {}

// Actual thread IDs can be reused by the OS once the old thread finished.
// This works around it by using our own counter for threads.
//
// Taken from the fragile crate
use std::sync::atomic::{AtomicUsize, Ordering};
fn next_thread_id() -> usize {
    static mut COUNTER: AtomicUsize = AtomicUsize::new(0);
    unsafe { COUNTER.fetch_add(1, Ordering::SeqCst) }
}

#[doc(alias = "get_thread_id")]
fn thread_id() -> usize {
    thread_local!(static THREAD_ID: usize = next_thread_id());
    THREAD_ID.with(|&x| x)
}

// Taken from glib-rs, but we don't want this to be public API
struct ThreadGuard<T> {
    thread_id: usize,
    value: Option<T>,
}

impl<T> ThreadGuard<T> {
    fn new(value: T) -> Self {
        Self {
            thread_id: thread_id(),
            value: Some(value),
        }
    }

    fn into_inner(mut self) -> T {
        if self.thread_id != thread_id() {
            panic!("Value accessed from different thread than where it was created");
        }

        self.value.take().expect("into_inner() called twice")
    }
}

impl<T> Drop for ThreadGuard<T> {
    fn drop(&mut self) {
        if self.thread_id != thread_id() {
            panic!("Value dropped on a different thread than where it was created");
        }
    }
}

unsafe impl<T> Send for ThreadGuard<T> {}
