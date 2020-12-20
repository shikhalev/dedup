use crate::opts::{Verbose, OPTS};
use chrono::Local;
use clap::lazy_static::lazy_static;
use shellexpand;
use std::{
  fs::{File, OpenOptions},
  io::{self, Write},
  path::PathBuf,
  str::FromStr,
  sync::Mutex,
  write,
};
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Level {
  Error,
  Change,
  File,
}

struct Logger {
  file: Option<File>,
}

impl Logger {
  // TODO: Схлопнуть ошибки
  // TODO: Убрать лишнее преобразование OsStr -> str -> OsStr
  #[inline]
  fn open_file() -> Option<File> {
    if OPTS.log_needed() {
      match OPTS.log_path().to_str() {
        Some(input_path) => match shellexpand::full(input_path) {
          Ok(expanded_path) => match PathBuf::from_str(&expanded_path) {
            Ok(path) => {
              let mut oo = OpenOptions::new();
              if path.exists() {
                oo.write(true).append(true);
              } else {
                oo.write(true).create(true).create_new(true);
              };
              match oo.open(path) {
                Ok(f) => Some(f),
                Err(e) => {
                  Self::echo(Level::Error, &e.to_string());
                  None
                }
              }
            }
            Err(e) => {
              Self::echo(Level::Error, &e.to_string());
              None
            }
          },
          Err(e) => {
            Self::echo(Level::Error, &e.to_string());
            None
          }
        },
        None => {
          Self::echo(Level::Error, "Invalid symbols in logfile path!");
          None
        }
      }
    } else {
      None
    }
  }

  fn new() -> Self {
    Logger {
      file: Self::open_file(),
    }
  }

  fn log(&mut self, level: Level, msg: &str) {
    if OPTS.log_needed()
      && (OPTS.log_verbose == Verbose::All
        || OPTS.log_verbose == Verbose::Actions && level <= Level::Change
        || OPTS.log_verbose == Verbose::Errors && level <= Level::Error)
    {
      self.save(level, msg);
    }
    if OPTS.verbose == Verbose::All
      || OPTS.verbose == Verbose::Actions && level <= Level::Change
      || OPTS.verbose == Verbose::Errors && level <= Level::Error
    {
      Self::echo(level, msg);
    }
  }

  fn save(&mut self, level: Level, msg: &str) {
    match &mut self.file {
      Some(f) => writeln!(
        f,
        "[{}] [{:?}]: {}",
        Local::now().format("%F %H:%M:%S%.6f %Z"),
        level,
        msg
      )
      .unwrap(),
      None => Logger::echo(Level::Error, "Illegal write!"),
    }
  }

  fn color_echo(level: Level, msg: &str) -> io::Result<()> {
    let mut out = StandardStream::stderr(OPTS.color_mode.into());
    out.reset()?;
    out.set_color(&*TIME_COLORSPEC)?;
    write!(out, "[{}] ", Local::now().format("%H:%M:%S%.6f"))?;
    out.set_color(level_spec(level))?;
    write!(out, "[{:?}]", level)?;
    out.reset()?;
    writeln!(out, ": {}", msg)?;
    Ok(())
  }

  fn echo(level: Level, msg: &str) {
    match Self::color_echo(level, msg) {
      Ok(_) => {}
      Err(_) => eprintln!(
        "[{}] [{:?}]: {}",
        Local::now().format("%H:%M:%S%.6f"),
        level,
        msg
      ),
    }
  }
}

lazy_static! {
  static ref LOGGER: Mutex<Logger> = Mutex::new(Logger::new());
}

pub fn log(level: Level, msg: &str) {
  LOGGER.lock().unwrap().log(level, msg);
}

// TODO: Переписать в макросы с форматированием

#[inline]
pub fn error(msg: &str) {
  log(Level::Error, msg)
}

#[inline]
pub fn change(msg: &str) {
  log(Level::Change, msg)
}

#[inline]
pub fn file(msg: &str) {
  log(Level::File, msg)
}

fn fg_spec(color: Color) -> ColorSpec {
  let mut r = ColorSpec::new();
  r.set_fg(Some(color)).set_intense(true);
  r
}

lazy_static! {
  static ref TIME_COLORSPEC: ColorSpec = fg_spec(Color::Blue);
  static ref ERROR_COLORSPEC: ColorSpec = fg_spec(Color::Red);
  static ref CHANGE_COLORSPEC: ColorSpec = fg_spec(Color::Yellow);
  static ref FILE_COLORSPEC: ColorSpec = fg_spec(Color::Cyan);
}

#[inline]
fn level_spec(level: Level) -> &'static ColorSpec {
  match level {
    Level::Error => &*ERROR_COLORSPEC,
    Level::Change => &*CHANGE_COLORSPEC,
    Level::File => &*FILE_COLORSPEC,
  }
}
