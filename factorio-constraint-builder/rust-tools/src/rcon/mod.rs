// RCON Bridge Module
// Handles communication with Factorio via RCON protocol

use anyhow::Result;
use rcon::Connection;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio::time::timeout;

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

    #[error("Not connected")]
    NotConnected,
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
    connection: Option<Connection<TcpStream>>,
}

impl RconBridge {
    /// Connect to Factorio RCON server
    pub async fn connect(config: RconConfig) -> Result<Self> {
        tracing::info!("Connecting to Factorio at {}:{}", config.host, config.port);

        let addr = format!("{}:{}", config.host, config.port);

        // Try to connect with timeout
        let connect_result = timeout(
            config.timeout,
            Connection::builder()
                .enable_factorio_quirks(true)
                .connect(&addr, &config.password),
        )
        .await;

        match connect_result {
            Ok(Ok(conn)) => {
                tracing::info!("Successfully connected to Factorio RCON");
                Ok(Self {
                    config,
                    connection: Some(conn),
                })
            }
            Ok(Err(e)) => {
                tracing::warn!("RCON connection failed: {} - running in offline mode", e);
                // Return a bridge in disconnected mode for offline testing
                Ok(Self {
                    config,
                    connection: None,
                })
            }
            Err(_) => {
                tracing::warn!("RCON connection timed out - running in offline mode");
                // Return a bridge in disconnected mode for offline testing
                Ok(Self {
                    config,
                    connection: None,
                })
            }
        }
    }

    /// Check if connected to Factorio
    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    /// Execute a raw RCON command and return the response string
    pub async fn execute_raw(&mut self, command: &str) -> Result<String> {
        tracing::debug!("Executing: {}", command);

        match &mut self.connection {
            Some(conn) => {
                let result = timeout(self.config.timeout, conn.cmd(command)).await;

                match result {
                    Ok(Ok(response)) => {
                        tracing::debug!("Response: {}", response);
                        Ok(response)
                    }
                    Ok(Err(e)) => Err(RconError::ExecutionFailed(e.to_string()).into()),
                    Err(_) => Err(RconError::Timeout.into()),
                }
            }
            None => {
                // Offline mode - return simulated success for testing
                tracing::debug!("Offline mode - simulating command: {}", command);
                Ok(
                    r#"{"success": true, "action": "simulated", "message": "offline mode"}"#
                        .to_string(),
                )
            }
        }
    }

    /// Execute a command and parse the JSON response
    pub async fn execute_command(&mut self, command: &str) -> Result<RconResponse> {
        let response = self.execute_raw(command).await?;

        // Handle empty response
        if response.is_empty() {
            return Ok(RconResponse {
                success: true,
                action: "command".to_string(),
                data: serde_json::json!({}),
            });
        }

        // Parse JSON response
        let parsed: RconResponse = serde_json::from_str(&response)
            .map_err(|e| RconError::InvalidResponse(format!("{}: {}", e, response)))?;

        Ok(parsed)
    }

    /// Place an entity in Factorio
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

    /// Query what entity is at a position
    pub async fn query_position(&mut self, x: i32, y: i32) -> Result<RconResponse> {
        let command = format!(
            "/silent-command rcon.print(remote.call('fcb', 'query', {{position = {{x = {}, y = {}}}}}))",
            x, y
        );

        self.execute_command(&command).await
    }

    /// Check if an entity can be placed at a position
    pub async fn can_place(&mut self, entity: &str, x: i32, y: i32) -> Result<RconResponse> {
        let command = format!(
            "/silent-command rcon.print(remote.call('fcb', 'can_place', {{entity = '{}', position = {{x = {}, y = {}}}}}))",
            entity, x, y
        );

        self.execute_command(&command).await
    }

    /// Clear all entities in an area
    pub async fn clear_area(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> Result<RconResponse> {
        let command = format!(
            "/silent-command rcon.print(remote.call('fcb', 'clear', {{area = {{left_top = {{x = {}, y = {}}}, right_bottom = {{x = {}, y = {}}}}}}}))",
            x1, y1, x2, y2
        );

        self.execute_command(&command).await
    }

    /// List all entities in an area
    pub async fn list_area(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> Result<RconResponse> {
        let command = format!(
            "/silent-command rcon.print(remote.call('fcb', 'list_area', {{area = {{left_top = {{x = {}, y = {}}}, right_bottom = {{x = {}, y = {}}}}}}}))",
            x1, y1, x2, y2
        );

        self.execute_command(&command).await
    }

    /// Remove an entity at a position
    pub async fn remove_entity(&mut self, x: i32, y: i32) -> Result<RconResponse> {
        let command = format!(
            "/silent-command rcon.print(remote.call('fcb', 'remove', {{position = {{x = {}, y = {}}}}}))",
            x, y
        );

        self.execute_command(&command).await
    }
}
