use std::fmt::{Display, Formatter};
use std::io::{stdout, Stdout, Write};
use std::process::ExitCode;

use crossterm::cursor::{MoveToColumn, MoveToRow};
use crossterm::event::{poll, Event, KeyCode};
use crossterm::style::{Color, SetForegroundColor};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{cursor, event, execute, style::Print, terminal, ErrorKind};
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
    colour: Option<Color>,
}

#[derive(Debug)]
enum Error {
    ExitCode(ExitCode),
    Usage(String),
    Message(String),
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
        Err(Error::Message(message)) => {
            eprintln!("{}", message);
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
    execute!(stdout, EnterAlternateScreen)?;
    main_loop(&options)?;
    execute!(stdout, LeaveAlternateScreen)?;

    disable_raw_mode()?;

    Ok(())
}

fn main_loop(options: &Options) -> Result<(), Error> {
    let mut stdout = stdout();
    let (mut columns, mut rows) = terminal::size()?;
    let format = options.format();

    // Clear the screen, move to middle row, and do the initial render
    init_screen(&mut stdout, columns, rows, options.colour)?;
    render_time(&mut stdout, format, columns)?;

    loop {
        // Wait up to 1s for another event
        if poll(options.poll_interval())? {
            // It's guaranteed that read() won't block if `poll` returns `Ok(true)`
            match event::read()? {
                Event::Resize(new_cols, new_rows) => {
                    columns = new_cols;
                    rows = new_rows;
                    init_screen(&mut stdout, columns, rows, options.colour)?;
                    render_time(&mut stdout, format, columns)?;
                }
                Event::Key(key_event)
                    if key_event == KeyCode::Esc.into()
                        || key_event == KeyCode::Char('q').into() =>
                {
                    break;
                }
                _ => {}
            }
        } else {
            // Timeout expired, no event for 1s
            render_time(&mut stdout, format, columns)?;
        }
    }

    execute!(stdout, cursor::Show, SetForegroundColor(Color::Reset))?;

    Ok(())
}

fn render_time(stdout: &mut Stdout, format: &[FormatItem], columns: u16) -> Result<(), Error> {
    let now = OffsetDateTime::now_local().unwrap();
    let time_str = now.format(format).unwrap();
    let (time, time_len) = segmentify(&time_str);

    execute!(
        stdout,
        Clear(ClearType::CurrentLine),
        MoveToColumn((columns / 2).saturating_sub(time_len as u16 / 2)),
        Print(time)
    )?;
    Ok(())
}

fn init_screen<S: Write>(
    screen: &mut S,
    _cols: u16,
    rows: u16,
    colour: Option<Color>,
) -> Result<(), Error> {
    if let Some(colour) = colour {
        execute!(
            screen,
            Clear(ClearType::All),
            MoveToRow(rows / 2),
            cursor::Hide,
            SetForegroundColor(colour)
        )?;
    } else {
        execute!(
            screen,
            Clear(ClearType::All),
            MoveToRow(rows / 2),
            cursor::Hide
        )?;
    }
    Ok(())
}

fn segmentify(s: &str) -> (String, usize) {
    let mut len = 0;
    (
        s.chars()
            .map(|ch| {
                len += 1;
                if ch.is_ascii_digit() {
                    std::char::from_u32(0x1FBC0 + ch as u32).unwrap()
                } else {
                    ch
                }
            })
            .collect::<String>(),
        len,
    )
}

fn parse_args() -> Result<Options, Error> {
    let mut options = Options::default();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                usage();
                return Err(Error::ExitCode(ExitCode::SUCCESS));
            }
            "-24" => options.twenty_four_hour = true,
            "-c" | "--color" | "--colour" => {
                options.colour =
                    Some(parse_colour(&args.next().ok_or_else(|| {
                        Error::Usage("--colour requires an argument".into())
                    })?)?);
            }
            "--seconds" => options.show_seconds = true,
            otherwise => return Err(Error::Usage(format!("unknown option: '{}'", otherwise))),
        }
    }

    Ok(options)
}

fn parse_colour(s: &str) -> Result<Color, Error> {
    if s.starts_with('#') {
        parse_hex(&s[1..])
    } else {
        Color::try_from(s).map_err(|()| Error::Message(format!("unable to parse colour: '{}'", s)))
    }
}

fn parse_hex(hex: &str) -> Result<Color, Error> {
    if hex.len() != 6 {
        return Err(Error::Message(format!("invalid colour: '#{}'", hex)));
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok();
    let g = u8::from_str_radix(&hex[2..4], 16).ok();
    let b = u8::from_str_radix(&hex[4..6], 16).ok();
    match (r, g, b) {
        (Some(r), Some(g), Some(b)) => Ok(Color::from((r, g, b))),
        _ => Err(Error::Message(format!("invalid colour: '#{}'", hex))),
    }
}

fn usage() {
    eprintln!(
        "{}

{bin} displays a clock using seven-segment characters.

USAGE:
    {bin} [OPTIONS]

OPTIONS:
    -h, --help
            Prints this help information.

    -24
            Use 24-hour time.

    -c, --color, --colour COLOUR
            Set the colour of the clock.
            COLOUR can be an RGB hex colour (#RRGGBB) or one of the eight
            standard colour names: black, red, green, yellow, blue, magenta, cyan,
            or white.

    --seconds
            Include seconds.

AUTHOR
    Wesley Moore <wes@wezm.net>

SEE ALSO
    https://github.com/wezm/7clock  Source code and issue tracker.",
        version_string(),
        bin = "7clock"
    );
}

pub fn version_string() -> String {
    format!(
        "{} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    )
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

    fn poll_interval(&self) -> std::time::Duration {
        let interval = if self.show_seconds { 500 } else { 1000 };
        std::time::Duration::from_millis(interval)
    }
}

impl Default for Options {
    fn default() -> Self {
        Options {
            twenty_four_hour: false,
            show_seconds: false,
            colour: None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ExitCode(_code) => write!(f, "exit code"),
            Error::Usage(message) => write!(f, "usage error: {message}"),
            Error::Message(message) => write!(f, "error: {message}"),
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
