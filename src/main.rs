mod echo;
mod err;
mod file;
mod logger;
mod options;
mod process;

fn main() {
  if options::OPTS.paths.len() == 0 {
    logger::error("No root paths specified!");
  } else {
    for path in &options::OPTS.paths {
      match file::expand_path(path) {
        Ok(p) => process::process_path(&p),
        Err(e) => logger::error(&e.to_string()),
      }
    }
  }
}
