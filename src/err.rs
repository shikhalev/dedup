use std::{
  convert::Infallible,
  env::VarError,
  error,
  ffi::{NulError, OsString},
  fmt, io, result,
};

use shellexpand::LookupError;

#[derive(Debug)]
pub enum Error {
  IOError(io::Error),
  ExpandError(LookupError<VarError>),
  UnicodeError { lossy: String },
  NulError(NulError),
  Unspecified(String),
}

pub type Result<T> = result::Result<T, Error>;

impl From<io::Error> for Error {
  fn from(src: io::Error) -> Self {
    Self::IOError(src)
  }
}

impl From<Infallible> for Error {
  fn from(src: Infallible) -> Self {
    panic!("{:#?}", src)
  }
}

impl From<LookupError<VarError>> for Error {
  fn from(src: LookupError<VarError>) -> Self {
    Self::ExpandError(src)
  }
}

impl From<OsString> for Error {
  fn from(src: OsString) -> Self {
    Self::UnicodeError {
      lossy: src.to_string_lossy().to_string(),
    }
  }
}

impl From<NulError> for Error {
  fn from(src: NulError) -> Self {
    Self::NulError(src)
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match &self {
      Error::IOError(e) => write!(f, "IO Error: {}", e),
      Error::ExpandError(e) => write!(f, "Expand Error: {}", e),
      Error::UnicodeError { lossy } => {
        write!(f, "Unicode Error: lossy string = {:?}", lossy)
      }
      Error::NulError(e) => write!(f, "NulError: {}", e),
      Error::Unspecified(s) => write!(f, "Unspecified Error: {}", s),
    }
  }
}

impl error::Error for Error {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match &self {
      Error::IOError(e) => Some(e),
      Error::ExpandError(e) => Some(e),
      Error::NulError(e) => Some(e),
      _ => None,
    }
  }

  fn description(&self) -> &str {
    "description() is deprecated; use Display"
  }

  fn cause(&self) -> Option<&dyn error::Error> {
    self.source()
  }
}
