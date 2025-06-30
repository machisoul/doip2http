use std::net::TcpStream;

static PROTO_VERSION:u8 = 0x02;
static INVER_PROTO_VERSION:u8 = 0x01;

#[repr(u16)]
enum VehicleConnectionPayloadType {
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
enum DiagnosticPayloadType {
    DiagnosticMessage = 0x8001,
    DiagnosticPositiveAck = 0x8002,
    DiagnosticNegativeAck = 0x8003,
}

pub strcut DoipClient {
  stream: Opetion<TcpStream>,
  connected: bool,
}

impl DoipClient{
  pub fn new(ecu_ip: String) -> Self {
    let stream = TcpStream::connect(format!("{}:{}", ecu_ip, 13400)).unwrap();

    if stream.is_connected() {
      Self { stream: Some(stream), connected: true }
    } else {
      Self { stream: None, connected: false }
    }
  }

  pub fn is_connected(&self) -> bool {
    self.connected
  }

  pub fn send(&self, uds_data: &[u8]) -> Result<usize, std::io::Error> {
    self.stream.as_ref().unwrap().write(uds_data);
    timeout(Duration::from_secs(1), self.stream.as_ref().unwrap().read(&mut buffer)).unwrap();
  }

  pub fn close(&mut self) {
    self.stream.as_ref().unwrap().close();
    self.connected = false;
  }
}