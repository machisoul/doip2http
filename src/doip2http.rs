use axum::{Router, extract::State, http::StatusCode, response::Json, routing::post};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

use crate::common::log::init_logger;
use crate::common::unity::parse_hex_string_to_bytes;
use crate::uds_client::UdsClient;

// Shared application state
#[derive(Clone)]
pub struct AppState {
  pub connections: Arc<Mutex<HashMap<String, UdsClient>>>,
}

impl AppState {
  pub fn new() -> Self {
    Self {
      connections: Arc::new(Mutex::new(HashMap::new())),
    }
  }
}

// Request/Response structures
#[derive(Deserialize)]
pub struct ConnectRequest {
  pub ecu_ip: String,
  pub doip_source_address: String,
}

#[derive(Deserialize)]
pub struct DiagnosticRequest {
  pub ecu_ip: String,
  pub doip_source_address: String,
  pub doip_target_address: String,
  pub uds_data: String,
}

#[derive(Serialize)]
pub struct ConnectResponse {
  pub success: bool,
  pub message: String,
  pub connection_id: Option<String>,
}

#[derive(Serialize)]
pub struct DiagnosticResponse {
  pub success: bool,
  pub message: String,
  pub response_data: Option<String>,
}

#[derive(Serialize)]
pub struct StatusResponse {
  pub active_connections: usize,
  pub connections: Vec<ConnectionInfo>,
}

#[derive(Serialize)]
pub struct ConnectionInfo {
  pub connection_id: String,
  pub ecu_ip: String,
  pub doip_source_address: String,
  pub connected: bool,
}

// HTTP Handlers

// POST /status - Get connection status
pub async fn get_status(
  State(state): State<AppState>,
  Json(request): Json<ConnectRequest>,
) -> Json<StatusResponse> {
  info!(
    "Get status request: ECU={}, Source={}",
    request.ecu_ip, request.doip_source_address
  );

  let connections = state.connections.lock().unwrap();
  let connection_key = format!("{}:{}", request.ecu_ip, request.doip_source_address);

  if let Some(client) = connections.get(&connection_key) {
    Json(StatusResponse {
      active_connections: 1,
      connections: vec![ConnectionInfo {
        connection_id: connection_key,
        ecu_ip: request.ecu_ip,
        doip_source_address: request.doip_source_address,
        connected: client.is_connected(),
      }],
    })
  } else {
    Json(StatusResponse {
      active_connections: 0,
      connections: vec![],
    })
  }
}

// POST /connect - Establish DoIP connection
pub async fn connect(
  State(state): State<AppState>,
  Json(request): Json<ConnectRequest>,
) -> (StatusCode, Json<ConnectResponse>) {
  info!(
    "Connect request: ECU={}, Source={}",
    request.ecu_ip, request.doip_source_address
  );

  let connection_id = format!("{}:{}", request.ecu_ip, request.doip_source_address);

  // Check if connection already exists
  {
    let connections = state.connections.lock().unwrap();
    if connections.contains_key(&connection_id) {
      return (
        StatusCode::CONFLICT,
        Json(ConnectResponse {
          success: false,
          message: "Connection already exists".to_string(),
          connection_id: Some(connection_id),
        }),
      );
    }
  }

  // Check request data
  {
    if request.ecu_ip.is_empty() {
      return (
        StatusCode::BAD_REQUEST,
        Json(ConnectResponse {
          success: false,
          message: "ECU IP address is required".to_string(),
          connection_id: None,
        }),
      );
    }

    if !request.doip_source_address.starts_with("0x") {
      return (
        StatusCode::BAD_REQUEST,
        Json(ConnectResponse {
          success: false,
          message: "Source address must start with '0x'".to_string(),
          connection_id: None,
        }),
      );
    }
  }

  // Try parsing the hex number after "0x"
  let address_str = &request.doip_source_address[2..];
  let parsed = u16::from_str_radix(address_str, 16);

  match parsed {
    Ok(num) if num <= 0xFFFF => {
      // valid: num is the parsed u16 source address
      // continue with your logic here…
    }
    Ok(_) => {
      // number > u16::MAX, but technically unreachable since u16::from_str_radix never returns Ok with out-of-range value
      return (
        StatusCode::BAD_REQUEST,
        Json(ConnectResponse {
          success: false,
          message: "Source address must be a 2-byte hex value (0x0000 - 0xFFFF)".to_string(),
          connection_id: None,
        }),
      );
    }
    Err(_) => {
      return (
        StatusCode::BAD_REQUEST,
        Json(ConnectResponse {
          success: false,
          message: "Source address must be a valid hexadecimal number".to_string(),
          connection_id: None,
        }),
      );
    }
  }

  // Create new UDS client
  let uds_client = UdsClient::new(request.ecu_ip.clone(), parsed.unwrap());

  if uds_client.is_connected() {
    // Store the connection
    let mut connections = state.connections.lock().unwrap();
    connections.insert(connection_id.clone(), uds_client);

    (
      StatusCode::OK,
      Json(ConnectResponse {
        success: true,
        message: "Successfully connected to ECU".to_string(),
        connection_id: Some(connection_id),
      }),
    )
  } else {
    (
      StatusCode::BAD_REQUEST,
      Json(ConnectResponse {
        success: false,
        message: "Failed to connect to ECU".to_string(),
        connection_id: None,
      }),
    )
  }
}

