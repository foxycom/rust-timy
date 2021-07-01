use std::thread;
use std::time::Duration;
use std::sync::{mpsc, Mutex, Arc};
use timy::Timer;
use std::io::BufReader;
use std::fs::{File, read};
use rodio::{Decoder, OutputStream, source::Source, Sink};
use clap::{Arg, App};
use druid::{WindowDesc, Widget, LocalizedString, AppLauncher, Env, WidgetExt, Data, Lens, Color, UnitPoint, EventCtx, LifeCycle, PaintCtx, LifeCycleCtx, BoxConstraints, Size, LayoutCtx, Event, UpdateCtx};
use druid::widget::{Label, TextBox, Flex, Align, Button};
use std::thread::sleep;

struct CliError {
    message: String,
}

#[derive(Clone, Data, Lens)]
struct TimyState {
    input: String,
}

/*fn main() {
    let main_window = WindowDesc::new(build_root_widget)
        .title(LocalizedString::new("Hello world"))
        .window_size((600.00, 400.00));

    let initial_state = TimyState {
        input: "".into()
    };

    AppLauncher::with_window(main_window)
        .launch(initial_state)
        .expect("Hello");
}*/

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

    println!("Starting timer with {} minutes and {} seconds.", minutes, seconds);

    let timer = Timer::new();



    let seconds = (minutes * 60 + seconds) as u64;
    timer.start(Duration::from_secs(seconds), move || {
        println!("Starting sound");
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let file = File::open("./music/space.mp3").unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        let decoder = Decoder::new(file).unwrap()
            .take_duration(Duration::from_secs(5));
        sink.append(decoder);

        sink.sleep_until_end();
        println!("Done");
    });
}
