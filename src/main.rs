mod doip_client;
mod uds_client;

use doip_client::DoipClient;
use doip_client::VehicleConnectionPayloadType;
use std::thread::sleep_ms;
use uds_client::UdsClient;

fn doip_test() {
  let mut uds_client = UdsClient::new("192.168.1.78".to_string(), 0x0e80);
}

fn main() {
  doip_test();
  println!("Hello, world!");
}
