// Copyright Motia LLC and/or licensed to Motia LLC under one or more
// contributor license agreements. Licensed under the Elastic License 2.0;
// you may not use this file except in compliance with the Elastic License 2.0.
// This software is patent protected. We welcome discussions - reach out at support@motia.dev
// See LICENSE and PATENTS files for details.

use iii_sdk::{Bridge, FunctionInfo, Trigger};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::worker_manager::{WorkerManager, WorkerCreateParams, WorkerStopParams};

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

#[derive(Debug, Clone, Deserialize)]
pub struct TriggerRegisterParams {
    pub trigger_type: String,
    pub function_path: String,
    pub config: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TriggerUnregisterParams {
    pub id: String,
}

struct StoredTrigger {
    #[allow(dead_code)]
    id: String,
    trigger_type: String,
    function_path: String,
    trigger: Trigger,
}

pub struct ToolsHandler {
    bridge: Bridge,
    worker_manager: Arc<WorkerManager>,
    triggers: Arc<Mutex<HashMap<String, StoredTrigger>>>,
}

impl ToolsHandler {
    pub fn new(bridge: Bridge, engine_url: String) -> Self {
        Self {
            bridge,
            worker_manager: Arc::new(WorkerManager::new(engine_url)),
            triggers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn function_to_tool(func: &FunctionInfo) -> McpTool {
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

    fn builtin_tools() -> Vec<McpTool> {
        vec![
            McpTool {
                name: "iii_worker_create".to_string(),
                description: Some("Create a temporary worker with custom function code".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "language": {
                            "type": "string",
                            "description": "Programming language: 'node' or 'python'",
                            "enum": ["node", "python"]
                        },
                        "code": {
                            "type": "string",
                            "description": "Function code (async handler that receives args and returns result)"
                        },
                        "function_name": {
                            "type": "string",
                            "description": "Function name (e.g., 'myservice.myfunction')"
                        },
                        "description": {
                            "type": "string",
                            "description": "Optional function description"
                        }
                    },
                    "required": ["language", "code", "function_name"]
                }),
            },
            McpTool {
                name: "iii_worker_stop".to_string(),
                description: Some("Stop a spawned worker and clean up".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Worker ID to stop"
                        }
                    },
                    "required": ["id"]
                }),
            },
            McpTool {
                name: "iii_trigger_register".to_string(),
                description: Some("Register a trigger to invoke a function".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "trigger_type": {
                            "type": "string",
                            "description": "Trigger type (e.g., 'cron', 'event', 'http')"
                        },
                        "function_path": {
                            "type": "string",
                            "description": "Function path to invoke when triggered"
                        },
                        "config": {
                            "type": "object",
                            "description": "Trigger-specific configuration (e.g., {schedule: '0 * * * *'} for cron)"
                        }
                    },
                    "required": ["trigger_type", "function_path", "config"]
                }),
            },
            McpTool {
                name: "iii_trigger_unregister".to_string(),
                description: Some("Unregister a previously registered trigger".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Trigger ID to unregister"
                        }
                    },
                    "required": ["id"]
                }),
            },
        ]
    }

    pub async fn list(&self) -> Value {
        let mut tools = Self::builtin_tools();

        match self.bridge.list_functions().await {
            Ok(functions) => {
                let function_tools: Vec<McpTool> = functions.iter().map(Self::function_to_tool).collect();
                tools.extend(function_tools);
            }
            Err(err) => {
                tracing::error!(error = %err, "Failed to list functions");
            }
        }

        json!(ToolsListResult { tools })
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

        match params.name.as_str() {
            "iii_worker_create" => return self.worker_create(params.arguments).await,
            "iii_worker_stop" => return self.worker_stop(params.arguments).await,
            "iii_trigger_register" => return self.trigger_register(params.arguments).await,
            "iii_trigger_unregister" => return self.trigger_unregister(params.arguments).await,
            _ => {}
        }

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

    async fn worker_create(&self, args: Value) -> Value {
        let params: WorkerCreateParams = match serde_json::from_value(args) {
            Ok(p) => p,
            Err(err) => {
                return json!(ToolCallResult {
                    content: vec![ToolContent::Text {
                        text: format!("Invalid params: {}", err)
                    }],
                    is_error: true,
                });
            }
        };

        match self.worker_manager.create_worker(params).await {
            Ok(result) => {
                let text = serde_json::to_string_pretty(&result).unwrap_or_default();
                json!(ToolCallResult {
                    content: vec![ToolContent::Text { text }],
                    is_error: false,
                })
            }
            Err(err) => {
                json!(ToolCallResult {
                    content: vec![ToolContent::Text {
                        text: format!("Error: {}", err)
                    }],
                    is_error: true,
                })
            }
        }
    }

    async fn worker_stop(&self, args: Value) -> Value {
        let params: WorkerStopParams = match serde_json::from_value(args) {
            Ok(p) => p,
            Err(err) => {
                return json!(ToolCallResult {
                    content: vec![ToolContent::Text {
                        text: format!("Invalid params: {}", err)
                    }],
                    is_error: true,
                });
            }
        };

        match self.worker_manager.stop_worker(params).await {
            Ok(result) => {
                let text = serde_json::to_string_pretty(&result).unwrap_or_default();
                json!(ToolCallResult {
                    content: vec![ToolContent::Text { text }],
                    is_error: false,
                })
            }
            Err(err) => {
                json!(ToolCallResult {
                    content: vec![ToolContent::Text {
                        text: format!("Error: {}", err)
                    }],
                    is_error: true,
                })
            }
        }
    }

    async fn trigger_register(&self, args: Value) -> Value {
        let params: TriggerRegisterParams = match serde_json::from_value(args) {
            Ok(p) => p,
            Err(err) => {
                return json!(ToolCallResult {
                    content: vec![ToolContent::Text {
                        text: format!("Invalid params: {}", err)
                    }],
                    is_error: true,
                });
            }
        };

        tracing::debug!(
            trigger_type = %params.trigger_type,
            function_path = %params.function_path,
            "Registering trigger"
        );

        match self.bridge.register_trigger(
            &params.trigger_type,
            &params.function_path,
            &params.config,
        ) {
            Ok(trigger) => {
                let id = uuid::Uuid::new_v4().to_string();
                
                let stored = StoredTrigger {
                    id: id.clone(),
                    trigger_type: params.trigger_type.clone(),
                    function_path: params.function_path.clone(),
                    trigger,
                };

                self.triggers.lock().await.insert(id.clone(), stored);

                let result = json!({
                    "id": id,
                    "trigger_type": params.trigger_type,
                    "function_path": params.function_path,
                    "message": "Trigger registered successfully"
                });

                json!(ToolCallResult {
                    content: vec![ToolContent::Text {
                        text: serde_json::to_string_pretty(&result).unwrap_or_default()
                    }],
                    is_error: false,
                })
            }
            Err(err) => {
                json!(ToolCallResult {
                    content: vec![ToolContent::Text {
                        text: format!("Error: {}", err)
                    }],
                    is_error: true,
                })
            }
        }
    }

    async fn trigger_unregister(&self, args: Value) -> Value {
        let params: TriggerUnregisterParams = match serde_json::from_value(args) {
            Ok(p) => p,
            Err(err) => {
                return json!(ToolCallResult {
                    content: vec![ToolContent::Text {
                        text: format!("Invalid params: {}", err)
                    }],
                    is_error: true,
                });
            }
        };

        let mut triggers = self.triggers.lock().await;
        
        if let Some(stored) = triggers.remove(&params.id) {
            stored.trigger.unregister();

            tracing::info!(
                trigger_id = %params.id,
                trigger_type = %stored.trigger_type,
                function_path = %stored.function_path,
                "Unregistered trigger"
            );

            let result = json!({
                "id": params.id,
                "message": "Trigger unregistered successfully"
            });

            json!(ToolCallResult {
                content: vec![ToolContent::Text {
                    text: serde_json::to_string_pretty(&result).unwrap_or_default()
                }],
                is_error: false,
            })
        } else {
            json!(ToolCallResult {
                content: vec![ToolContent::Text {
                    text: format!("Trigger not found: {}", params.id)
                }],
                is_error: true,
            })
        }
    }
}
