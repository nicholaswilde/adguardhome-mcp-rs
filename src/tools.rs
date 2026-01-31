use crate::adguard::AdGuardClient;
use crate::error::Result;
use crate::settings::Settings;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::pin::Pin;

#[derive(Clone)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub handler: Arc<dyn Fn(&AdGuardClient, Option<Value>) -> Pin<Box<dyn std::future::Future<Output = Result<Value>> + Send>> + Send + Sync>,
}

pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
    enabled_tools: HashSet<String>,
    lazy_mode: bool,
}

impl ToolRegistry {
    pub fn new(settings: &Settings) -> Self {
        Self {
            tools: HashMap::new(),
            enabled_tools: HashSet::new(),
            lazy_mode: settings.lazy_mode,
        }
    }

    pub fn register<F, Fut>(&mut self, name: &str, description: &str, input_schema: Value, handler: F)
    where
        F: Fn(&AdGuardClient, Option<Value>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Value>> + Send + 'static,
    {
        let tool = Tool {
            name: name.to_string(),
            description: description.to_string(),
            input_schema,
            handler: Arc::new(move |client, params| Box::pin(handler(client, params))),
        };
        self.tools.insert(name.to_string(), tool);
        
        // In non-lazy mode, all tools are enabled by default
        if !self.lazy_mode {
            self.enabled_tools.insert(name.to_string());
        }
    }

    pub fn list_tools(&self) -> Vec<Value> {
        let mut result = Vec::new();
        
        // Always include "manage_tools" if in lazy mode (it will be registered separately or handled here)
        // Ideally manage_tools is just another tool that is always enabled.
        
        for tool_name in &self.enabled_tools {
            if let Some(tool) = self.tools.get(tool_name) {
                result.push(serde_json::json!({
                    "name": tool.name,
                    "description": tool.description,
                    "inputSchema": tool.input_schema
                }));
            }
        }
        
        // Sort for stability
        result.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
        
        result
    }

    pub async fn call_tool(&self, name: &str, client: &AdGuardClient, params: Option<Value>) -> Result<Value> {
        if !self.enabled_tools.contains(name) {
             return Err(crate::error::Error::Mcp(crate::mcp::ResponseError {
                code: -32601,
                message: format!("Tool not found or not enabled: {}", name),
                data: None,
            }));
        }

        if let Some(tool) = self.tools.get(name) {
            (tool.handler)(client, params).await
        } else {
            Err(crate::error::Error::Mcp(crate::mcp::ResponseError {
                code: -32601,
                message: format!("Tool not found: {}", name),
                data: None,
            }))
        }
    }
    
    // Management methods
    pub fn enable_tool(&mut self, name: &str) -> bool {
        if self.tools.contains_key(name) {
            self.enabled_tools.insert(name.to_string());
            true
        } else {
            false
        }
    }

    pub fn disable_tool(&mut self, name: &str) -> bool {
        // Prevent disabling manage_tools if we implement it as a regular tool? 
        // Or maybe manage_tools is special.
        self.enabled_tools.remove(name)
    }
    
    pub fn list_available_tools(&self) -> Vec<Value> {
        let mut result = Vec::new();
        for tool in self.tools.values() {
            result.push(serde_json::json!({
                "name": tool.name,
                "description": tool.description,
                "enabled": self.enabled_tools.contains(&tool.name)
            }));
        }
        result.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
        result
    }
}
