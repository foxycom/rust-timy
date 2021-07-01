use std::time::Duration;
use std::thread::{JoinHandle, Thread};
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::fs::File;
use std::path::Path;

type Callback = Box<dyn FnOnce() + Send + 'static>;

static SOUND_VAR_NAME: &'static str = "TIMY_SOUND_DIR";

pub struct MusicError {
    pub message: String,
}

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
        where F: FnOnce() + Send + 'static {
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
            println!("Playing music");

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