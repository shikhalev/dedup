use crate::{err::Result, options::{ExternalFSMode, SymlinkMode, OPTS}, file};
use clap::lazy_static::lazy_static;
use std::{
  collections::HashMap,
  fs,
  path::PathBuf,
  sync::Mutex,
};
use std::os::linux::fs::MetadataExt;

fn process_dir(path: &PathBuf) {
  eprintln!("process_dir = {:?}", &path);
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

fn make_temp_hardlink(path: &PathBuf, target: &PathBuf) -> Result<PathBuf> {
  let new_name = file::temp_name(path)?;
  fs::hard_link(target, &new_name)?;
  Ok(new_name)
}

fn make_link(path: &PathBuf, target: &PathBuf) {
  eprintln!("make_link = {:?} => {:?}", &path, &target);
  if OPTS.scan_only {
    eprintln!("scan_only");
    return;
  }
  match make_temp_hardlink(path, target) {
    Ok(temp) => {
      match file::copy_permissions(path, &temp) {
        Ok(_) => {}
        Err(e) => eprintln!("{:#?}", e),
      }
      match file::copy_owner(path, &temp) {
        Ok(_) => {}
        Err(e) => eprintln!("{:#?}", e),
      }
      // TODO: xattr & ACL

      match file::replace(path, &temp) {
          Ok(_) => eprintln!("done"),
          Err(e) => {
            eprintln!("{:#?}", e);
            match fs::remove_file(&temp) {
                Ok(_) => {}
                Err(e) => eprintln!("{:#?}", e)
            }
          }
      }
    }
    Err(e) => eprintln!("{:#?}", e),
  }
}

fn process_file(path: &PathBuf, md: &fs::Metadata) {
  eprintln!("process_file = {:?} [ino = {}]", &path, &md.st_ino());

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

  let crc = match file::crc64(&path, OPTS.buffer_size) {
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
      match file::compare_content(&p, &path, OPTS.buffer_size) {
        Ok(eq) => {
          if eq {
            make_link(&path, &p);
            return;
          }
        }
        Err(e) => eprintln!("{:#?}", e),
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
  eprintln!("process_symlink = {:?}", &path);
  match OPTS.on_symlink {
    SymlinkMode::Ignore => {}
    SymlinkMode::Follow => match fs::canonicalize(&path) {
      Ok(p) => {
        eprintln!("{:?} => {:?}", &path, &p);
        process_path(&p)},
      Err(e) => eprintln!("{:#?}", e),
    },
  }
}

pub fn process_path(path: &PathBuf) {
  match path.symlink_metadata() {
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
    Err(e) => eprintln!("{:#?}", e),
  }
}
