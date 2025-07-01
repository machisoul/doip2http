use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

static PROTO_VERSION: u8 = 0x02;
static INVER_PROTO_VERSION: u8 = 0xfd;

#[repr(u16)]
pub enum VehicleConnectionPayloadType {
  GenericNegativeAck = 0x0000,
  VehicleIdRequest = 0x0001,
  VehicleIdRequestWithEID = 0x0002,
  VehicleIdRequestWithVIN = 0x0003,
  VehicleAnnouncement = 0x0004,
  RoutingActivationRequest = 0x0005,
  RoutingActivationResponse = 0x0006,
  AliveCheckRequest = 0x0007,
  AliveCheckResponse = 0x0008,
}

#[repr(u16)]
enum EntityStatusPayloadType {
  EntityStatusRequest = 0x4001,
  EntityStatusResponse = 0x4002,
  PowerModeInfoRequest = 0x4003,
  PowerModeInfoResponse = 0x4004,
}

#[repr(u16)]
pub enum DiagnosticPayloadType {
  DiagnosticMessage = 0x8001,
  DiagnosticPositiveAck = 0x8002,
  DiagnosticNegativeAck = 0x8003,
}

pub struct DoipClient {
  stream: Option<TcpStream>,
  connected: bool,
}

fn encode_doip_message(payload_type: u16, uds_msg: Option<&[u8]>) -> Vec<u8> {
  let mut message = Vec::new();

  message.push(PROTO_VERSION);
  message.push(INVER_PROTO_VERSION);
  message.push((payload_type >> 8) as u8);
  message.push((payload_type & 0xFF) as u8);

  let payload = uds_msg.unwrap_or(&[]);
  let payload_len = payload.len() as u32;
  message.extend_from_slice(&payload_len.to_be_bytes());

  message.extend_from_slice(payload);

  message
}

impl DoipClient {
  pub fn new(ecu_ip: String) -> Self {
    match TcpStream::connect(format!("{}:13400", ecu_ip)) {
      Ok(stream) => Self {
        stream: Some(stream),
        connected: true,
      },
      Err(_) => Self {
        stream: None,
        connected: false,
      },
    }
  }

  pub fn is_connected(&self) -> bool {
    self.connected
  }

  pub fn send_and_receive(
    &mut self,
    payload_type: u16,
    uds_data: Option<&[u8]>,
  ) -> Result<Vec<u8>, std::io::Error> {
    if let Some(stream) = self.stream.as_mut() {
      let message = encode_doip_message(payload_type, uds_data);
      stream.write_all(&message)?; // send the message completely

      let mut header = [0u8; 8]; // DoIP general header is 8 bytes
      stream.read_exact(&mut header)?; // read header fully
      println!("Header: {:x?}", header);

      // Parse header: protocol version, inverse version, payload type, payload length
      let payload_len = u32::from_be_bytes([header[4], header[5], header[6], header[7]]);
      let mut payload = vec![0u8; payload_len as usize];
      stream.read_exact(&mut payload)?; // read the payload fully

      Ok(payload) // return the response payload
    } else {
      Err(std::io::Error::new(
        std::io::ErrorKind::NotConnected,
        "Not connected to ECU",
      ))
    }
  }
}
