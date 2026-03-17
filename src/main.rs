mod a2a;
mod json_rpc;
mod mcp;
mod transport;
mod worker_manager;

use std::sync::Arc;

use clap::Parser;
use iii_sdk::{InitOptions, register_worker};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[derive(Parser, Debug)]
#[command(name = "iii-connect")]
#[command(version)]
#[command(about = "MCP and A2A protocol worker for iii-engine")]
#[command(long_about = r#"
iii-connect is an iii worker that registers protocol handlers as iii functions.

Every MCP and A2A request flows through the engine — full observability,
any language can extend it, all functions are protocol-consumable.

Modes:
  iii-connect                  MCP stdio (Claude Desktop, Cursor)
  iii-connect --a2a            MCP stdio + A2A HTTP endpoints
  iii-connect --a2a --no-stdio A2A HTTP only (headless, stays alive)
"#)]
struct Args {
    #[arg(long, short = 'e', default_value = "ws://localhost:49134")]
    engine_url: String,

    #[arg(long, short = 'd')]
    debug: bool,

    #[arg(long, help = "Register A2A endpoints")]
    a2a: bool,

    #[arg(long, help = "Skip stdio transport (for headless/HTTP-only mode)")]
    no_stdio: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let filter = if args.debug {
        EnvFilter::new("iii_connect=debug,iii_sdk=debug")
    } else {
        EnvFilter::new("iii_connect=info,iii_sdk=warn")
    };

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(filter)
        .init();

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        engine_url = %args.engine_url,
        a2a = args.a2a,
        no_stdio = args.no_stdio,
        "Starting iii-connect"
    );

    let iii = register_worker(&args.engine_url, InitOptions::default());

    mcp::McpHandler::register(&iii);

    if args.a2a {
        a2a::A2AHandler::register(&iii);
    }

    if args.no_stdio {
        tracing::info!(
            "Running headless (no stdio). Functions registered on engine. Ctrl+C to stop."
        );
        tokio::signal::ctrl_c().await?;
    } else {
        let handler = Arc::new(mcp::McpHandler::new(iii, args.engine_url));
        transport::StdioTransport::run(handler).await?;
    }

    tracing::info!("iii-connect stopped");
    Ok(())
}
