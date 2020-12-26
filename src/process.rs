use crate::{
  logger,
  options::{SymlinkMode, OPTS},
};
use clap::lazy_static::lazy_static;
use crc64fast::Digest;
use std::{
  collections::HashMap,
  fs,
  io::{self, Read},
  path::PathBuf,
  sync::Mutex,
};

fn process_dir(path: &PathBuf, _: &fs::Metadata) {
  logger::file(&path.to_string_lossy());
  match fs::read_dir(path) {
    Ok(rd) => {
      for entry in rd {
        match entry {
          Ok(en) => process_path(&en.path()),
          Err(e) => logger::error(&e.to_string()),
        }
      }
    }
    Err(e) => logger::error(&e.to_string()),
  }
}

type Files = HashMap<
  u64, // dev
  HashMap<
    u64, // len
    HashMap<
      u64, // CRC
      HashMap<
        u64, // ino
        PathBuf,
      >,
    >,
  >,
>;

lazy_static! {
  static ref FILES: Mutex<Files> = Mutex::new(Files::new());
}

fn file_crc64(path: &PathBuf) -> io::Result<u64> {
  let mut file = fs::File::open(&path)?;
  let mut buffer = Vec::<u8>::with_capacity(OPTS.buffer_size);
  let mut digest = Digest::new();
  loop {
    let l = file.read(&mut buffer)?;
    if l == 0 {
      break;
    }
    digest.write(&buffer);
  }
  Ok(digest.sum64())
}

fn process_file(path: &PathBuf, md: &fs::Metadata) {
  logger::file(&path.to_string_lossy());
  //
  todo!();
}

// TODO: Красиво логировать skip, dir и вообще...
fn process_symlink(path: &PathBuf, _: &fs::Metadata) {
  logger::file(&path.to_string_lossy());
  match OPTS.on_symlink {
    SymlinkMode::Ignore => {}
    SymlinkMode::Follow => match fs::read_link(&path) {
      Ok(p) => process_path(&p),
      Err(e) => logger::error(&e.to_string()),
    },
    SymlinkMode::Process => todo!(),
  }
}

pub fn process_path(path: &PathBuf) {
  match path.metadata() {
    Ok(md) => {
      let ft = md.file_type();
      if ft.is_symlink() {
        process_symlink(&path, &md);
      } else if ft.is_dir() {
        process_dir(&path, &md);
      } else if ft.is_file() {
        process_file(&path, &md);
      }
    }
    Err(e) => logger::error(&e.to_string()),
  }
}
