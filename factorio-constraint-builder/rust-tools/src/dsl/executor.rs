// DSL Executor
// Executes parsed DSL commands via RCON

use super::ast::*;
use crate::rcon::RconBridge;
use anyhow::Result;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error("RCON error: {0}")]
    RconError(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),
}

pub struct ExecutionState {
    pub history: Vec<Command>,
    pub saved_states: HashMap<String, Vec<Command>>,
}

impl Default for ExecutionState {
    fn default() -> Self {
        Self {
            history: Vec::new(),
            saved_states: HashMap::new(),
        }
    }
}

pub struct DslExecutor {
    rcon: RconBridge,
    state: ExecutionState,
}

impl DslExecutor {
    pub fn new(rcon: RconBridge) -> Self {
        Self {
            rcon,
            state: ExecutionState::default(),
        }
    }

    /// Check if connected to Factorio
    pub fn is_connected(&self) -> bool {
        self.rcon.is_connected()
    }

    /// Get the number of commands in history
    pub fn history_len(&self) -> usize {
        self.state.history.len()
    }

    /// Execute a single DSL command
    pub async fn execute(&mut self, cmd: Command) -> Result<String> {
        // Add to history if it's a mutating command
        if cmd.is_mutating() {
            self.state.history.push(cmd.clone());
        }

        match cmd {
            Command::Place(place) => self.execute_place(place).await,
            Command::Query(query) => self.execute_query(query).await,
            Command::Clear(clear) => self.execute_clear(clear).await,
            Command::Remove(pos) => self.execute_remove(pos).await,
            Command::Undo { count } => self.execute_undo(count).await,
            Command::SaveState { name } => self.execute_save_state(name),
            Command::LoadState { name } => self.execute_load_state(name).await,
            Command::Comment(_) => Ok("".to_string()),
        }
    }

    /// Execute a script (multiple commands)
    pub async fn execute_script(&mut self, commands: Vec<Command>) -> Result<Vec<String>> {
        let mut results = Vec::new();

        for cmd in commands {
            match self.execute(cmd).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(results)
    }

    async fn execute_place(&mut self, place: PlaceCommand) -> Result<String> {
        let direction = place.direction.map(|d| d.to_keyword()).unwrap_or("north");

        let response = self
            .rcon
            .place_entity(
                &place.entity_type,
                place.position.x,
                place.position.y,
                direction,
                place.recipe.as_deref(),
                if place.modules.is_empty() {
                    None
                } else {
                    Some(place.modules)
                },
            )
            .await?;

        if response.success {
            Ok(format!(
                "Placed {} at ({}, {})",
                place.entity_type, place.position.x, place.position.y
            ))
        } else {
            Err(ExecutionError::CommandFailed(
                response
                    .data
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error")
                    .to_string(),
            )
            .into())
        }
    }

    async fn execute_query(&mut self, query: QueryCommand) -> Result<String> {
        match query {
            QueryCommand::WhatAt(pos) => {
                let response = self.rcon.query_position(pos.x, pos.y).await?;
                Ok(serde_json::to_string_pretty(&response.data)?)
            }
            QueryCommand::CanPlace {
                entity_type,
                position,
            } => {
                let response = self
                    .rcon
                    .can_place(&entity_type, position.x, position.y)
                    .await?;
                Ok(serde_json::to_string_pretty(&response.data)?)
            }
            QueryCommand::ListArea { x1, y1, x2, y2 } => {
                let response = self.rcon.list_area(x1, y1, x2, y2).await?;
                Ok(serde_json::to_string_pretty(&response.data)?)
            }
        }
    }

    async fn execute_clear(&mut self, clear: ClearCommand) -> Result<String> {
        match clear {
            ClearCommand::ClearArea { x1, y1, x2, y2 } => {
                let response = self.rcon.clear_area(x1, y1, x2, y2).await?;
                let count = response
                    .data
                    .get("cleared_count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                Ok(format!(
                    "Cleared {} entities in area ({},{}) to ({},{})",
                    count, x1, y1, x2, y2
                ))
            }
            ClearCommand::ClearEntity {
                entity_type,
                x1,
                y1,
                x2,
                y2,
            } => {
                // For now, we clear the whole area - selective clearing would need mod support
                let response = self.rcon.clear_area(x1, y1, x2, y2).await?;
                let count = response
                    .data
                    .get("cleared_count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                Ok(format!(
                    "Cleared {} entities of type {} in area ({},{}) to ({},{})",
                    count, entity_type, x1, y1, x2, y2
                ))
            }
        }
    }

    async fn execute_remove(&mut self, pos: Position) -> Result<String> {
        let response = self.rcon.remove_entity(pos.x, pos.y).await?;

        if response.success {
            Ok(format!("Removed entity at ({}, {})", pos.x, pos.y))
        } else {
            Err(ExecutionError::CommandFailed(
                response
                    .data
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error")
                    .to_string(),
            )
            .into())
        }
    }

    async fn execute_undo(&mut self, count: usize) -> Result<String> {
        if self.state.history.is_empty() {
            return Ok("Nothing to undo".to_string());
        }

        let undo_count = count.min(self.state.history.len());
        let mut undone = 0;

        for _ in 0..undo_count {
            if let Some(cmd) = self.state.history.pop() {
                // For Place commands, remove the entity
                if let Command::Place(place) = cmd {
                    match self.rcon.remove_entity(place.position.x, place.position.y).await {
                        Ok(_) => undone += 1,
                        Err(e) => {
                            tracing::warn!("Failed to undo placement: {}", e);
                        }
                    }
                }
            }
        }

        Ok(format!("Undone {} commands", undone))
    }

    fn execute_save_state(&mut self, name: String) -> Result<String> {
        self.state
            .saved_states
            .insert(name.clone(), self.state.history.clone());
        Ok(format!("Saved state: {}", name))
    }

    async fn execute_load_state(&mut self, name: String) -> Result<String> {
        let saved = self
            .state
            .saved_states
            .get(&name)
            .ok_or_else(|| ExecutionError::InvalidState(format!("State '{}' not found", name)))?
            .clone();

        // TODO: Clear current state and replay saved commands
        self.state.history = saved;
        Ok(format!("Loaded state: {} (replay not yet implemented)", name))
    }
}
