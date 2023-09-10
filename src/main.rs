use async_std::task::spawn;
use std::future::Future;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};
use scoped_tls::scoped_thread_local;
use signal::Signal;
use task::Task;
mod signal;
mod task;

scoped_thread_local!(static SIGNAL:Arc<Signal>);
scoped_thread_local!(static RUNNABLE:Mutex<VecDeque<Arc<Task>>>);

async fn demo() {
    let (tx, rx) = async_channel::bounded::<()>(1);
    spawn(demo2(tx));
    println!("hello world!");
    let _ = rx.recv().await;
}

async fn demo2(tx: async_channel::Sender<()>) {
    println!("hello world2!");
    let _ = tx.send(()).await;
}

fn block_on<F: Future>(future: F) -> F::Output {
    let mut fut = std::pin::pin!(future);
    let signal:Arc<Signal> = Arc::new(Signal::new());
    let waker: Waker = Waker::from(signal.clone());

    let mut cx = Context::from_waker(&waker);
    let runnable: Mutex<VecDeque<Arc<Task>>> = Mutex::new(VecDeque::with_capacity(1024));
    SIGNAL.set(&signal, || {
        RUNNABLE.set(&runnable, || loop {
            if let Poll::Ready(output) = fut.as_mut().poll(&mut cx) {
                return output;
            }
            while let Some(task) = runnable.lock().unwrap().pop_back() {
                let waker = Waker::from(task.clone());
                let mut cx = Context::from_waker(&waker);
                let _ = task.future.borrow_mut().as_mut().poll(&mut cx);
            }
            signal.wait();
        })
    })
}

fn main() {
    block_on(demo());
}