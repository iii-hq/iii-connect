// Copyright Motia LLC and/or licensed to Motia LLC under one or more
// contributor license agreements. Licensed under the Elastic License 2.0;
// you may not use this file except in compliance with the Elastic License 2.0.
// This software is patent protected. We welcome discussions - reach out at support@motia.dev
// See LICENSE and PATENTS files for details.

use iii_sdk::Bridge;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesListResult {
    pub resources: Vec<McpResource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReadParams {
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceContent {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReadResult {
    pub contents: Vec<ResourceContent>,
}

pub struct ResourcesHandler {
    bridge: Bridge,
}

impl ResourcesHandler {
    pub fn new(bridge: Bridge) -> Self {
        Self { bridge }
    }

    pub async fn list(&self) -> Value {
        let resources = vec![
            McpResource {
                uri: "iii://functions".to_string(),
                name: "Functions".to_string(),
                description: Some("All registered iii-engine functions".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            McpResource {
                uri: "iii://workers".to_string(),
                name: "Workers".to_string(),
                description: Some("Connected iii-engine workers".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            McpResource {
                uri: "iii://triggers".to_string(),
                name: "Triggers".to_string(),
                description: Some("Registered iii-engine triggers".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            McpResource {
                uri: "iii://context".to_string(),
                name: "Context".to_string(),
                description: Some("Runtime context available to functions (logging, state, events, tracing)".to_string()),
                mime_type: Some("application/json".to_string()),
            },
        ];

        json!(ResourcesListResult { resources })
    }

    pub async fn read(&self, params: Option<Value>) -> Value {
        let params: ResourceReadParams = match params {
            Some(p) => match serde_json::from_value(p) {
                Ok(p) => p,
                Err(err) => {
                    return json!({
                        "error": format!("Invalid params: {}", err)
                    });
                }
            },
            None => {
                return json!({
                    "error": "Missing params"
                });
            }
        };

        let uri = &params.uri;
        tracing::debug!(uri = %uri, "Reading resource");

        let (text, mime_type) = match uri.as_str() {
            "iii://functions" => match self.bridge.list_functions().await {
                Ok(functions) => {
                    let text = serde_json::to_string_pretty(&functions)
                        .unwrap_or_else(|_| "[]".to_string());
                    (text, "application/json")
                }
                Err(err) => {
                    return json!({
                        "error": format!("Failed to list functions: {}", err)
                    });
                }
            },
            "iii://workers" => match self.bridge.list_workers().await {
                Ok(workers) => {
                    let text =
                        serde_json::to_string_pretty(&workers).unwrap_or_else(|_| "[]".to_string());
                    (text, "application/json")
                }
                Err(err) => {
                    return json!({
                        "error": format!("Failed to list workers: {}", err)
                    });
                }
            },
            "iii://triggers" => match self.bridge.list_triggers().await {
                Ok(triggers) => {
                    let text = serde_json::to_string_pretty(&triggers)
                        .unwrap_or_else(|_| "[]".to_string());
                    (text, "application/json")
                }
                Err(err) => {
                    return json!({
                        "error": format!("Failed to list triggers: {}", err)
                    });
                }
            },
            "iii://context" => {
                let context_info = json!({
                    "description": "Runtime context available to functions during execution",
                    "capabilities": {
                        "logger": {
                            "description": "Logging capabilities",
                            "tools": ["engine_log_info", "engine_log_debug", "engine_log_warn", "engine_log_error", "engine_log_trace"]
                        },
                        "state": {
                            "description": "Persistent data access",
                            "tools": ["state_get", "state_set", "state_delete", "state_update", "state_list"]
                        },
                        "events": {
                            "description": "Event publishing",
                            "tools": ["emit", "publish"]
                        },
                        "tracing": {
                            "description": "Distributed tracing and baggage",
                            "tools": ["engine_baggage_get", "engine_baggage_set", "engine_baggage_getAll"]
                        }
                    }
                });
                let text = serde_json::to_string_pretty(&context_info)
                    .unwrap_or_else(|_| "{}".to_string());
                (text, "application/json")
            },
            _ => {
                return json!({
                    "error": format!("Unknown resource: {}", uri)
                });
            }
        };

        json!(ResourceReadResult {
            contents: vec![ResourceContent {
                uri: uri.clone(),
                mime_type: Some(mime_type.to_string()),
                text: Some(text),
            }]
        })
    }
}
