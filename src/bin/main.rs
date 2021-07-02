use std::thread;
use std::time::Duration;
use std::sync::{mpsc, Mutex, Arc};
use timy::{Timer, music};
use std::io::{BufReader, Error, Stdout};
use rodio::{Decoder, OutputStream, source::Source, Sink};
use clap::{Arg, App};
use druid::{WindowDesc, Widget, LocalizedString, AppLauncher, Env, WidgetExt, Data, Lens, Color, UnitPoint, EventCtx, LifeCycle, PaintCtx, LifeCycleCtx, BoxConstraints, Size, LayoutCtx, Event, UpdateCtx, TextAlignment};
use druid::widget::{Label, TextBox, Flex, Align, Button};
use std::thread::sleep;
use std::alloc::System;
use std::convert::TryFrom;
use std::env::VarError;
use std::path::Path;

use notify_rust::{Notification, Hint};
use pbr::{Units, ProgressBar};
use std::borrow::{Borrow, BorrowMut};


struct CliError {
    message: String,
}

#[derive(Clone, Data, Lens)]
struct TimyState {
    minutes: String,
    seconds: String,
}

fn build_root_widget() -> impl Widget<TimyState> {
    let mut col = Flex::column().with_child(
        Flex::row()
            .with_flex_spacer(1.0)
            .with_child(
                TextBox::new()
                    .with_text_size(38.0)
                    .with_text_alignment(TextAlignment::Center)
                    .fix_height(50.0)
                    .lens(TimyState::minutes),
            )
            .with_child(
                Label::new(":")
            )
            .with_child(
                TextBox::new()
                    .with_text_size(38.0)
                    .with_text_alignment(TextAlignment::Center)
                    .fix_height(50.0)
                    .lens(TimyState::seconds)
            ).with_flex_spacer(1.0)
            .fix_height(200.0)
            .background(Color::rgb8(0, 0x77, 0x88)))
        .with_child(
            Flex::row()
                .with_child(
                    Button::new("Start")
                        .fix_width(100.0)
                        .padding(10.)
                )
        );
    col
}

fn push_notification() {
    Notification::new()
        .summary("Timy")
        .body("The timer has expired.")
        .icon("finder")
        .appname("Timy")
        .timeout(0) // this however is
        .show();
}

/*fn main() {
    let window = WindowDesc::new(build_root_widget).title("Very flexible");

    let state = TimyState { minutes: "0".to_string(), seconds: "0".to_string() };
    AppLauncher::with_window(window)
        .launch(state)
        .expect("launch failed");
}*/

fn create_progress_bar(total: u64) -> ProgressBar<Stdout> {
    let mut pb = pbr::ProgressBar::new(total);
    pb.format("╢▌▌░╟");
    pb.show_counter = false;
    pb.show_speed = false;
    pb.show_percent = false;
    pb
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

    let seconds = (minutes * 60 + seconds) as u64;

    let mut timer = Timer::new();
    let mut pb = create_progress_bar(seconds);

    timer.tick_callback = Some(Box::new(move || {
        pb.inc();
    }));

    timer.start(Duration::from_secs(seconds), move || {
        push_notification();
        music!("space.mp3", volume);
        println!("\nDone");
    });

    timer.wait();
}