use crate::err::{Error, Result};
use std::{ffi::CString, fs, io::{self, Read}, os::{linux::fs::MetadataExt, unix::prelude::OsStrExt}, path::PathBuf, str::FromStr};
use crc64fast::Digest;
use shellexpand;

// TODO: Убрать лишнее преобразование OsStr -> str -> OsStr
pub fn expand_path(src: &PathBuf) -> Result<PathBuf> {
  match src.to_str() {
    Some(s) => Ok(PathBuf::from_str(&shellexpand::full(s)?)?.canonicalize()?),
    None => Err(Error::UnicodeError {
      lossy: src.to_string_lossy().to_string(),
    }),
  }
}

pub fn crc64(path: &PathBuf, buffer_size: usize) -> io::Result<u64> {
  let mut file = fs::File::open(&path)?;
  let mut buffer = vec![0; buffer_size];
  let mut digest = Digest::new();
  loop {
    let l = file.read(&mut buffer)?;
    if l == 0 {
      break;
    }
    digest.write(&buffer[0..l]);
  }
  Ok(digest.sum64())
}

pub fn compare_content(first_path: &PathBuf, second_path: &PathBuf, buffer_size: usize) -> io::Result<bool> {
  let mut f1 = fs::File::open(&first_path)?;
  let mut f2 = fs::File::open(&second_path)?;
  let mut b1 = vec![0; buffer_size];
  let mut b2 = vec![0; buffer_size];
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

pub fn temp_name(path: &PathBuf) -> Result<PathBuf> {
  //let str_path = path.to_str();
  match path.to_str() {
    Some(s) => {
      let mut i = 0;
      let mut result = PathBuf::from_str(&format!("{}_{}", s, i))?;
      while result.exists() {
        i += 1;
        result = PathBuf::from_str(&format!("{}_{}", s, i))?;
      }
      eprintln!("temp_name = {:?} [{:?}]", &result, &path);
      Ok(result)
    }
    None => Err(Error::UnicodeError {
      lossy: path.to_string_lossy().to_string(),
    }),
  }
}

pub fn copy_permissions(from: &PathBuf, to: &PathBuf) -> Result<()> {
  fs::set_permissions(to, from.metadata()?.permissions())?;
  Ok(())
}

pub fn copy_owner(from: &PathBuf, to: &PathBuf) -> Result<()> {
  let c_name = CString::new(to.as_os_str().as_bytes())?;
  let md = from.metadata()?;
  if unsafe { libc::chown(c_name.as_ptr(), md.st_uid(), md.st_gid()) } == 0 {
    Ok(())
  } else {
    Err(Error::Unspecified("Error in libc::chown".to_string()))
  }
}

pub fn replace(path: &PathBuf, temp: &PathBuf) -> Result<()> {
  fs::remove_file(path)?;
  fs::rename(temp, path)?;
  Ok(())
}
