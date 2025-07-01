use crate::doip_client::DEFAULT_ACTIVATION_RESERVED;
use crate::doip_client::DEFAULT_ACTIVATION_TYPE;
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

pub struct UdsClient {
  doip_client: Option<DoipClient>,
  source_address: u16,
}

impl UdsClient {
  pub fn new(ecu_ip: String, source_address: u16) -> Self {
    let mut doip_client = DoipClient::new(ecu_ip);
    if doip_client.is_connected() {
      let mut uds_msg = source_address.to_be_bytes().to_vec();
      uds_msg.push(DEFAULT_ACTIVATION_TYPE);
      uds_msg.extend_from_slice(&DEFAULT_ACTIVATION_RESERVED.to_be_bytes());

      match doip_client.send_and_receive(
        VehicleConnectionPayloadType::RoutingActivationRequest as u16,
        Some(&uds_msg),
      ) {
        Ok(response) => {
          println!("Routing activation response: {:x?}", response);
        }
        Err(e) => {
          println!("Error: {}", e);
          return Self {
            doip_client: None,
            source_address: source_address,
          };
        }
      }
    }
    Self {
      doip_client: Some(doip_client),
      source_address: source_address,
    }
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
