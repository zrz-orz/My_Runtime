use std::{cell::RefCell, sync::Arc, task::Wake};

use futures::future::BoxFuture;

use crate::{signal::Signal, RUNNABLE};

pub struct Task {
    pub future: RefCell<BoxFuture<'static, ()>>,
    signal: Arc<Signal>,
}

unsafe impl Send for Task {}

unsafe impl Sync for Task {}

impl Wake for Task {
    fn wake(self: Arc<Self>) {
        RUNNABLE.with(|runnable| runnable.lock().unwrap().push_back(self.clone()));
        self.signal.notify();
    }
}