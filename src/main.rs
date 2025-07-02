mod common;
mod demo;
mod doip_client;
mod uds_client;
use log::{debug, error, info, trace, warn};

use common::log::init_logger;
use demo::console_input::{prompt_ip, prompt_source_address};

use uds_client::UdsClient;

fn doip_test() {
  info!("Starting DoIP test");
  let ip = prompt_ip();
  let source_address = prompt_source_address();

  let mut uds_client = UdsClient::new(ip.to_string(), source_address);
  std::thread::sleep_ms(5000);
}

fn main() {
  init_logger();
  doip_test();
}
