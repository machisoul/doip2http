mod doip_client;
mod uds_client;

use doip_client::DoipClient;
use doip_client::VehicleConnectionPayloadType;
use std::thread::sleep_ms;
use uds_client::UdsClient;

fn doip_test() {
  let mut doip_client = DoipClient::new("10.113.129.22".to_string());

  let uds_msg = vec![
    0x0e, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff,
  ];

  if doip_client.is_connected() {
    match doip_client.send_and_receive(
      VehicleConnectionPayloadType::RoutingActivationRequest as u16,
      Some(&uds_msg),
    ) {
      Ok(response) => println!("Response: {:x?}", response),
      Err(e) => println!("Error: {}", e),
    }
    sleep_ms(5000);
  }
}

fn main() {
  doip_test();
  println!("Hello, world!");
}
