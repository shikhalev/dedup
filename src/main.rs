mod echo;
mod logger;
mod options;

fn main() {
  dbg!(&*options::OPTS);
  logger::error("TEST");
  let s: String = "Test String".to_string();
  dbg!(&s);
  logger::error(&s);
}
