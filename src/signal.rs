use std::sync::{Arc, Mutex, Condvar};
use std::task::Wake;

pub struct Signal {
    state: Mutex<State>,
    condvar: Condvar,
}

enum State {
    Empty,
    Waiting,
    Notified,
}

impl Signal {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(State::Empty),
            condvar: Condvar::new(),
        }
    }

    pub fn wait(&self) {
        let mut state = self.state.lock().unwrap();
        match *state {
            State::Notified => *state = State::Empty,
            State::Waiting => {
                panic!("multiple wait");
            }
            State::Empty => {
                *state = State::Waiting;
                while let State::Waiting = *state {
                    state = self.condvar.wait(state).unwrap();
                }
            }
        }
    }

    pub fn notify(&self) {
        let mut state = self.state.lock().unwrap();
        match *state {
            State::Notified => {}
            State::Empty => *state = State::Notified,
            State::Waiting => {
                *state = State::Empty;
                self.condvar.notify_one();
            }
        }
    }
}

impl Wake for Signal {
    fn wake(self: std::sync::Arc<Self>) {
        self.notify();
    }
}
