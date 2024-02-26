use core::{future::Future, pin::Pin, task::{Context, Poll}};
use alloc::boxed::Box;

pub mod simple_executor;
pub mod keyboard;

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    // NOTE: `'static` not just means static lifetime, also means the `future` own its ownership.
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Self { future: Box::pin(future) }
    }

    fn poll(&mut self, ctx: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(ctx)
    }
}
