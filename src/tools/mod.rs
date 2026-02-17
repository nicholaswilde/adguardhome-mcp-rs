pub mod clients;
pub mod dns;
pub mod filtering;
pub mod protection;
pub mod sync;
pub mod system;

use crate::adguard::AdGuardClient;
use crate::config::AppConfig;
use crate::error::Result;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::sync::Arc;

pub type ToolHandler = dyn Fn(
        &AdGuardClient,
        Option<Value>,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Value>> + Send>>
    + Send
    + Sync;

#[derive(Clone)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub handler: Arc<ToolHandler>,
}

#[derive(Clone)]
pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
    enabled_tools: HashSet<String>,
    lazy_mode: bool,
}

impl ToolRegistry {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            tools: HashMap::new(),
            enabled_tools: HashSet::new(),
            lazy_mode: config.lazy_mode,
        }
    }

    pub fn register<F, Fut>(
        &mut self,
        name: &str,
        description: &str,
        input_schema: Value,
        handler: F,
    ) where
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

    pub async fn call_tool(
        &self,
        name: &str,
        client: &AdGuardClient,
        params: Option<Value>,
    ) -> Result<Value> {
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

    pub fn is_tool_enabled(&self, name: &str) -> bool {
        self.enabled_tools.contains(name)
    }

    pub fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tools.get(name).cloned()
    }
}

#[cfg(test)]
mod tests;
