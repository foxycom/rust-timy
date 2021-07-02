mod formatters;

use std::time::Duration;
use std::thread::{JoinHandle, Thread};
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::fs::File;
use std::path::Path;

type Callback = Box<dyn FnOnce() + Send + 'static>;
type RecurrentCallback = Box<dyn FnMut() + Send +'static>;

const SOUND_VAR_NAME: &str = "TIMY_SOUND_DIR";

pub struct MusicError {
    pub message: String,
}

struct TimeSettings {
    duration: Duration,
    tick_duration: Duration,
    end_callback: Callback,
    tick_callback: Option<RecurrentCallback>,
}

enum TimeMessage {
    Time(TimeSettings),
    Stop,
}

pub struct Timer {
    pub tick: Duration,
    pub tick_callback: Option<RecurrentCallback>,
    worker: Worker,
    sender: mpsc::Sender<TimeMessage>,
}

impl Timer {
    pub fn new() -> Timer {
        let (sender, receiver) = mpsc::channel();
        Timer {
            tick: Duration::from_millis(1000),
            tick_callback: None,
            worker: Worker::new(receiver),
            sender
        }
    }

    pub fn start<F>(&mut self, duration: Duration, callback: F)
        where F: FnOnce() + Send + 'static {
        let callback = Box::new(callback);

        let tick_callback = match self.tick_callback.take() {
            None => None,
            Some(callback) => Some(callback)
        };

        let settings = TimeSettings {
            duration,
            tick_callback,
            tick_duration: self.tick,
            end_callback: callback,
        };
        self.sender.send(TimeMessage::Time(settings)).unwrap();
    }

    pub fn wait(&mut self) {
        if let Some(thread) = self.worker.thread.take() {
            thread.join().unwrap();
        }
    }

    pub fn stop(&self) {
        self.sender.send(TimeMessage::Stop).unwrap();
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
                TimeMessage::Time(mut settings) => {
                    duration = settings.duration;
                    loop {
                        let delay = duration.min(Duration::from_millis(1000));
                        thread::sleep(delay);
                        duration = duration.saturating_sub(Duration::from_millis(1000));

                        if let Some(ref mut tick_callback) = settings.tick_callback {
                            tick_callback();
                        }

                        let message = receiver.try_recv();
                        match message {
                            Ok(message) => {
                                match message {
                                    TimeMessage::Time(settings) => duration = settings.duration,
                                    TimeMessage::Stop => break
                                }
                            }
                            Err(err) => {
                                match err {
                                    TryRecvError::Empty => {
                                        if let Duration::ZERO = duration {
                                            let end_callback = settings.end_callback;
                                            end_callback();
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

pub fn get_music_file(music: &str) -> Result<File, MusicError> {
    match std::env::var(SOUND_VAR_NAME) {
        Ok(path) => {
            let dir = Path::new(&path);
            let file = Path::new(music);
            let joined_path = dir.join(file);
            match File::open(joined_path.as_path()) {
                Ok(file) => {
                    Ok(file)
                }
                Err(_) => {
                    Err(MusicError { message: format!("Could not find the music file in {}", path.as_str()) })
                }
            }
        }
        Err(_) => {
            Err(MusicError { message: format!("Environment variable {} not set", SOUND_VAR_NAME) })
        }
    }
}

#[macro_export]
macro_rules! music {
    ($path:literal, $volume:expr) => {
        {
            match timy::get_music_file($path) {
                Ok(file) => {
                    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

                    let sink = Sink::try_new(&stream_handle).unwrap();
                    sink.set_volume($volume);

                    let decoder = Decoder::new(file).unwrap()
                        .take_duration(Duration::from_secs(5));
                    sink.append(decoder);

                    sink.sleep_until_end();
                }
                Err(err) => {
                    println!("Sound error: {}", err.message);
                }
            }
        }
    }
}