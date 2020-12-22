use crate::{
  echo::{echo, Level},
  file::expand_path,
  options::{ErrorMode, Verbose, OPTS},
};
use chrono::Local;
use clap::lazy_static::lazy_static;
use std::{
  fs::{File, OpenOptions},
  io::Write,
  sync::Mutex,
};

struct Logger {
  file: Option<File>,
}

impl Logger {
  #[inline]
  fn open_file() -> Option<File> {
    if OPTS.log_needed() {
      match expand_path(OPTS.log_path()) {
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
              echo(Level::Error, &e.to_string());
              None
            }
          }
        }
        Err(e) => {
          echo(Level::Error, &e.to_string());
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
    if OPTS.on_error != ErrorMode::Ignore
      && (OPTS.verbose == Verbose::All
        || OPTS.verbose == Verbose::Actions && level <= Level::Change
        || OPTS.verbose == Verbose::Errors && level <= Level::Error)
    {
      echo(level, msg);
    }
    if OPTS.on_error == ErrorMode::Abort && level == Level::Error {
      panic!("Stop on Error");
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
      None => echo(Level::Error, "Illegal write!"),
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
