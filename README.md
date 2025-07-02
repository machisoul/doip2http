# DoIP2HTTP

A Rust-based HTTP server that bridges DoIP (Diagnostics over Internet Protocol) connections to HTTP requests. This project provides a complete HTTP API for automotive diagnostic communication with ECUs through DoIP protocol.

## Project Overview

DoIP2HTTP is a production-ready HTTP server written in Rust that enables automotive diagnostics through RESTful APIs:

- **HTTP API Server**: Complete HTTP server with RESTful endpoints for DoIP operations
- **DoIP Client Implementation**: Full DoIP protocol implementation with connection management and routing activation
- **UDS Service Support**: Comprehensive UDS (Unified Diagnostic Services) service validation and processing
- **Connection Management**: Multi-ECU connection handling with persistent connection pooling
- **Asynchronous Processing**: Built on Tokio async runtime for high-performance concurrent operations

### Core Features

#### HTTP API Endpoints
- **POST /status** - Check connection status for specific ECU and source address
- **POST /connect** - Establish DoIP connection to ECU with routing activation
- **POST /diagnostic** - Send UDS diagnostic messages and receive responses

#### DoIP Protocol Support
- TCP connection with configurable timeouts (5s connection, 10s I/O)
- Routing Activation Request/Response handling
- Diagnostic Message transmission with proper framing
- Connection state management and validation

#### UDS Service Support
Validates and supports the following UDS services:
- Diagnostic Session Control (0x10)
- ECU Reset (0x11)
- Clear Diagnostic Information (0x14)
- Read DTC Information (0x19)
- Security Access (0x27)
- Communication Control (0x28)
- Tester Present (0x3E)
- Control DTC Setting (0x85)
- Response On Event (0x86)
- Link Control (0x87)
- Data Read/Write Services (0x22, 0x2E)
- Memory Read/Write Services (0x23, 0x3D)
- Routine Control (0x31)
- Data Transfer Services (0x34-0x37)
- Access Timing Parameter (0x83)
- Secured Data Transmission (0x84)

## System Requirements

- Rust 1.70 or higher
- Supported OS: Linux, macOS, Windows

## Building

### 1. Clone the Repository

```bash
git clone https://github.com/machisoul/doip2http.git
cd doip2http
```

### 2. Build the Project

```bash
# Build debug version
cargo build

# Build release version (recommended for production)
cargo build --release
```

### 3. Run Tests

```bash
cargo test
```

## Usage

### 1. Start the HTTP Server

Run the DoIP2HTTP server:

```bash
# Run debug version
cargo run --bin doip2http

# Run release version
./target/release/doip2http

# Specify custom port via environment variable
PORT=3000 cargo run
```

The server will start on `http://0.0.0.0:8080` (or specified port) with the following endpoints:
- `POST /status` - Get connection status
- `POST /connect` - Connect to ECU
- `POST /diagnostic` - Send diagnostic messages

### 2. Environment Configuration

Configure logging level through environment variables:

```bash
# Set log level (supported: error, warn, info, debug, trace)
export DOIP2HTTP_LOG_LEVEL=debug

# Set log style
export DOIP2HTTP_LOG_STYLE=always

# Run with custom port
export PORT=3000
```

### 3. API Usage Examples

#### Check Connection Status
```bash
curl -X POST http://localhost:8080/status \
  -H "Content-Type: application/json" \
  -d '{
    "ecu_ip": "192.168.1.100",
    "doip_source_address": "0x1234"
  }'
```

#### Connect to ECU
```bash
curl -X POST http://localhost:8080/connect \
  -H "Content-Type: application/json" \
  -d '{
    "ecu_ip": "192.168.1.100",
    "doip_source_address": "0x1234"
  }'
```

#### Send Diagnostic Message
```bash
curl -X POST http://localhost:8080/diagnostic \
  -H "Content-Type: application/json" \
  -d '{
    "ecu_ip": "192.168.1.100",
    "doip_source_address": "0x1234",
    "doip_target_address": "0x5678",
    "uds_data": "22F190"
  }'
```

