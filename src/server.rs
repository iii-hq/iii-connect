// Copyright Motia LLC and/or licensed to Motia LLC under one or more
// contributor license agreements. Licensed under the Elastic License 2.0;
// you may not use this file except in compliance with the Elastic License 2.0.
// This software is patent protected. We welcome discussions - reach out at support@motia.dev
// See LICENSE and PATENTS files for details.

use std::sync::atomic::{AtomicBool, Ordering};

use iii_sdk::Bridge;
use serde_json::Value;

use crate::handlers::{InitializeHandler, ResourcesHandler, ToolsHandler};
use crate::json_rpc::{INTERNAL_ERROR, JsonRpcRequest, JsonRpcResponse, METHOD_NOT_FOUND};

pub struct McpServer {
    initialize_handler: InitializeHandler,
    tools_handler: ToolsHandler,
    resources_handler: ResourcesHandler,
    initialized: AtomicBool,
}

impl McpServer {
    pub fn new(bridge: Bridge, engine_url: String) -> Self {
        Self {
            initialize_handler: InitializeHandler::new("iii-mcp", env!("CARGO_PKG_VERSION")),
            tools_handler: ToolsHandler::new(bridge.clone(), engine_url),
            resources_handler: ResourcesHandler::new(bridge),
            initialized: AtomicBool::new(false),
        }
    }

    pub async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id.clone();
        let method = request.method.as_str();

        tracing::debug!(method = %method, "Handling MCP request");

        let result = match method {
            "initialize" => {
                self.initialized.store(true, Ordering::SeqCst);
                Ok(self.initialize_handler.handle(request.params))
            }
            "notifications/initialized" => {
                return JsonRpcResponse::success(id, Value::Null);
            }

            "tools/list" => {
                if !self.initialized.load(Ordering::SeqCst) {
                    return JsonRpcResponse::error(id, INTERNAL_ERROR, "Server not initialized");
                }
                Ok(self.tools_handler.list().await)
            }
            "tools/call" => {
                if !self.initialized.load(Ordering::SeqCst) {
                    return JsonRpcResponse::error(id, INTERNAL_ERROR, "Server not initialized");
                }
                Ok(self.tools_handler.call(request.params).await)
            }

            "resources/list" => {
                if !self.initialized.load(Ordering::SeqCst) {
                    return JsonRpcResponse::error(id, INTERNAL_ERROR, "Server not initialized");
                }
                Ok(self.resources_handler.list().await)
            }
            "resources/read" => {
                if !self.initialized.load(Ordering::SeqCst) {
                    return JsonRpcResponse::error(id, INTERNAL_ERROR, "Server not initialized");
                }
                Ok(self.resources_handler.read(request.params).await)
            }
            "resources/templates/list" => Ok(serde_json::json!({ "resourceTemplates": [] })),

            "ping" => Ok(Value::Object(serde_json::Map::new())),

            _ => {
                tracing::warn!(method = %method, "Unknown MCP method");
                Err(format!("Unknown method: {}", method))
            }
        };

        match result {
            Ok(value) => JsonRpcResponse::success(id, value),
            Err(err) => JsonRpcResponse::error(id, METHOD_NOT_FOUND, err),
        }
    }
}
