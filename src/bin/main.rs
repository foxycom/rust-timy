use std::thread;
use std::time::Duration;
use std::sync::{mpsc, Mutex, Arc};
use timy::Timer;
use std::io::{BufReader, Error};
use std::fs::{File, read};
use rodio::{Decoder, OutputStream, source::Source, Sink};
use clap::{Arg, App};
use druid::{WindowDesc, Widget, LocalizedString, AppLauncher, Env, WidgetExt, Data, Lens, Color, UnitPoint, EventCtx, LifeCycle, PaintCtx, LifeCycleCtx, BoxConstraints, Size, LayoutCtx, Event, UpdateCtx};
use druid::widget::{Label, TextBox, Flex, Align, Button};
use std::thread::sleep;
use std::alloc::System;
use std::convert::TryFrom;
use std::env::VarError;
use std::path::Path;

static SOUND_VAR_NAME: &'static str = "TIMY_SOUND_DIR";

struct CliError {
    message: String,
}

struct MusicError {
    message: String,
}

#[derive(Clone, Data, Lens)]
struct TimyState {
    input: String,
}

fn get_music_file(music: &str) -> Result<File, MusicError> {
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

macro_rules! music {
    ($path:literal, $volume:expr) => {
        {
            println!("Playing music");
            match get_music_file($path) {
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

fn build_root_widget() -> impl Widget<TimyState> {
    let label = Label::new(|data: &TimyState, _env: &Env| format!("{} seconds", data.input));
    // a textbox that modifies `name`.
    let textbox = TextBox::new()
        .with_placeholder("Enter timer in seconds")
        .fix_width(200.0)
        .lens(TimyState::input);

    // arrange the two widgets vertically, with some padding
    let layout = Flex::column()
        .with_child(label)
        .with_spacer(20.0)
        .with_child(textbox);

    // center the two widgets in the available space
    Align::centered(layout)
}

fn main() {
    ctrlc::set_handler(move || {
        println!("Canceled timer");
        std::process::exit(0);
    });

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
        .arg(Arg::new("volume")
            .short('v')
            .long("volume")
            .value_name("VOLUME")
            .takes_value(true))
        .get_matches();

    let seconds: usize = matches.value_of_t("seconds").unwrap_or(0);
    let minutes: usize = matches.value_of_t("minutes").unwrap_or(0);
    let volume: f32 = matches.value_of_t("volume").unwrap_or(1.0);

    println!("Starting timer with {} minutes and {} seconds.", minutes, seconds);

    let timer = Timer::new();

    let seconds = (minutes * 60 + seconds) as u64;
    timer.start(Duration::from_secs(seconds), move || {
        music!("space.mp3", volume);
        println!("Done");
    });
}
