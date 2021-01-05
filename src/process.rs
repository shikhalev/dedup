use crate::{
  options::{ExternalFSMode, SymlinkMode, OPTS},
};
use clap::lazy_static::lazy_static;
use crc64fast::Digest;
use std::os::linux::fs::MetadataExt;
use std::{
  collections::HashMap,
  fs,
  io::{self, Read},
  path::PathBuf,
  sync::Mutex,
};

fn process_dir(path: &PathBuf) {
  dbg!(&path);
  match fs::read_dir(path) {
    Ok(rd) => {
      for entry in rd {
        match entry {
          Ok(en) => process_path(&en.path()),
          Err(e) => eprintln!("{:#?}", e),
        }
      }
    }
    Err(e) => eprintln!("{:#?}", e),
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
  let mut buffer = vec![0; OPTS.buffer_size];
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

fn file_equal(first_path: &PathBuf, second_path: &PathBuf) -> io::Result<bool> {
  let mut f1 = fs::File::open(&first_path)?;
  let mut f2 = fs::File::open(&second_path)?;
  let mut b1 = vec![0; OPTS.buffer_size];
  let mut b2 = vec![0; OPTS.buffer_size];
  loop {
    let l1 = f1.read(&mut b1)?;
    let l2 = f2.read(&mut b2)?;
    if l1 == 0 && l2 == 0 {
      break;
    }
    if l1 != l2 || &b1[0..l1] != &b2[0..l2] {
      return Ok(false);
    }
  }
  Ok(true)
}

fn make_link(path: &PathBuf, target: &PathBuf) {
  dbg!(&path, &target);
  todo!();
  // TODO:
  //  создать hardlink в новом файле
  //  перенести владельца, права, xattr и acl если есть
  //  удалить старый файл и переименовать hardlink
}

fn process_file(path: &PathBuf, md: &fs::Metadata) {
  dbg!(&path, &md);

  let dev = md.st_dev();
  let efs_m = OPTS.on_external_fs;
  if efs_m == ExternalFSMode::Symlink {
    todo!();
  } else if efs_m == ExternalFSMode::Group {
    // nothing to do
  } else {
    if !OPTS.check_dev(dev) {
      if efs_m == ExternalFSMode::Error {
        eprintln!("External!");
      }
      return;
    }
  }

  let len = md.len();
  if (len as usize) < OPTS.ignore_less {
    // TODO: message about skip
    return;
  }

  let crc = match file_crc64(&path) {
    Ok(r) => r,
    Err(e) => {
      eprintln!("{:#?}", e);
      return;
    }
  };
  let ino = md.st_ino();

  let mut all_files = match FILES.lock() {
    Ok(v) => v,
    Err(e) => {
      eprintln!("{:#?}", e);
      return;
    }
  };
  if !all_files.contains_key(&dev) {
    all_files.insert(dev, HashMap::new());
  }

  let files_with_dev = match all_files.get_mut(&dev) {
    Some(v) => v,
    None => {
      eprintln!("Unknown error!!!");
      return;
    }
  };
  if !files_with_dev.contains_key(&len) {
    files_with_dev.insert(len, HashMap::new());
  }

  let files_with_dev_and_len = match files_with_dev.get_mut(&len) {
    Some(v) => v,
    None => {
      eprintln!("Unknown error!!!");
      return;
    }
  };
  if !files_with_dev_and_len.contains_key(&crc) {
    files_with_dev_and_len.insert(crc, HashMap::new());
  }

  let files_with_dev_and_len_and_crc = match files_with_dev_and_len.get(&crc) {
    Some(v) => v,
    None => {
      eprintln!("Unknown error!!!");
      return;
    }
  };
  if !files_with_dev_and_len_and_crc.contains_key(&ino) {
    for (_, p) in files_with_dev_and_len_and_crc {
      match file_equal(&p, &path) {
        Ok(eq) => {
          if eq {
            make_link(&path, &p);
            return;
          }
        }
        Err(e) => eprintln!("{:#?}", e)
      }
    }

    let files_with_dev_and_len_and_crc =
      match files_with_dev_and_len.get_mut(&crc) {
        Some(v) => v,
        None => {
          eprintln!("Unknown error!!!");
          return;
        }
      };
    files_with_dev_and_len_and_crc.insert(ino, path.clone());
  }
}

// TODO: Красиво логировать skip, dir и вообще...
fn process_symlink(path: &PathBuf) {
  dbg!(&path);
  match OPTS.on_symlink {
    SymlinkMode::Ignore => {}
    SymlinkMode::Follow => match fs::read_link(&path) {
      Ok(p) => process_path(&p),
      Err(e) => eprintln!("{:#?}", e)
    },
    SymlinkMode::Process => todo!(),
  }
}

pub fn process_path(path: &PathBuf) {
  match path.metadata() {
    Ok(md) => {
      let ft = md.file_type();
      if ft.is_symlink() {
        process_symlink(&path);
      } else if ft.is_dir() {
        process_dir(&path);
      } else if ft.is_file() {
        process_file(&path, &md);
      } else {
        todo!(); // TODO: show error
      }
    }
    Err(e) => eprintln!("{:#?}", e)
  }
}
