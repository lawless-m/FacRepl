// RCON Bridge Module
// Handles communication with Factorio via RCON protocol

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

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
    // Actual RCON client will be added in implementation phase
}

impl RconBridge {
    pub async fn connect(config: RconConfig) -> Result<Self> {
        // TODO: Implement actual RCON connection
        // For now, this is a placeholder
        tracing::info!("Connecting to Factorio at {}:{}", config.host, config.port);
        
        Ok(Self { config })
    }
    
    pub async fn execute_raw(&mut self, command: &str) -> Result<String> {
        // TODO: Implement RCON command execution
        // This will use the rcon crate to send commands
        tracing::debug!("Executing: {}", command);
        
        Ok(String::new())
    }
    
    pub async fn execute_command(&mut self, command: &str) -> Result<RconResponse> {
        let response = self.execute_raw(command).await?;
        
        // Parse JSON response
        let parsed: RconResponse = serde_json::from_str(&response)
            .map_err(|e| RconError::InvalidResponse(e.to_string()))?;
        
        Ok(parsed)
    }
    
    pub async fn place_entity(
        &mut self,
        entity: &str,
        x: i32,
        y: i32,
        direction: &str,
        recipe: Option<&str>,
        modules: Option<Vec<String>>,
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
        
        let command = format!(
            "/silent-command rcon.print(remote.call('fcb', 'place', {}))",
            params
        );
        
        self.execute_command(&command).await
    }
    
    pub async fn query_position(&mut self, x: i32, y: i32) -> Result<RconResponse> {
        let command = format!(
            "/silent-command rcon.print(remote.call('fcb', 'query', {{position = {{x = {}, y = {}}}}}))",
            x, y
        );
        
        self.execute_command(&command).await
    }
    
    pub async fn can_place(&mut self, entity: &str, x: i32, y: i32) -> Result<RconResponse> {
        let command = format!(
            "/silent-command rcon.print(remote.call('fcb', 'can_place', {{entity = '{}', position = {{x = {}, y = {}}}}}))",
            entity, x, y
        );
        
        self.execute_command(&command).await
    }
    
    pub async fn clear_area(
        &mut self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
    ) -> Result<RconResponse> {
        let command = format!(
            "/silent-command rcon.print(remote.call('fcb', 'clear', {{area = {{left_top = {{x = {}, y = {}}}, right_bottom = {{x = {}, y = {}}}}}}}))",
            x1, y1, x2, y2
        );
        
        self.execute_command(&command).await
    }
    
    pub async fn remove_entity(&mut self, x: i32, y: i32) -> Result<RconResponse> {
        let command = format!(
            "/silent-command rcon.print(remote.call('fcb', 'remove', {{position = {{x = {}, y = {}}}}}))",
            x, y
        );
        
        self.execute_command(&command).await
    }
}
