use crate::{
  logger,
  options::{SymlinkMode, OPTS},
};
use std::{
  fs::{self, Metadata},
  path::PathBuf,
};

fn process_dir(path: &PathBuf, _: &Metadata) {
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

fn process_file(path: &PathBuf, md: &Metadata) {
  dbg!(&path, &md);
  todo!();
}

// TODO: Красиво логировать skip, dir и вообще...
fn process_symlink(path: &PathBuf, _: &Metadata) {
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
