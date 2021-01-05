use crate::options::OPTS;
use chrono::Local;
use clap::lazy_static::lazy_static;
use std::{
  collections::HashMap,
  fs::{File, Metadata},
  io::{self, Write},
  path::Path,
  sync::{Mutex, MutexGuard, PoisonError},
};
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Level {
  Error,
  Change,
  File,
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

pub fn echo(level: Level, msg: &str) {
  match color_echo(level, msg) {
    Ok(_) => {}
    Err(_) => eprintln!(
      "[{}] [{:?}]: {}",
      Local::now().format("%H:%M:%S%.6f"),
      level,
      msg
    ),
  }
}

pub enum XLevel {
  Error,
  Change,
  Skip,
  Read,
}

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum Event {
  Error,
  Change,
  Skip,
  ListDir,
  ReadFile,
  Follow,
}

impl Event {
  #[inline]
  pub fn level(&self) -> XLevel {
    match &self {
      Event::Error => XLevel::Error,
      Event::Change => XLevel::Change,
      Event::Skip => XLevel::Skip,
      _ => XLevel::Read,
    }
  }

  #[inline]
  pub fn marker(&self) -> &'static str {
    match &self {
      Event::Error => "error",
      Event::Change => {
        if OPTS.scan_only {
          "found"
        } else {
          "done"
        }
      }
      Event::Skip => "skip",
      Event::ListDir => "dir",
      Event::ReadFile => "file",
      Event::Follow => "link",
    }
  }

  #[inline]
  pub fn marker_color(&self) -> &ColorSpec {
    match &self {
      Event::Error => &*RED,
      Event::Change => &*YELLOW,
      Event::Skip => &*TEAL,
      Event::ListDir | Event::ReadFile => &*GREEN,
      Event::Follow => &*CYAN,
    }
  }

  #[inline]
  pub fn message_color(&self) -> &ColorSpec {
    match &self {
      Event::Error | Event::Change => &*WHITE,
      _ => &*SILVER,
    }
  }
}

fn spec(color: Color, intense: bool) -> ColorSpec {
  let mut result = ColorSpec::new();
  result.set_fg(Some(color)).set_intense(intense);
  return result;
}

lazy_static! {
  // Standard foreground colors
  static ref SILVER: ColorSpec = spec(Color::White, false);
  static ref WHITE: ColorSpec = spec(Color::White, true);
  static ref RED: ColorSpec = spec(Color::Red, true);
  static ref YELLOW: ColorSpec = spec(Color::Yellow, true);
  static ref CYAN: ColorSpec = spec(Color::Cyan, true);
  static ref GREEN: ColorSpec = spec(Color::Green, true);
  static ref TEAL: ColorSpec = spec(Color::Cyan, false);
  static ref GRAY: ColorSpec = spec(Color::Black, true);
  static ref BLUE: ColorSpec = spec(Color::Blue, true);
  // Setup foreground colors
  static ref TIME_COLOR: &'static ColorSpec = &*BLUE;
}

pub enum SkipReason {
  Exclude,
  SmallSize,
  ExternalFS,
  Symlink,
}

pub struct PathInfo<'a> {
  path: &'a Path,
  metadata: &'a Metadata,
}

fn prepare_logfile() -> Option<Mutex<File>> {
  None // TODO:
}

lazy_static! {
  static ref STDERR: Mutex<StandardStream> =
    Mutex::new(StandardStream::stderr(OPTS.color_mode.into()));
  static ref LOGGER: Option<Mutex<File>> = prepare_logfile();
}

struct CheckOnlyError;

impl From<PoisonError<MutexGuard<'_, StandardStream>>> for CheckOnlyError {
  fn from(_: PoisonError<MutexGuard<StandardStream>>) -> Self {
    Self
  }
}

impl From<PoisonError<MutexGuard<'_, File>>> for CheckOnlyError {
  fn from(_: PoisonError<MutexGuard<File>>) -> Self {
    Self
  }
}

impl From<std::io::Error> for CheckOnlyError {
  fn from(_: std::io::Error) -> Self {
    Self
  }
}

fn color_error(
  time: &str,
  err: &dyn std::error::Error,
  path: Option<&PathInfo>,
) -> std::result::Result<(), CheckOnlyError> {
  let mut out = STDERR.lock()?;
  out.reset()?;
  out.set_color(&*BLUE)?;
  write!(out, "{} ", time)?;
  out.set_color(Event::Error.marker_color())?;
  write!(out, "[{}]: ", Event::Error.marker())?;
  out.set_color(Event::Error.message_color())?;
  write!(out, "{:#?}", err)?;
  if let Some(p) = path {
    out.set_color(&*SILVER)?;
    write!(out, " at ")?;
    out.set_color(Event::Error.message_color())?;
    write!(out, "{:?}", p.path)?;
  }
  out.reset()?;
  writeln!(out)?;
  out.flush()?;
  Ok(())
}

fn file_error(
  time: &str,
  err: &dyn std::error::Error,
  path: Option<&PathInfo>,
) -> std::result::Result<(), CheckOnlyError> {
  if let Some(out) = &*LOGGER {
    let mut out = out.lock()?;
    write!(out, "{} ", time)?;
    write!(out, "[{}]: ", Event::Error.marker())?;
    write!(out, "{:#?}", err)?;
    if let Some(p) = path {
      write!(out, " at ")?;
      write!(out, "{:?}", p.path)?;
    }
    writeln!(out)?;
    out.flush()?;
  }
  Ok(())
}

pub fn error(err: &dyn std::error::Error, path: Option<&PathInfo>) {
  let time = Local::now();
  match color_error(&time.format("%H:%M:%S%.6f").to_string(), err, path) {
    Ok(_) => {}
    Err(_) => {
      eprint!(
        "{} [{}]: {:#?}",
        time.format("%H:%M:%S%.6f"),
        Event::Error.marker(),
        err
      );
      if let Some(p) = path {
        eprint!(" at {:?}", p.path);
      }
      eprintln!();
    }
  }
  match file_error(&time.format("%F %H:%M:%S%.6f %Z").to_string(), err, path) {
    Ok(_) => {}
    Err(_) => eprintln!("Error write to log!"),
  }
}

pub fn done(path: &PathInfo, target: &PathInfo) {}
pub fn found(path: &PathInfo, target: &PathInfo) {}

pub fn follow(path: &PathInfo, target: &PathInfo) {}
pub fn enter(path: &PathInfo) {}
pub fn checked(path: &PathInfo) {}
pub fn skip(reason: SkipReason, path: &PathInfo, target: Option<&PathInfo>) {}

// log!(error <e>[ at <path>]);
// log!(done <path> to <target>); with dev & ino => ino
// log!(follow <path> to <target>);
// log!(enter <path>);
// log!(unique <path>); dev & ino
// log!(exclude <path>);
// log!(small <path with len>); len < OPTS.ignore_less
// log!(skip external <path with dev>);
// log!(skip symlink <path> to <target>);
