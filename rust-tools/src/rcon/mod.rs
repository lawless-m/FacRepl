// RCON Bridge Module
// Handles communication with Factorio via RCON protocol

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tokio::net::TcpStream;

// Helper function to convert JSON value to Lua table syntax
fn json_to_lua(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "nil".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "\\'")),
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(json_to_lua).collect();
            format!("{{{}}}", items.join(", "))
        }
        serde_json::Value::Object(obj) => {
            let items: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("{} = {}", k, json_to_lua(v)))
                .collect();
            format!("{{{}}}", items.join(", "))
        }
    }
}

#[derive(Error, Debug)]
pub enum RconError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    #[error("Timeout")]
    Timeout,
}

#[derive(Debug, Clone)]
pub struct RconConfig {
    pub host: String,
    pub port: u16,
    pub password: String,
    pub timeout: Duration,
}

impl Default for RconConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 27015,
            password: String::new(),
            timeout: Duration::from_secs(5),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RconResponse {
    pub success: bool,
    pub action: String,
    #[serde(flatten)]
    pub data: serde_json::Value,
}

pub struct RconBridge {
    config: RconConfig,
    client: rcon::Connection<TcpStream>,
}

impl RconBridge {
    pub async fn connect(config: RconConfig) -> Result<Self> {
        tracing::info!("Connecting to Factorio at {}:{}", config.host, config.port);

        let address = format!("{}:{}", config.host, config.port);
        let client = rcon::Connection::<TcpStream>::builder()
            .enable_factorio_quirks(true)
            .connect(&address, &config.password).await
            .map_err(|e| RconError::ConnectionFailed(e.to_string()))?;

        tracing::info!("Successfully connected to Factorio RCON");

        Ok(Self { config, client })
    }

    pub async fn execute_raw(&mut self, command: &str) -> Result<String> {
        tracing::debug!("Executing: {}", command);

        let response = self.client.cmd(command).await
            .map_err(|e| RconError::ExecutionFailed(e.to_string()))?;

        tracing::debug!("Response: {}", response);

        Ok(response)
    }
    
    pub async fn execute_command(&mut self, command: &str) -> Result<RconResponse> {
        let response = self.execute_raw(command).await?;

        // Handle empty response
        if response.trim().is_empty() {
            return Err(RconError::InvalidResponse("Empty response from server".to_string()).into());
        }

        // Parse JSON response
        let parsed: RconResponse = serde_json::from_str(&response)
            .map_err(|e| {
                tracing::error!("Failed to parse response as JSON: {}", response);
                RconError::InvalidResponse(format!("{} (response: {})", e, response))
            })?;

        Ok(parsed)
    }
    
    pub async fn place_entity(
        &mut self,
        entity: &str,
        x: f64,
        y: f64,
        direction: &str,
        recipe: Option<&str>,
        modules: Option<Vec<String>>,
        input_priority: Option<&str>,
        output_priority: Option<&str>,
        filter: Option<&str>,
    ) -> Result<RconResponse> {
        let mut params = serde_json::json!({
            "entity": entity,
            "position": {"x": x, "y": y},
            "direction": direction,
        });

        if let Some(r) = recipe {
            params["recipe"] = serde_json::json!(r);
        }

        if let Some(m) = modules {
            params["modules"] = serde_json::json!(m);
        }

        // Splitter-specific settings
        if let Some(ip) = input_priority {
            params["input_priority"] = serde_json::json!(ip);
        }

        if let Some(op) = output_priority {
            params["output_priority"] = serde_json::json!(op);
        }

        if let Some(f) = filter {
            params["filter"] = serde_json::json!(f);
        }

        let lua_params = json_to_lua(&params);
        let command = format!(
            "/silent-command rcon.print(remote.call('fcb', 'place', {}))",
            lua_params
        );

        self.execute_command(&command).await
    }
    
    pub async fn query_position(&mut self, x: f64, y: f64) -> Result<RconResponse> {
        let command = format!(
            "/silent-command rcon.print(remote.call('fcb', 'query', {{position = {{x = {}, y = {}}}}}))",
            x, y
        );
        
        self.execute_command(&command).await
    }
    
    pub async fn can_place(&mut self, entity: &str, x: f64, y: f64) -> Result<RconResponse> {
        let command = format!(
            "/silent-command rcon.print(remote.call('fcb', 'can_place', {{entity = '{}', position = {{x = {}, y = {}}}}}))",
            entity, x, y
        );
        
        self.execute_command(&command).await
    }
    
    pub async fn clear_area(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
    ) -> Result<RconResponse> {
        let command = format!(
            "/silent-command rcon.print(remote.call('fcb', 'clear', {{area = {{left_top = {{x = {}, y = {}}}, right_bottom = {{x = {}, y = {}}}}}}}))",
            x1, y1, x2, y2
        );
        
        self.execute_command(&command).await
    }
    
    pub async fn remove_entity(&mut self, x: f64, y: f64) -> Result<RconResponse> {
        let command = format!(
            "/silent-command rcon.print(remote.call('fcb', 'remove', {{position = {{x = {}, y = {}}}}}))",
            x, y
        );

        self.execute_command(&command).await
    }

    pub async fn spawn_chest(&mut self, items: Vec<(String, u32)>) -> Result<RconResponse> {
        // Build items array
        let items_json: Vec<String> = items.iter()
            .map(|(name, count)| format!("{{name = '{}', count = {}}}", name, count))
            .collect();

        let command = format!(
            "/silent-command rcon.print(remote.call('fcb', 'spawn_chest', {{items = {{{}}}}}))",
            items_json.join(", ")
        );

        self.execute_command(&command).await
    }

    pub async fn create_blueprint(&mut self, name: &str, entities: Vec<BlueprintEntity>) -> Result<RconResponse> {
        // Build entities array
        let entities_lua: Vec<String> = entities.iter()
            .map(|e| {
                let mut parts = vec![
                    format!("entity_name = '{}'", e.entity_name),
                    format!("position = {{x = {}, y = {}}}", e.position.0, e.position.1),
                ];

                if let Some(dir) = e.direction {
                    parts.push(format!("direction = {}", dir));
                }

                if let Some(ref recipe) = e.recipe {
                    parts.push(format!("recipe = '{}'", recipe));
                }

                format!("{{{}}}", parts.join(", "))
            })
            .collect();

        let command = format!(
            "/silent-command rcon.print(remote.call('fcb', 'create_blueprint', {{name = '{}', entities = {{{}}}}}))",
            name, entities_lua.join(", ")
        );

        self.execute_command(&command).await
    }
}

#[derive(Debug, Clone)]
pub struct BlueprintEntity {
    pub entity_name: String,
    pub position: (f64, f64),
    pub direction: Option<u8>,
    pub recipe: Option<String>,
}
