use log::{self, Level, LevelFilter, Log, Metadata, Record};

use crate::console::print;

macro_rules! with_color {
    ($color_code:expr, $($arg:tt)*) => {{
        format_args!("\u{1B}[{}m{}\u{1B}[m", $color_code as u8, format_args!($($arg)*))
    }};
}

pub fn init() {
    static LOGGER: SimpleLogger = SimpleLogger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(match option_env!("LOG") {
        Some("error") => LevelFilter::Error,
        Some("warn") => LevelFilter::Warn,
        Some("info") => LevelFilter::Info,
        Some("debug") => LevelFilter::Debug,
        Some("trace") => LevelFilter::Trace,
        _ => LevelFilter::Debug,
    });
}

#[repr(u8)]
#[allow(dead_code)]
enum ColorCode {
    Black = 30,
    Red = 31,
    Green = 32,
    Yellow = 33,
    Blue = 34,
    Magenta = 35,
    Cyan = 36,
    White = 37,
    BrightBlack = 90,
    BrightRed = 91,
    BrightGreen = 92,
    BrightYellow = 93,
    BrightBlue = 94,
    BrightMagenta = 95,
    BrightCyan = 96,
    BrightWhite = 97,
}

struct SimpleLogger;

impl Log for SimpleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let level = record.level();
        let line = record.line().unwrap_or(0);
        let target = record.target();
        let level_color = match level {
            Level::Error => ColorCode::BrightRed,
            Level::Warn => ColorCode::BrightYellow,
            Level::Info => ColorCode::BrightGreen,
            Level::Debug => ColorCode::BrightCyan,
            Level::Trace => ColorCode::BrightBlack,
        };
        let args_color = match level {
            Level::Error => ColorCode::Red,
            Level::Warn => ColorCode::Yellow,
            Level::Info => ColorCode::Green,
            Level::Debug => ColorCode::Cyan,
            Level::Trace => ColorCode::BrightBlack,
        };
        // if super::init_ok() {
        //     let now = crate::timer::current_time();
        //     print(with_color!(
        //         ColorCode::White,
        //         "[{:>3}.{:06} {} {} {}\n",
        //         now.as_secs(),
        //         now.subsec_micros(),
        //         with_color!(level_color, "{:<5}", level),
        //         with_color!(ColorCode::White, "{}:{}]", target, line),
        //         with_color!(args_color, "{}", record.args()),
        //     ));
        // } else {
        //     print(with_color!(
        //         ColorCode::White,
        //         "[{} {} {}\n",
        //         with_color!(level_color, "{:<5}", level),
        //         with_color!(ColorCode::White, "{}:{}]", target, line),
        //         with_color!(args_color, "{}", record.args()),
        //     ));
        // }

        print(with_color!(
            ColorCode::White,
            "[{} {} {}\n",
            with_color!(level_color, "{:<5}", level),
            with_color!(ColorCode::White, "{}:{}]", target, line),
            with_color!(args_color, "{}", record.args()),
        ));
    }

    fn flush(&self) {}
}
