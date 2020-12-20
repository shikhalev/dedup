use crate::options::OPTS;
use chrono::Local;
use clap::lazy_static::lazy_static;
use std::io::{self, Write};
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
