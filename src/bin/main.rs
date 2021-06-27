use std::thread;
use std::time::Duration;
use std::sync::{mpsc, Mutex, Arc};
use timy::Timer;
use rodio::{OutputStream, Decoder, Source};
use std::io::BufReader;
use std::fs::{File, read};
use clap::{Arg, App};

struct CliError {
    message: String,
}

fn main() {
    let matches = App::new("Timy")
        .version("0.1.0")
        .arg(Arg::new("minutes")
            .short('m')
            .long("minutes")
            .value_name("MINUTES")
            .takes_value(true))
        .arg(Arg::new("seconds")
            .short('s')
            .long("seconds")
            .value_name("SECONDS")
            .default_value("0")
            .takes_value(true))
        .get_matches();

    let seconds: usize = matches.value_of_t("seconds").unwrap_or(0);
    let minutes: usize = matches.value_of_t("minutes").unwrap_or(0);

    let timer = Timer::new();
    /*let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let file = BufReader::new(File::open("music/space.mp3").unwrap());
    let source = Decoder::new(file).unwrap();
    let samples = source.convert_samples();*/

    let seconds = (minutes * 60 + seconds) as u64;
    timer.start(Duration::from_secs(seconds), move || {
        /*stream_handle.play_raw(samples);*/
        /*thread::sleep(Duration::from_secs(3));*/
        println!("Done");
    });
}


