use std::time::Duration;
use std::thread::{JoinHandle, Thread};
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;

type Callback = Box<dyn FnMut() + Send + 'static>;

enum TimeMessage {
    Time(Duration, Callback),
    Stop,
}

pub struct Timer {
    remaining_duration: Option<Duration>,
    worker: Worker,
    sender: mpsc::Sender<TimeMessage>,
}

impl Timer {
    pub fn new() -> Timer {
        let (sender, receiver) = mpsc::channel();
        Timer { remaining_duration: None, worker: Worker::new(receiver), sender }
    }

    pub fn start<F>(&self, duration: Duration, callback: F)
        where F: FnMut() + Send + 'static {
        let callback = Box::new(callback);
        self.sender.send(TimeMessage::Time(duration, callback)).unwrap();
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        if let Some(thread) = self.worker.thread.take() {
            thread.join().unwrap();
        }
    }
}

struct Worker {
    thread: Option<JoinHandle<()>>,
}


impl Worker {
    fn new(receiver: mpsc::Receiver<TimeMessage>) -> Worker {
        let thread = thread::spawn(move || {
            let message = receiver.recv().unwrap();
            let mut duration;

            match message {
                TimeMessage::Time(new_duration, mut callback) => {
                    duration = new_duration;
                    loop {
                        let delay = duration.min(Duration::from_millis(1000));
                        thread::sleep(delay);
                        duration = duration.saturating_sub(Duration::from_millis(1000));
                        let message = receiver.try_recv();
                        match message {
                            Ok(message) => {
                                match message {
                                    TimeMessage::Time(new_duration, callback) => duration = new_duration,
                                    TimeMessage::Stop => break
                                }
                            }
                            Err(err) => {
                                match err {
                                    TryRecvError::Empty => {
                                        if let Duration::ZERO = duration {
                                            callback();
                                            break;
                                        }
                                    }
                                    TryRecvError::Disconnected => {
                                        // TODO do something
                                    }
                                }
                            }
                        }
                    }
                }
                TimeMessage::Stop => {}
            }
        });
        Worker { thread: Some(thread) }
    }

    fn run() {}
}

