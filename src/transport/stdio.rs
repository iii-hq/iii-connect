// Copyright Motia LLC and/or licensed to Motia LLC under one or more
// contributor license agreements. Licensed under the Elastic License 2.0;
// you may not use this file except in compliance with the Elastic License 2.0.
// This software is patent protected. We welcome discussions - reach out at support@motia.dev
// See LICENSE and PATENTS files for details.

use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};

use crate::json_rpc::{JsonRpcRequest, JsonRpcResponse, PARSE_ERROR};
use crate::server::McpServer;

pub struct StdioTransport;

impl StdioTransport {
    pub async fn run(server: Arc<McpServer>) -> anyhow::Result<()> {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();

        let mut reader = BufReader::new(stdin);
        let mut writer = BufWriter::new(stdout);

        tracing::info!("MCP server started on stdio");

        loop {
            let mut line = String::new();

            match reader.read_line(&mut line).await {
                Ok(0) => {
                    tracing::info!("stdin closed, shutting down");
                    break;
                }
                Ok(_) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    let response = match serde_json::from_str::<JsonRpcRequest>(line) {
                        Ok(request) => {
                            let is_notification = request.id.is_none();

                            let response = server.handle_request(request).await;

                            if is_notification {
                                continue;
                            }

                            response
                        }
                        Err(err) => {
                            tracing::warn!(error = %err, "Failed to parse JSON-RPC request");
                            JsonRpcResponse::error(
                                None,
                                PARSE_ERROR,
                                format!("Parse error: {}", err),
                            )
                        }
                    };

                    match serde_json::to_string(&response) {
                        Ok(json) => {
                            if let Err(err) = writer.write_all(json.as_bytes()).await {
                                tracing::error!(error = %err, "Failed to write response");
                                break;
                            }
                            if let Err(err) = writer.write_all(b"\n").await {
                                tracing::error!(error = %err, "Failed to write newline");
                                break;
                            }
                            if let Err(err) = writer.flush().await {
                                tracing::error!(error = %err, "Failed to flush stdout");
                                break;
                            }
                        }
                        Err(err) => {
                            tracing::error!(error = %err, "Failed to serialize response");
                        }
                    }
                }
                Err(err) => {
                    tracing::error!(error = %err, "Failed to read from stdin");
                    break;
                }
            }
        }

        Ok(())
    }
}
