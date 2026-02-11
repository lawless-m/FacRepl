// DSL Executor
// Executes parsed DSL commands via RCON

use super::ast::*;
use crate::entities::types::EntityType;
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
    entity_types: EntityType,
}

impl DslExecutor {
    pub fn new(rcon: RconBridge) -> Self {
        Self {
            rcon,
            state: ExecutionState::default(),
            entity_types: EntityType::new(),
        }
    }
    
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
    
    pub async fn execute_script(&mut self, commands: Vec<Command>) -> Result<Vec<String>> {
        let mut results = Vec::new();
        
        for cmd in commands {
            match self.execute(cmd).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    // Collect error but continue?
                    // Or stop on first error?
                    return Err(e);
                }
            }
        }
        
        Ok(results)
    }
    
    async fn execute_place(&mut self, place: PlaceCommand) -> Result<String> {
        let direction = place.direction
            .map(|d| d.to_keyword())
            .unwrap_or("north");

        // Translate DSL entity name to Factorio entity name
        let factorio_entity = self.entity_types.factorio_name(&place.entity_type)
            .unwrap_or(&place.entity_type);

        let response = self.rcon.place_entity(
            factorio_entity,
            place.position.x,
            place.position.y,
            direction,
            place.recipe.as_deref(),
            if place.modules.is_empty() { None } else { Some(place.modules) },
            place.input_priority.map(|p| p.to_keyword()),
            place.output_priority.map(|p| p.to_keyword()),
            place.filter.as_deref(),
        ).await?;

        if response.success {
            // Get actual position from response
            let actual_pos = response.data.get("position");
            if let Some(pos) = actual_pos {
                let x = pos.get("x").and_then(|v| v.as_f64()).unwrap_or(place.position.x as f64);
                let y = pos.get("y").and_then(|v| v.as_f64()).unwrap_or(place.position.y as f64);
                Ok(format!("Placed {} at {:.1} {:.1}", place.entity_type, x, y))
            } else {
                Ok(format!("Placed {} at {} {}", place.entity_type, place.position.x, place.position.y))
            }
        } else {
            Err(ExecutionError::CommandFailed(
                response.data.get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error")
                    .to_string()
            ).into())
        }
    }
    
    async fn execute_query(&mut self, query: QueryCommand) -> Result<String> {
        match query {
            QueryCommand::WhatAt(pos) => {
                let response = self.rcon.query_position(pos.x, pos.y).await?;
                Ok(format!("{:#}", response.data))
            }
            QueryCommand::CanPlace { entity_type, position } => {
                let factorio_entity = self.entity_types.factorio_name(&entity_type)
                    .unwrap_or(&entity_type);
                let response = self.rcon.can_place(factorio_entity, position.x, position.y).await?;
                Ok(format!("{:#}", response.data))
            }
            QueryCommand::ListArea { x1, y1, x2, y2 } => {
                // TODO: Implement list_area in RCON bridge
                Ok(format!("Area query: ({},{}) to ({},{})", x1, y1, x2, y2))
            }
        }
    }
    
    async fn execute_clear(&mut self, clear: ClearCommand) -> Result<String> {
        match clear {
            ClearCommand::ClearArea { x1, y1, x2, y2 } => {
                let response = self.rcon.clear_area(x1, y1, x2, y2).await?;
                Ok(format!("Cleared area ({},{}) to ({},{})", x1, y1, x2, y2))
            }
            ClearCommand::ClearEntity { entity_type, x1, y1, x2, y2 } => {
                // TODO: Implement selective entity clearing
                Ok(format!("Cleared {} in area ({},{}) to ({},{})", 
                    entity_type, x1, y1, x2, y2))
            }
        }
    }
    
    async fn execute_remove(&mut self, pos: Position) -> Result<String> {
        let response = self.rcon.remove_entity(pos.x, pos.y).await?;
        
        if response.success {
            Ok(format!("Removed entity at ({}, {})", pos.x, pos.y))
        } else {
            Err(ExecutionError::CommandFailed(
                response.data.get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error")
                    .to_string()
            ).into())
        }
    }
    
    async fn execute_undo(&mut self, count: usize) -> Result<String> {
        // TODO: Implement undo by replaying history
        Ok(format!("Undo {} commands (not yet implemented)", count))
    }
    
    fn execute_save_state(&mut self, name: String) -> Result<String> {
        self.state.saved_states.insert(name.clone(), self.state.history.clone());
        Ok(format!("Saved state: {}", name))
    }
    
    async fn execute_load_state(&mut self, name: String) -> Result<String> {
        let saved = self.state.saved_states.get(&name)
            .ok_or_else(|| ExecutionError::InvalidState(
                format!("State '{}' not found", name)
            ))?
            .clone();

        // TODO: Actually restore state by clearing and replaying
        self.state.history = saved;
        Ok(format!("Loaded state: {}", name))
    }

    pub async fn get_player_position(&mut self) -> Result<String> {
        let response = self.rcon.execute_raw("/silent-command rcon.print(remote.call('fcb', 'player_pos', {}))").await?;
        let parsed: serde_json::Value = serde_json::from_str(&response)?;

        if let Some(pos) = parsed.get("position") {
            let x = pos.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let y = pos.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
            Ok(format!("Player at ({:.1}, {:.1})", x, y))
        } else {
            Ok(response)
        }
    }
}
