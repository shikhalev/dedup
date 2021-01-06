use clap::{lazy_static::lazy_static, AppSettings, Clap, ValueHint};
use std::os::linux::fs::MetadataExt;
use std::{path::PathBuf, str::FromStr};
// use termcolor::ColorChoice;

// #[derive(Clone, Copy, Clap, Debug, PartialEq)]
// pub enum Verbose {
//   All,
//   Actions,
//   Errors,
//   None,
// }

// #[derive(Clone, Copy, Clap, Debug, PartialEq)]
// pub enum ColorMode {
//   Always,
//   #[clap(visible_alias = "ansi")]
//   AlwaysAnsi,
//   #[clap(visible_alias = "default")]
//   Auto,
//   Never,
// }

// impl Into<ColorChoice> for ColorMode {
//   fn into(self) -> ColorChoice {
//     match self {
//       ColorMode::Always => ColorChoice::Always,
//       ColorMode::AlwaysAnsi => ColorChoice::AlwaysAnsi,
//       ColorMode::Auto => ColorChoice::Auto,
//       ColorMode::Never => ColorChoice::Never,
//     }
//   }
// }

#[derive(Clone, Copy, Clap, Debug, PartialEq)]
pub enum SymlinkMode {
  Ignore,
  Follow,
  Process,
}

#[derive(Clone, Copy, Clap, Debug, PartialEq)]
pub enum ErrorMode {
  Ignore,
  Warning,
  #[clap(visible_alias = "panic")]
  Abort,
}

#[derive(Clone, Copy, Clap, Debug, PartialEq)]
pub enum ExternalFSMode {
  Ignore,
  Group,
  Error,
  Symlink,
}

fn parse_bytes(src: &str) -> usize {
  use byte_unit::Byte;
  Byte::from_str(src).unwrap().get_bytes() as usize
}

