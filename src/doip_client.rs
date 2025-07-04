use log::{error, info, warn};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

static PROTO_VERSION: u8 = 0x02;
static INVER_PROTO_VERSION: u8 = 0xfd;

pub static DEFAULT_ACTIVATION_TYPE: u8 = 0x00;
pub static DEFAULT_ACTIVATION_RESERVED: u32 = 0x00;

// Default connection timeout in seconds
pub static DEFAULT_CONNECTION_TIMEOUT_SECS: u64 = 5;
// Default read/write timeout in seconds
pub static DEFAULT_IO_TIMEOUT_SECS: u64 = 10;

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
    let address = format!("{}:13400", ecu_ip);
    info!(
      "DoipClient: Attempting to connect to {} with timeout {:?}",
      address, DEFAULT_CONNECTION_TIMEOUT_SECS
    );

    match address.parse::<SocketAddr>() {
      Ok(socket_addr) => {
        match TcpStream::connect_timeout(
          &socket_addr,
          Duration::from_secs(DEFAULT_CONNECTION_TIMEOUT_SECS),
        ) {
          Ok(stream) => {
            info!("DoipClient: Successfully connected to {}", address);
            // Set read and write timeouts for the stream
            if let Err(e) =
              stream.set_read_timeout(Some(Duration::from_secs(DEFAULT_IO_TIMEOUT_SECS)))
            {
              warn!("DoipClient: Failed to set read timeout: {}", e);
            }
            if let Err(e) =
              stream.set_write_timeout(Some(Duration::from_secs(DEFAULT_IO_TIMEOUT_SECS)))
            {
              warn!("DoipClient: Failed to set write timeout: {}", e);
            }

            Self {
              stream: Some(stream),
              connected: true,
            }
          }
          Err(e) => {
            error!("DoipClient: Failed to connect to {}: {}", address, e);
            Self {
              stream: None,
              connected: false,
            }
          }
        }
      }
      Err(e) => {
        error!("DoipClient: Invalid address format {}: {}", address, e);
        Self {
          stream: None,
          connected: false,
        }
      }
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

      info!("DoipClient: sent raw data: {:x?}", message);

      let mut header = [0u8; 8]; // DoIP general header is 8 bytes
      stream.read_exact(&mut header)?; // read header fully

      // Parse header: protocol version, inverse version, payload type, payload length
      let payload_len = u32::from_be_bytes([header[4], header[5], header[6], header[7]]);
      let mut payload = vec![0u8; payload_len as usize];
      stream.read_exact(&mut payload)?; // read the payload fully

      let mut raw = Vec::with_capacity(8 + payload.len());
      raw.extend_from_slice(&header);
      raw.extend_from_slice(&payload);
      info!("DoipClient: received raw data: {:x?}", raw);

      Ok(payload) // return the response payload
    } else {
      Err(std::io::Error::new(
        std::io::ErrorKind::NotConnected,
        "Not connected to ECU",
      ))
    }
  }
}
