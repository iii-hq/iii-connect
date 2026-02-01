// Copyright Motia LLC and/or licensed to Motia LLC under one or more
// contributor license agreements. Licensed under the Elastic License 2.0;
// you may not use this file except in compliance with the Elastic License 2.0.
// This software is patent protected. We welcome discussions - reach out at support@motia.dev
// See LICENSE and PATENTS files for details.

//! iii-mcp: MCP (Model Context Protocol) server for iii-engine
//!
//! This standalone binary connects to iii-engine and exposes its capabilities
//! through the MCP protocol, allowing AI assistants like Claude and Cursor
//! to interact with iii-engine functions, state, events, and more.

mod handlers;
mod json_rpc;
mod server;
mod transport;

use std::sync::Arc;

use clap::Parser;
use iii_sdk::Bridge;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use server::McpServer;
use transport::StdioTransport;

#[derive(Parser, Debug)]
#[command(name = "iii-mcp")]
#[command(version)]
#[command(about = "MCP (Model Context Protocol) server for iii-engine")]
#[command(long_about = r#"
iii-mcp connects to an iii-engine instance and exposes its capabilities
through the Model Context Protocol (MCP).

This allows AI assistants like Claude Desktop and Cursor to:
- Invoke iii-engine functions
- Manage state
- Emit events
- List workers, functions, and triggers

Usage with Claude Desktop:
  Add to claude_desktop_config.json:
  {
    "mcpServers": {
      "iii": {
        "command": "iii-mcp",
        "args": ["--engine-url", "ws://localhost:8080"]
      }
    }
  }
"#)]
struct Args {
    #[arg(long, short = 'e', default_value = "ws://localhost:8080")]
    engine_url: String,

    #[arg(long, short = 'd')]
    debug: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let filter = if args.debug {
        EnvFilter::new("iii_mcp=debug,iii_sdk=debug")
    } else {
        EnvFilter::new("iii_mcp=info,iii_sdk=warn")
    };

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(filter)
        .init();

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        engine_url = %args.engine_url,
        "Starting iii-mcp server"
    );

    let bridge = Bridge::new(&args.engine_url);

    tracing::info!("Connecting to iii-engine at {}", args.engine_url);
    bridge.connect().await?;
    tracing::info!("Connected to iii-engine");

    let server = Arc::new(McpServer::new(bridge));

    StdioTransport::run(server).await?;

    tracing::info!("iii-mcp server stopped");
    Ok(())
}
