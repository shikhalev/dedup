mod err;
mod file;
mod options;
mod process;

fn main() {
  dbg!(&*options::OPTS);
  if options::OPTS.paths.len() == 0 {
  } else {
    for path in &options::OPTS.paths {
      match file::expand_path(path) {
        Ok(p) => process::process_path(&p),
        Err(e) => eprintln!("{:#?}", e),
      }
    }
  }
}
