use std::fmt::{Display, Formatter};
use std::io::stdout;
use std::process::ExitCode;

use crossterm::event::DisableMouseCapture;
use crossterm::event::EnableMouseCapture;
use crossterm::event::{poll, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{
    cursor, event, execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    ErrorKind, ExecutableCommand,
};
use time::{format_description::FormatItem, macros::format_description, OffsetDateTime};

const TWELVE_HOUR_HMS: &[FormatItem] =
    format_description!("[hour repr:12 padding:none]:[minute]:[second] [period]");
const TWELVE_HOUR_HM: &[FormatItem] =
    format_description!("[hour repr:12 padding:none]:[minute] [period]");
const TWENTY_FOUR_HOUR_HMS: &[FormatItem] = format_description!("[hour]:[minute]:[second]");
const TWENTY_FOUR_HOUR_HM: &[FormatItem] = format_description!("[hour]:[minute]");

struct Options {
    twenty_four_hour: bool,
    show_seconds: bool,
}

#[derive(Debug)]
enum Error {
    ExitCode(ExitCode),
    Usage(String),
    Terminal(crossterm::ErrorKind),
}

fn main() -> ExitCode {
    match try_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(Error::ExitCode(code)) => code,
        Err(Error::Usage(message)) => {
            eprintln!("{}", message);
            usage();
            ExitCode::from(2)
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            ExitCode::FAILURE
        }
    }
}

fn try_main() -> Result<(), Error> {
    let options = parse_args()?;

    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnableMouseCapture)?;
    main_loop(options.format())?;
    execute!(stdout, DisableMouseCapture)?;

    disable_raw_mode()?;

    Ok(())
}

fn main_loop(format: &[FormatItem]) -> Result<(), Error> {
    loop {
        // Wait up to 1s for another event
        if poll(std::time::Duration::from_millis(1_000))? {
            // It's guaranteed that read() won't block if `poll` returns `Ok(true)`
            let event = event::read()?;

            println!("Event::{:?}\r", event);

            if event == Event::Key(KeyCode::Char('c').into()) {
                println!("Cursor position: {:?}\r", cursor::position());
            }

            if event == Event::Key(KeyCode::Esc.into()) {
                break;
            }
        } else {
            // Timeout expired, no event for 1s
            let now = OffsetDateTime::now_local().unwrap();
            let time_str = now.format(format).unwrap();
            let time = format!("{}", segmentify(&time_str));
            println!("{}", time);
        }
    }

    Ok(())
}
fn segmentify(s: &str) -> String {
    s.chars()
        .map(|ch| {
            if ch.is_ascii_digit() {
                std::char::from_u32(0x1FBC0 + ch as u32).unwrap()
            } else {
                ch
            }
        })
        .collect()
}

fn parse_args() -> Result<Options, Error> {
    let mut options = Options::default();
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "-h" | "--help" => {
                usage();
                return Err(Error::ExitCode(ExitCode::SUCCESS));
            }
            "-24" => options.twenty_four_hour = true,
            "--seconds" => options.show_seconds = true,
            otherwise => return Err(Error::Usage(format!("unknown option: '{}'", otherwise))),
        }
    }

    Ok(options)
}

fn usage() {
    eprintln!(r#"Usage: 7clock"#)
}

impl Options {
    fn format(&self) -> &[FormatItem] {
        match (self.twenty_four_hour, self.show_seconds) {
            (true, true) => TWENTY_FOUR_HOUR_HMS,
            (true, false) => TWENTY_FOUR_HOUR_HM,
            (false, true) => TWELVE_HOUR_HMS,
            (false, false) => TWELVE_HOUR_HM,
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        Options {
            twenty_four_hour: false,
            show_seconds: false,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ExitCode(_code) => write!(f, "exit code"),
            Error::Usage(message) => write!(f, "usage error: {message}"),
            Error::Terminal(kind) => write!(f, "terminal error: {kind}"),
        }
    }
}

impl From<crossterm::ErrorKind> for Error {
    fn from(err: ErrorKind) -> Self {
        Error::Terminal(err)
    }
}

impl std::error::Error for Error {}