#[derive(Clap, Debug)]
#[clap(version, author, about, setting = AppSettings::ColoredHelp, setting = AppSettings::UnifiedHelpMessage)]
pub struct Opts {
  // #[clap(
  //   short,
  //   long = "color",
  //   arg_enum,
  //   default_value = "auto",
  //   value_name = "mode"
  // )]
  // pub color_mode: ColorMode,
  /// What do (or not) when error occurred.
  ///
  /// `warning` - print message on `stderr`;
  /// `ignore` - do nothing, silence mode;
  /// `abort` - stop program execution.
  #[clap(
    short = 'e',
    long,
    arg_enum,
    default_value = "warning",
    value_name = "mode"
  )]
  pub on_error: ErrorMode,

  /// No changes, only check files.
  #[clap(short, long)]
  pub scan_only: bool,

  /// Don't show total count of files and bytes at end of work
  #[clap(short = 'S', long = "no-summary", parse(from_flag = std::ops::Not::not))]
  pub summary: bool,

  /// What do (or not) when file is symlink.
  ///
  /// `ignore` - skip symlinks;
  /// `follow` - check symlink target (include directories);
  /// `process` - process a symlink as regular file.
  #[clap(
    short = 'y',
    long,
    arg_enum,
    value_name = "mode",
    default_value = "ignore"
  )]
  pub on_symlink: SymlinkMode,

  /// Use symlinks instead hardlinks. Dangerous option!
  #[clap(short = 'Y', long)]
  pub use_symlinks: bool,

  /// What do (or not) if some files are in different filesystems.
  ///
  /// May be one of follow:
  /// `ignore` - skip files on non-primary filesystems;
  /// `group` - process all files grouped by filesystem;
  /// `error` - files on non-primary filesystems causes error;
  /// `symlink` (dangerous!) - files on non-primary filesystems
  /// replaces with symlink to primary.
  /// If any value except `group` is used the `--primary-fs` option is required.
  #[clap(
    short = 'x',
    long,
    value_name = "mode",
    arg_enum,
    default_value = "group",
    requires_ifs(&[
      ("ignore", "primary-fs"),
      ("error", "primary-fs"),
      ("symlink", "primary-fs")
    ]),
  )]
  pub on_external_fs: ExternalFSMode,

  /// Path to any file on primary filesystem. Used with `--on-external-fs`.
  #[clap(short = 'f', long, parse(from_os_str), value_hint = ValueHint::AnyPath, value_name = "fs")]
  primary_fs: Option<PathBuf>,

  /// Ignore files less than specified size.
  ///
  /// May be number or number with suffix such as B (byte),
  /// KB (kilobyte = 1000B) or KiB (kibibyte = 1024B). Processed all files not less this value,
  /// if set parameter to 0, zero-sized files will be processed too.
  #[clap(
    short,
    long,
    parse(from_str = parse_bytes),
    default_value = "1B",
    value_name = "size",
  )]
  pub ignore_less: usize,

  /// Buffer size for I/O operations.
  ///
  /// May be number or number with suffix such as B (byte),
  /// KB (kilobyte = 1000B) or KiB (kibibyte = 1024B).
  #[clap(short, long, parse(from_str = parse_bytes), default_value = "1MiB", value_name = "size")]
  pub buffer_size: usize,

  // /// Verbose level.
  // ///
  // /// `all` - show all checked files;
  // /// `actions` - show changes;
  // /// `errors` - show error warnings only;
  // /// `none` - full silence mode.
  // #[clap(
  //   short,
  //   long,
  //   arg_enum,
  //   default_value = "actions",
  //   value_name = "mode"
  // )]
  // pub verbose: Verbose,

  // /// Write log file.
  // ///
  // /// If file name is not specified, write to `./dedup.log`
  // #[clap(short, long, value_hint = ValueHint::FilePath, value_name = "file")]
  // pub log: Option<Option<PathBuf>>,

  // /// What will written to log.
  // ///
  // /// See `--verbose` for details.
  // #[clap(long, arg_enum, default_value = "actions", value_name = "mode")]
  // pub log_verbose: Verbose,
  /// Exclude some names or paths from processing.
  #[clap(
    short = 'X',
    long,
    parse(from_os_str),
    value_name = "name",
    multiple = true,
    use_delimiter = true,
    value_delimiter = ":",
    multiple_occurrences = true,
    value_hint = ValueHint::DirPath
  )]
  pub exclude: Vec<PathBuf>,

  /// Files and directories for check and process.
  #[clap(value_name = "path", parse(from_os_str), value_hint = ValueHint::DirPath)]
  pub paths: Vec<PathBuf>,

  #[clap(skip)]
  pub primary_fs_dev: Option<u64>,
}

static DEFAULT_LOG_FILENAME: &'static str = "./dedup.log";

lazy_static! {
  pub static ref DEFAULT_LOG_PATH: PathBuf =
    PathBuf::from_str(DEFAULT_LOG_FILENAME).unwrap();
}

impl Opts {
  // pub fn log_needed(&self) -> bool {
  //   self.log.is_some() && self.log_verbose != Verbose::None
  // }

  // pub fn log_path(&self) -> &PathBuf {
  //   match &self.log {
  //     Some(opt) => match &opt {
  //       Some(path) => path,
  //       None => &*DEFAULT_LOG_PATH,
  //     },
  //     None => &*DEFAULT_LOG_PATH,
  //   }
  // }

  pub fn calculate() -> Self {
    let mut result = Self::parse();
    result.primary_fs_dev = if result.on_external_fs == ExternalFSMode::Group {
      None
    } else {
      match &result.primary_fs {
        Some(path) => match path.metadata() {
          Ok(md) => Some(md.st_dev()),
          Err(_) => {
            // TODO: Более подробную ошибку
            eprintln!("Invalid primary FS path! = {:#?}", path);
            None
          }
        },
        None => None,
      }
    };
    result
  }

  pub fn check_dev(&self, dev: u64) -> bool {
    self.primary_fs_dev == None || self.primary_fs_dev == Some(dev)
  }
}

lazy_static! {
  pub static ref OPTS: Opts = Opts::calculate();
}
