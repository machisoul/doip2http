mod doip_client;
mod uds_client;
use std::io::{self, Write};
use std::net::IpAddr;

use doip_client::DoipClient;
use doip_client::VehicleConnectionPayloadType;
use std::thread::sleep_ms;
use uds_client::UdsClient;

fn prompt_ip() -> IpAddr {
  loop {
    print!("Enter IP address: ");
    io::stdout().flush().unwrap();
    let mut ip_input = String::new();
    io::stdin().read_line(&mut ip_input).unwrap();
    let ip_input = ip_input.trim();

    match ip_input.parse::<IpAddr>() {
      Ok(ip) => return ip,
      Err(_) => println!("Invalid IP address. Please try again."),
    }
  }
}

fn prompt_source_address() -> u16 {
  loop {
    print!("Enter client source address (hex, e.g., 0x1234): ");
    io::stdout().flush().unwrap();
    let mut addr_input = String::new();
    io::stdin().read_line(&mut addr_input).unwrap();
    let addr_input = addr_input.trim();

    if let Some(stripped) = addr_input.strip_prefix("0x") {
      if let Ok(addr) = u16::from_str_radix(stripped, 16) {
        return addr;
      }
    } else if let Ok(addr) = addr_input.parse::<u16>() {
      return addr;
    }

    println!("Invalid source address. Please enter a valid 2-byte number.");
  }
}

fn doip_test() {
  let ip = prompt_ip();
  let source_address = prompt_source_address();

  let mut uds_client = UdsClient::new(ip.to_string(), source_address);
  std::thread::sleep_ms(5000);
}

fn main() {
  doip_test();
  println!("Hello, world!");
}
