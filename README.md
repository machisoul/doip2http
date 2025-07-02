# DoIP2HTTP

A Rust project that bridges DoIP (Diagnostics over Internet Protocol) connections to HTTP requests. This project implements DoIP client functionality for automotive diagnostic communication with ECUs and provides HTTP API interfaces.

## Project Overview

DoIP2HTTP is a diagnostic tool written in Rust with the following key features:

- **DoIP Client Implementation**: Complete DoIP protocol implementation including vehicle connection and routing activation
- **UDS Service Support**: Full implementation of UDS (Unified Diagnostic Services) service set
- **HTTP API Bridge**: Exposes DoIP diagnostic functionality through HTTP interfaces for easy integration
- **Asynchronous Processing**: Built on Tokio async runtime for high-concurrency handling

### Core Features

#### DoIP Protocol Support
- Vehicle Identification Request
- Routing Activation Request/Response
- Diagnostic Message transmission
- Alive Check functionality

#### UDS Service Support
Supports the following UDS services:
- Diagnostic Session Control (0x10)
- ECU Reset (0x11)
- Clear Diagnostic Information (0x14)
- Read DTC Information (0x19)
- Security Access (0x27)
- Communication Control (0x28)
- Tester Present (0x3E)
- Data Read/Write Services (0x22, 0x2E)
- Routine Control (0x31)
- Data Transfer Services (0x34-0x37)
- And many more...

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

### 1. Basic DoIP Testing

Run the main program for DoIP connection testing:

```bash
# Run debug version
cargo run --bin doip2http

# Or run release version
./target/release/doip2http
```

The program will prompt for:
- ECU IP address
- Client source address (hexadecimal format, e.g., 0x1234)

### 2. HTTP Server Demo

Run the HTTP server demonstration:

```bash
cargo run --bin axum-demo
```

The server will start at `http://0.0.0.0:3000` with the following endpoints:
- `GET /` - Returns "Hello, World!"
- `POST /users` - Demo endpoint for user creation

### 3. Environment Configuration

Configure logging level through environment variables:

```bash
# Set log level
export DOIP2HTTP_LOG_LEVEL=debug

# Set log style
export DOIP2HTTP_LOG_STYLE=always
```

Supported log levels: `error`, `warn`, `info`, `debug`, `trace`

## API Usage Examples

### DoIP Client Usage

```rust
use doip2http::uds_client::UdsClient;

// Create UDS client
let mut uds_client = UdsClient::new("192.168.1.100".to_string(), 0x1234);

// Send UDS diagnostic message
let uds_data = vec![0x22, 0xF1, 0x90]; // Read VIN
match uds_client.doip(Some(&uds_data)).await {
    Ok(response) => {
        println!("Diagnostic response: {:02X?}", response);
    }
    Err(e) => {
        eprintln!("Diagnostic error: {}", e);
    }
}
```

### HTTP API Integration

The project includes an Axum framework HTTP server implementation that can be extended into a complete DoIP-HTTP bridge service.

## Project Structure

```
doip2http/
├── src/
│   ├── main.rs              # Main program entry
│   ├── doip_client.rs       # DoIP client implementation
│   ├── uds_client.rs        # UDS client wrapper
│   ├── common/
│   │   ├── mod.rs
│   │   └── log.rs           # Logging configuration
│   └── demo/
│       ├── mod.rs
│       ├── axum.rs          # HTTP server demo
│       └── console_input.rs # Console input utilities
├── Cargo.toml               # Project configuration
└── README.md               # Project documentation
```