mod common;
mod demo;
mod doip2http;
mod doip_client;
mod uds_client;

use std::env;

static DEFAULT_PORT: u16 = 8080;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Get port from environment variable or use default
  let port = env::var("PORT")
    .unwrap_or_else(|_| DEFAULT_PORT.to_string())
    .parse::<u16>()
    .unwrap_or(DEFAULT_PORT);

  // Start the HTTP server
  doip2http::run_server(port).await?;

  Ok(())
}
