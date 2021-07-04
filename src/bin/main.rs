use std::thread;
use std::time::Duration;
use std::sync::{mpsc, Mutex, Arc};
use timy::{Timer, music};
use std::io::{BufReader, Error, Stdout};
use rodio::{Decoder, OutputStream, source::Source, Sink};
use clap::{Arg, App};
use std::thread::sleep;
use std::alloc::System;
use std::convert::TryFrom;
use std::env::VarError;
use std::path::Path;
use std::borrow::{Borrow, BorrowMut};
use std::option::Option::Some;
use std::sync::mpsc::Sender;
use std::cell::RefCell;
use notify_rust::{Notification, Hint};

#[cfg(feature = "CLI")]
use pbr::{Units, ProgressBar};

#[cfg(feature = "GUI")]
use druid::{WindowDesc, Widget, LocalizedString, AppLauncher, Env, WidgetExt, Data, Lens, Color, UnitPoint, EventCtx, LifeCycle, PaintCtx, LifeCycleCtx, BoxConstraints, Size, LayoutCtx, Event, UpdateCtx, TextAlignment, Selector, Target, AppDelegate, DelegateCtx, Command, Handled, ExtEventSink};
#[cfg(feature = "GUI")]
use druid::widget::{Label, TextBox, Flex, Align, Button};

#[cfg(feature = "GUI")]
const TIMY_FINISH: Selector<()> = Selector::new("timy_finish");
#[cfg(feature = "GUI")]
const TIMY_TICK: Selector<Duration> = Selector::new("timy_tick");
#[cfg(feature = "GUI")]
const TIMY_STOP: Selector<()> = Selector::new("timy_stop");


struct CliError {
    message: String,
}

#[cfg(feature = "GUI")]
enum EventMessage {
    Stop
}

#[cfg(feature = "GUI")]
#[derive(Clone, Default, Data, Lens)]
struct TimyState {
    minutes: String,
    seconds: String,
    running: bool,

    #[data(ignore)]
    sender: Option<Sender<EventMessage>>,
}

#[cfg(feature = "GUI")]
impl TimyState {
    fn new(minutes: u8, seconds: u8) -> TimyState {
        TimyState {
            minutes: minutes.to_string(),
            seconds: seconds.to_string(),
            running: false,
            sender: None,
        }
    }
}

#[cfg(feature = "GUI")]
struct Delegate;

#[cfg(feature = "GUI")]
impl AppDelegate<TimyState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        state: &mut TimyState,
        _env: &Env) -> Handled {

        if let Some(_) = cmd.get(TIMY_FINISH) {
            state.running = false;
            state.sender = None;
            push_notification();
            Handled::Yes
        } else if let Some(remaining_duration) = cmd.get(TIMY_TICK) {
            let duration_in_secs = remaining_duration.as_secs();
            let subsec_nanos = remaining_duration.subsec_nanos();

            state.seconds = (duration_in_secs % 60 + if subsec_nanos > 500000000 { 1 } else { 0 }).to_string();
            state.minutes = (duration_in_secs / 60).to_string();
            Handled::Yes
        } else if let Some(_) = cmd.get(TIMY_STOP) {
            Handled::Yes
        } else {
            println!("Not handled");
            Handled::No
        }
    }
}

#[cfg(feature = "GUI")]
fn build_root_widget() -> impl Widget<TimyState> {
    let mut col = Flex::column().with_child(
        Flex::row()
            .with_flex_spacer(1.0)
            .with_child(
                TextBox::new()
                    .with_text_size(38.0)
                    .with_text_alignment(TextAlignment::Center)
                    .fix_height(50.0)
                    .lens(TimyState::minutes)
                    .disabled_if(|state, _| state.running),
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
                    .disabled_if(|state, _| state.running)
            ).with_flex_spacer(1.0)
            .fix_height(200.0)
            .background(Color::rgb8(0, 0x77, 0x88)))
        .with_child(
            Flex::row()
                .with_child(
                    Button::dynamic(|state: &TimyState, _: &Env| {
                        match state.running {
                            true => "Stop".into(),
                            false => "Start".into()
                        }
                    })
                        .fix_width(100.0)
                        .padding(10.)
                        .on_click(|ctx, state: &mut TimyState, _| {
                            if state.running {
                                let sender = state.sender.as_ref();
                                if let Some(sender) = sender {
                                    sender.send(EventMessage::Stop);
                                    state.running = false;
                                }
                            } else {
                                state.running = true;
                                let seconds = state.seconds.parse::<u64>().unwrap();
                                let minutes = state.minutes.parse::<u64>().unwrap();
                                let seconds = seconds + minutes * 60;
                                let sink = ctx.get_external_handle();
                                let sender = wait(sink, seconds);
                                state.sender = Some(sender);
                            }
                        })
                )
        );
    col
}

#[cfg(feature = "GUI")]
fn wait(sink: ExtEventSink, seconds: u64) -> Sender<EventMessage> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let sink = Arc::new(sink);
        let tick_sink = sink.clone();
        let mut timer = Timer::new();

        timer.tick_callback = Some(Box::new(move |remaining_duration| {
            tick_sink.submit_command(TIMY_TICK, remaining_duration, Target::Auto).expect("");
        }));

        timer.start(Duration::from_secs(seconds), move || {
            // TODO play sound
            sink.submit_command(TIMY_FINISH, (), Target::Auto).expect("");
        });

        for msg in receiver.recv() {
            match msg {
                EventMessage::Stop => timer.stop()
            }
        }
    });

    sender
}

fn push_notification() {
    Notification::new()
        .summary("Timy")
        .body("The timer has expired.")
        .icon("finder")
        .appname("Timy")
        .timeout(0)
        .show();
}

#[cfg(feature = "CLI")]
fn create_progress_bar(total: u64) -> ProgressBar<Stdout> {
    let mut pb = pbr::ProgressBar::new(total);
    pb.format("╢▌▌░╟");
    pb.show_counter = false;
    pb.show_speed = false;
    pb.show_percent = false;
    pb
}

fn main() {
    #[cfg(feature = "GUI")]
        {
            let window = WindowDesc::new(build_root_widget()).title("Timy");
            let launcher = AppLauncher::with_window(window);


            let state = TimyState::new(0, 0);
            launcher
                .delegate(Delegate {})
                .launch(state)
                .expect("launch failed");
        }

    #[cfg(feature = "CLI")]
        {
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

            timer.tick_callback = Some(Box::new(move |remaining_duration| {
                pb.inc();
            }));

            timer.start(Duration::from_secs(seconds), move || {
                push_notification();
                music!("space.mp3", volume);
                println!("\nDone");
            });

            timer.wait();
        }
}