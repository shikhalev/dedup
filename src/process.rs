use std::{fs::Metadata, path::PathBuf};

use crate::logger;

fn process_dir(path: &PathBuf, md: &Metadata) {
  todo!();
}

fn process_file(path: &PathBuf, md: &Metadata) {
  todo!();
}

fn process_symlink(path: &PathBuf, md: &Metadata) {
  logger::file(&path.to_string_lossy());
  todo!();
}

pub fn process_path(path: &PathBuf) {
  dbg!(&path);
  match path.metadata() {
    Ok(md) => {
      dbg!(&md);
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
