use crate::err::{Error, Result};
use std::{path::PathBuf, str::FromStr};

// TODO: Убрать лишнее преобразование OsStr -> str -> OsStr
pub fn expand_path(src: &PathBuf) -> Result<PathBuf> {
  match src.to_str() {
    Some(s) => Ok(PathBuf::from_str(&shellexpand::full(s)?)?.canonicalize()?),
    None => Err(Error::UnicodeError {
      lossy: src.to_string_lossy().to_string(),
    }),
  }
}
