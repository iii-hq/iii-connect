// Copyright Motia LLC and/or licensed to Motia LLC under one or more
// contributor license agreements. Licensed under the Elastic License 2.0;
// you may not use this file except in compliance with the Elastic License 2.0.
// This software is patent protected. We welcome discussions - reach out at support@motia.dev
// See LICENSE and PATENTS files for details.

use iii_sdk::{Bridge, FunctionInfo};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpTool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsListResult {
    pub tools: Vec<McpTool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallParams {
    pub name: String,
    #[serde(default)]
    pub arguments: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ToolContent {
    Text { text: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallResult {
    pub content: Vec<ToolContent>,
    #[serde(default)]
    pub is_error: bool,
}

pub struct ToolsHandler {
    bridge: Bridge,
}

impl ToolsHandler {
    pub fn new(bridge: Bridge) -> Self {
        Self { bridge }
    }

    fn function_to_tool(func: &FunctionInfo) -> McpTool {
        // Convert function path to tool name (dots to underscores)
        let name = func.function_path.replace('.', "_");

        let input_schema = func.request_format.clone().unwrap_or_else(|| {
            json!({
                "type": "object",
                "properties": {}
            })
        });

        McpTool {
            name,
            description: func.description.clone(),
            input_schema,
        }
    }

    pub async fn list(&self) -> Value {
        match self.bridge.list_functions().await {
            Ok(functions) => {
                let tools: Vec<McpTool> = functions.iter().map(Self::function_to_tool).collect();

                json!(ToolsListResult { tools })
            }
            Err(err) => {
                tracing::error!(error = %err, "Failed to list functions");
                json!(ToolsListResult { tools: vec![] })
            }
        }
    }

    pub async fn call(&self, params: Option<Value>) -> Value {
        let params: ToolCallParams = match params {
            Some(p) => match serde_json::from_value(p) {
                Ok(p) => p,
                Err(err) => {
                    return json!(ToolCallResult {
                        content: vec![ToolContent::Text {
                            text: format!("Invalid params: {}", err)
                        }],
                        is_error: true,
                    });
                }
            },
            None => {
                return json!(ToolCallResult {
                    content: vec![ToolContent::Text {
                        text: "Missing params".to_string()
                    }],
                    is_error: true,
                });
            }
        };

        let function_path = params.name.replace('_', ".");

        tracing::debug!(
            tool_name = %params.name,
            function_path = %function_path,
            "Invoking function"
        );

        match self
            .bridge
            .invoke_function(&function_path, params.arguments)
            .await
        {
            Ok(result) => {
                let text =
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string());

                json!(ToolCallResult {
                    content: vec![ToolContent::Text { text }],
                    is_error: false,
                })
            }
            Err(err) => {
                tracing::error!(
                    function_path = %function_path,
                    error = %err,
                    "Function invocation failed"
                );

                json!(ToolCallResult {
                    content: vec![ToolContent::Text {
                        text: format!("Error: {}", err)
                    }],
                    is_error: true,
                })
            }
        }
    }
}
