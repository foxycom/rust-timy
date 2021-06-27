use std::thread;
use std::time::Duration;
use std::sync::{mpsc, Mutex, Arc};
use timy::Timer;

fn main() {

    let timer = Timer::new();
    timer.start(Duration::from_millis(3000), || {
        println!("Done");
    });
}
