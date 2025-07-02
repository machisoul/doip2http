use log::{debug, error, info, trace, warn};
use std::io::{self, Write};
use std::net::IpAddr;

pub fn prompt_ip() -> IpAddr {
  loop {
    print!("Enter IP address: ");
    io::stdout().flush().unwrap();
    let mut ip_input = String::new();
    io::stdin().read_line(&mut ip_input).unwrap();
    let ip_input = ip_input.trim();

    match ip_input.parse::<IpAddr>() {
      Ok(ip) => return ip,
      Err(_) => warn!("Invalid IP address. Please try again."),
    }
  }
}

pub fn prompt_source_address() -> u16 {
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

    warn!("Invalid source address. Please enter a valid 2-byte number.");
  }
}