// POST /diagnostic - Send UDS diagnostic message
pub async fn diagnostic_handler(
  State(state): State<AppState>,
  Json(request): Json<DiagnosticRequest>,
) -> (StatusCode, Json<DiagnosticResponse>) {
  info!(
    "Diagnostic request: ECU={}, Source={}, Target={}, UDS data={}",
    request.ecu_ip, request.doip_source_address, request.doip_target_address, request.uds_data
  );

  // Check request data
  {
    if request.ecu_ip.is_empty() {
      return (
        StatusCode::BAD_REQUEST,
        Json(DiagnosticResponse {
          success: false,
          message: "ECU IP address is required".to_string(),
          response_data: None,
        }),
      );
    }

    if !request.doip_source_address.starts_with("0x") {
      return (
        StatusCode::BAD_REQUEST,
        Json(DiagnosticResponse {
          success: false,
          message: "Source address must start with '0x'".to_string(),
          response_data: None,
        }),
      );
    }

    if !request.doip_target_address.starts_with("0x") {
      return (
        StatusCode::BAD_REQUEST,
        Json(DiagnosticResponse {
          success: false,
          message: "Target address must start with '0x'".to_string(),
          response_data: None,
        }),
      );
    }

    if !request.uds_data.starts_with("0x") {
      return (
        StatusCode::BAD_REQUEST,
        Json(DiagnosticResponse {
          success: false,
          message: "UDS data must start with '0x'".to_string(),
          response_data: None,
        }),
      );
    }
  }

  //Parse address and UDS data
  let source_address_str = &request.doip_source_address[2..];
  let target_address_str = &request.doip_target_address[2..];

  let source_address = u16::from_str_radix(source_address_str, 16).unwrap();
  let target_address = u16::from_str_radix(target_address_str, 16).unwrap();

  let mut uds_data_with_address = Vec::new();
  uds_data_with_address.extend_from_slice(&source_address.to_be_bytes());
  uds_data_with_address.extend_from_slice(&target_address.to_be_bytes());
  uds_data_with_address.extend_from_slice(&parse_hex_string_to_bytes(&request.uds_data).unwrap());

  // Check if connection exists for this ECU and source address
  let connection_key = format!("{}:{}", request.ecu_ip, request.doip_source_address);

  let mut connections = state.connections.lock().unwrap();
  if let Some(uds_client) = connections.get_mut(&connection_key) {
    if !uds_client.is_connected() {
      return (
        StatusCode::BAD_REQUEST,
        Json(DiagnosticResponse {
          success: false,
          message: "Not connected to ECU".to_string(),
          response_data: None,
        }),
      );
    }
    match uds_client.doip(Some(&uds_data_with_address)) {
      Ok(response) => (
        StatusCode::OK,
        Json(DiagnosticResponse {
          success: true,
          message: "Successfully sent diagnostic message".to_string(),
          response_data: Some(format!("0x{:X?}", response)),
        }),
      ),
      Err(e) => (
        StatusCode::BAD_REQUEST,
        Json(DiagnosticResponse {
          success: false,
          message: format!("Failed to send diagnostic message: {}", e),
          response_data: None,
        }),
      ),
    }
  } else {
    (
      StatusCode::BAD_REQUEST,
      Json(DiagnosticResponse {
        success: false,
        message: "Connection not found".to_string(),
        response_data: None,
      }),
    )
  }
}

// Create the router
pub fn create_router() -> Router {
  let state = AppState::new();

  Router::new()
    .route("/status", post(get_status))
    .route("/connect", post(connect))
    .route("/diagnostic", post(diagnostic_handler))
    .with_state(state)
}

// Main server function
pub async fn run_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
  init_logger();

  let app = create_router();
  let addr = format!("0.0.0.0:{}", port);

  info!("DoIP2HTTP server starting on {}", addr);
  info!("Available endpoints:");
  info!("  GET  /status     - Get connection status");
  info!("  POST /connect    - Connect to ECU (ecu_ip, source_address)");
  info!("  POST /diagnostic - Send diagnostic message (target_address, uds_data)");

  let listener = TcpListener::bind(&addr).await?;
  axum::serve(listener, app).await?;

  Ok(())
}
