use env_logger::{Builder, Env};
use std::io::{self, Write};

pub fn init_logger() {
  // 1. Init logger early, with defaults
  let env = Env::default()
    .filter_or("DOIP2HTTP_LOG_LEVEL", "info")
    .write_style_or("DOIP2HTTP_LOG_STYLE", "auto");

  Builder::from_env(env)
    .format(|buf, record| {
      writeln!(
        buf,
        "{} [{}] {}: {}",
        buf.timestamp(), // timestamp
        record.level(),  // log level
        record.module_path().unwrap_or("<unknown>"),
        record.args()
      )
    })
    .init();
}
