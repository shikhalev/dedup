mod logger;
mod opts;

fn main() {
  println!("\nfiles = {:?}", *opts::OPTS);
  logger::error("TEST");
  let s: String = "Test String".to_string();
  dbg!(&s);
  logger::error(&s);
}
