use crate::doip_client::DiagnosticPayloadType;
use crate::doip_client::DoipClient;
use crate::doip_client::VehicleConnectionPayloadType;
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::io::{Error, ErrorKind};
use tokio::io::AsyncWriteExt;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UdsServiceType {
  DiagnosticSessionControl = 0x10,   // 0x10
  ECUReset = 0x11,                   // 0x11
  ClearDiagnosticInformation = 0x14, // 0x14
  ReadDTCInformation = 0x19,         // 0x19
  SecurityAccess = 0x27,             // 0x27
  CommunicationControl = 0x28,       // 0x28
  TesterPresent = 0x3E,              // 0x3E
  ControlDTCSetting = 0x85,          // 0x85
  ResponseOnEvent = 0x86,            // 0x86
  LinkControl = 0x87,                // 0x87
  ReadDataByIdentifier = 0x22,       // 0x22
  WriteDataByIdentifier = 0x2E,      // 0x2E
  ReadMemoryByAddress = 0x23,        // 0x23
  WriteMemoryByAddress = 0x3D,       // 0x3D
  RoutineControl = 0x31,             // 0x31
  RequestDownload = 0x34,            // 0x34
  RequestUpload = 0x35,              // 0x35
  TransferData = 0x36,               // 0x36
  RequestTransferExit = 0x37,        // 0x37
  AccessTimingParameter = 0x83,      // 0x83
  SecuredDataTransmission = 0x84,    // 0x84
}

pub static UDS_SERVICE_SET: Lazy<HashSet<u8>> = Lazy::new(|| {
  use UdsServiceType::*;
  HashSet::from([
    DiagnosticSessionControl as u8,
    ECUReset as u8,
    ClearDiagnosticInformation as u8,
    ReadDTCInformation as u8,
    SecurityAccess as u8,
    CommunicationControl as u8,
    TesterPresent as u8,
    ControlDTCSetting as u8,
    ResponseOnEvent as u8,
    LinkControl as u8,
    ReadDataByIdentifier as u8,
    WriteDataByIdentifier as u8,
    ReadMemoryByAddress as u8,
    WriteMemoryByAddress as u8,
    RoutineControl as u8,
    RequestDownload as u8,
    RequestUpload as u8,
    TransferData as u8,
    RequestTransferExit as u8,
    AccessTimingParameter as u8,
    SecuredDataTransmission as u8,
  ])
});

fn is_service_defined(service_set: &HashSet<u8>, service_id: u8) -> bool {
  service_set.contains(&service_id)
}

pub struct UdsClient {
  doip_client: Option<DoipClient>,
  source_address: u16,
  target_address: Vec<u16>,
}

impl UdsClient {
  pub fn new(ecu_ip: String, source_address: u16) -> Self {
    Self {
      doip_client: Some(DoipClient::new(ecu_ip)),
      source_address: source_address,
      target_address: Vec::new(),
    }
  }

  pub async fn routing_active(&mut self, target_address: u16) -> Result<(), Error> {
    // Check if DoIP client exists and connected
    let doip_client = match self.doip_client.as_mut() {
      Some(client) if client.is_connected() => client,
      _ => {
        return Err(Error::new(ErrorKind::NotConnected, "Not connected to ECU"));
      }
    };

    // Build routing activation payload according to DoIP spec
    // Logical Address: 2 bytes (big endian)
    // Routing Activation Type: 1 byte (0x00 default)
    // Reserved: 1 byte (0x00)
    let mut payload = Vec::with_capacity(4);
    payload.push((target_address >> 8) as u8); // high byte
    payload.push((target_address & 0xFF) as u8); // low byte
    payload.push(0x00); // Routing Activation Type: default
    payload.push(0x00); // Reserved

    // Send routing activation message and wait for response
    // Assuming DoipClient has a send_and_receive method that returns Result<Vec<u8>, Error>
    let response = doip_client
      .send_and_receive(
        VehicleConnectionPayloadType::RoutingActivationRequest as u16,
        Some(&payload),
      )
      .await?;

    // Check the response to confirm successful activation
    // According to DoIP, successful response payload starts with 0x10 (Routing Activation Response type)
    // and has a result byte (0x00 means success)
    // Here, we do a minimal check assuming response is well-formed

    if response.len() < 5 {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "Invalid routing activation response",
      ));
    }

    // response[0..2] = protocol version & inverse protocol version (usually 0x02 0x01)
    // response[2..4] = payload type (should be RoutingActivationResponse)
    // response[4] = routing activation result (0x00 means success)

    const ROUTING_ACTIVATION_RESPONSE_PAYLOAD_TYPE: u16 =
      VehicleConnectionPayloadType::RoutingActivationResponse as u16;

    let payload_type = ((response[2] as u16) << 8) | (response[3] as u16);
    let routing_result = response[4];

    if payload_type != ROUTING_ACTIVATION_RESPONSE_PAYLOAD_TYPE {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "Unexpected payload type in routing activation response",
      ));
    }

    if routing_result != 0x00 {
      return Err(Error::new(
        ErrorKind::Other,
        format!(
          "Routing activation failed with code: 0x{:02X}",
          routing_result
        ),
      ));
    }

    // Append the successfully activated address if not already present
    if !self.target_address.contains(&target_address) {
      self.target_address.push(target_address);
    }

    Ok(())
  }

  pub async fn doip(&mut self, uds_data: Option<&[u8]>) -> Result<Vec<u8>, Error> {
    if !self.doip_client.as_ref().unwrap().is_connected() {
      return Err(Error::new(ErrorKind::NotConnected, "Not connected to ECU"));
    }

    let uds =
      uds_data.ok_or_else(|| Error::new(ErrorKind::InvalidInput, "No UDS data provided"))?;

    if let Some(&service_id) = uds.first() {
      if !UDS_SERVICE_SET.contains(&service_id) {
        return Err(Error::new(
          ErrorKind::InvalidData,
          format!("Unknown UDS service ID: 0x{:02X}", service_id),
        ));
      }
    } else {
      return Err(Error::new(
        ErrorKind::InvalidInput,
        "UDS data is empty; cannot read first byte",
      ));
    }

    if let Some(doip_client) = self.doip_client.as_mut() {
      doip_client.send_and_receive(DiagnosticPayloadType::DiagnosticMessage as u16, Some(uds))
    } else {
      Err(Error::new(ErrorKind::NotConnected, "Not connected to ECU"))
    }
  }
}