## API Reference

### Request/Response Formats

#### POST /status
**Request:**
```json
{
  "ecu_ip": "192.168.1.100",
  "doip_source_address": "0x1234"
}
```

**Response:**
```json
{
  "active_connections": 1,
  "connections": [
    {
      "connection_id": "192.168.1.100:0x1234",
      "ecu_ip": "192.168.1.100",
      "doip_source_address": "0x1234",
      "connected": true
    }
  ]
}
```

#### POST /connect
**Request:**
```json
{
  "ecu_ip": "192.168.1.100",
  "doip_source_address": "0x1234"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Successfully connected to ECU",
  "connection_id": "192.168.1.100:0x1234"
}
```

#### POST /diagnostic
**Request:**
```json
{
  "ecu_ip": "192.168.1.100",
  "doip_source_address": "0x1234",
  "doip_target_address": "0x5678",
  "uds_data": "22F190"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Successfully sent diagnostic message",
  "response_data": "0x[62, F1, 90, ...]"
}
```

### Error Handling

The API returns appropriate HTTP status codes:
- `200 OK` - Successful operation
- `400 Bad Request` - Invalid request data or connection issues
- `409 Conflict` - Connection already exists
- `500 Internal Server Error` - Server-side errors

## Project Structure

```
doip2http/
├── src/
│   ├── main.rs              # Main HTTP server entry point
│   ├── doip2http.rs         # HTTP API handlers and server logic
│   ├── doip_client.rs       # DoIP protocol client implementation
│   ├── uds_client.rs        # UDS client with service validation
│   └── common/
│       ├── mod.rs           # Common module exports
│       ├── log.rs           # Logging configuration
│       ├── unity.rs         # Utility functions (hex parsing, etc.)
│       └── console_input.rs # Console input utilities
├── Cargo.toml               # Project configuration and dependencies
├── Cargo.lock               # Dependency lock file
└── README.md               # Project documentation
```

## Technical Details

### Connection Management
- Each ECU connection is identified by `ecu_ip:doip_source_address`
- Connections are pooled and reused across HTTP requests
- Automatic routing activation during connection establishment
- Connection state validation before diagnostic operations

### DoIP Protocol Implementation
- Standard DoIP port 13400
- Protocol version 0x02 with inverse 0xFD
- Configurable timeouts: 5s connection, 10s I/O operations
- Proper message framing with payload type and length headers
- Support for routing activation and diagnostic message payload types

### UDS Service Validation
- Validates UDS service IDs against supported service set
- Proper message formatting with source/target addresses
- Hex string parsing for UDS data input
- Binary response data formatting

### Dependencies
- **axum**: Modern async web framework for HTTP server
- **tokio**: Async runtime for concurrent operations
- **serde**: JSON serialization/deserialization
- **log + env_logger**: Structured logging
- **once_cell**: Lazy static initialization for service sets

## Configuration

### Environment Variables
- `PORT`: HTTP server port (default: 8080)
- `RUST_LOG`: Log level (error, warn, info, debug, trace)

### DoIP Configuration
- **Connection Timeout**: 5 seconds
- **I/O Timeout**: 10 seconds
- **DoIP Port**: 13400 (standard)
- **Protocol Version**: 0x02

## Troubleshooting

### Common Issues

1. **Connection Timeout**
   - Ensure ECU is reachable on port 13400
   - Check network connectivity and firewall settings
   - Verify ECU IP address is correct

2. **Invalid UDS Service**
   - Check that the UDS service ID is supported
   - Ensure proper hex formatting (e.g., "22F190" not "0x22F190")
   - Verify target address matches ECU configuration

3. **Connection Already Exists**
   - Each ECU+source_address combination allows only one connection
   - Use different source addresses for multiple connections to same ECU
   - Check existing connections via `/status` endpoint

### Logging
Enable debug logging to troubleshoot issues:
```bash
RUST_LOG=debug cargo run
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request