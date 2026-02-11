// DSL Parser
// Parses DSL text into AST

use super::ast::{Command, Direction, PlaceCommand, Position, QueryCommand, ClearCommand, SplitterPriority};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid syntax: {0}")]
    InvalidSyntax(String),
    
    #[error("Unknown command: {0}")]
    UnknownCommand(String),
    
    #[error("Invalid position: expected x y coordinates")]
    InvalidPosition,
    
    #[error("Invalid direction: {0}")]
    InvalidDirection(String),
    
    #[error("Missing required argument: {0}")]
    MissingArgument(String),
    
    #[error("Invalid number: {0}")]
    InvalidNumber(String),
}

pub struct DslParser;

impl DslParser {
    /// Parse a single line of DSL
    pub fn parse_line(input: &str) -> Result<Command, ParseError> {
        let input = input.trim();
        
        // Skip empty lines
        if input.is_empty() {
            return Ok(Command::Comment(String::new()));
        }
        
        // Handle comments
        if input.starts_with(';') {
            return Ok(Command::Comment(input[1..].trim().to_string()));
        }
        
        // Tokenize
        let tokens: Vec<&str> = input.split_whitespace().collect();
        if tokens.is_empty() {
            return Ok(Command::Comment(String::new()));
        }
        
        // Parse based on first token (command)
        let command = tokens[0];
        
        match command {
            // Entity placement commands
            cmd if cmd.starts_with("belt-")
                || cmd.starts_with("inserter-")
                || cmd.starts_with("assembler-")
                || cmd.starts_with("power-pole-")
                || cmd.starts_with("splitter-")
                || cmd.starts_with("underground-belt-")
                || cmd == "pipe"
                || cmd == "pump"
                || cmd == "storage-tank" => {
                Self::parse_place_command(tokens)
            }
            
            // Query commands
            "what-at" => Self::parse_what_at(tokens),
            "can-place" => Self::parse_can_place(tokens),
            "area" => Self::parse_list_area(tokens),
            
            // Area operations
            "clear" => Self::parse_clear(tokens),
            "clear-entity" => Self::parse_clear_entity(tokens),
            
            // State management
            "undo" => Self::parse_undo(tokens),
            "save-state" => Self::parse_save_state(tokens),
            "load-state" => Self::parse_load_state(tokens),
            
            _ => Err(ParseError::UnknownCommand(command.to_string())),
        }
    }
    
    /// Parse a full script (multiple lines)
    pub fn parse_script(input: &str) -> Result<Vec<Command>, ParseError> {
        let mut commands = Vec::new();
        
        for (line_num, line) in input.lines().enumerate() {
            match Self::parse_line(line) {
                Ok(cmd) => {
                    // Skip empty comments
                    if let Command::Comment(ref s) = cmd {
                        if s.is_empty() {
                            continue;
                        }
                    }
                    commands.push(cmd);
                }
                Err(e) => {
                    return Err(ParseError::InvalidSyntax(
                        format!("Line {}: {}", line_num + 1, e)
                    ));
                }
            }
        }
        
        Ok(commands)
    }
    
    fn parse_place_command(tokens: Vec<&str>) -> Result<Command, ParseError> {
        if tokens.len() < 3 {
            return Err(ParseError::MissingArgument(
                "entity placement requires: entity x y [direction] [options]".to_string()
            ));
        }

        let entity_type = tokens[0].to_string();
        let x = Self::parse_number(tokens[1])?;
        let y = Self::parse_number(tokens[2])?;

        // Direction is optional (parsed if 4th token starts with ':')
        let (direction, options_start) = if tokens.len() > 3 && tokens[3].starts_with(':') {
            (Some(Direction::from_keyword(&tokens[3][1..])
                .ok_or_else(|| ParseError::InvalidDirection(tokens[3].to_string()))?), 4)
        } else {
            (None, 3)
        };

        // Parse optional recipe, modules, and splitter settings
        let mut recipe = None;
        let mut modules = Vec::new();
        let mut input_priority = None;
        let mut output_priority = None;
        let mut filter = None;

        for token in tokens.iter().skip(options_start) {
            if let Some(recipe_name) = token.strip_prefix("recipe:") {
                recipe = Some(recipe_name.to_string());
            } else if let Some(module_name) = token.strip_prefix("module:") {
                modules.push(module_name.to_string());
            } else if let Some(priority) = token.strip_prefix("in-priority:") {
                input_priority = Some(SplitterPriority::from_keyword(priority)
                    .ok_or_else(|| ParseError::InvalidSyntax(
                        format!("Invalid input priority: {} (use left, right, or none)", priority)
                    ))?);
            } else if let Some(priority) = token.strip_prefix("out-priority:") {
                output_priority = Some(SplitterPriority::from_keyword(priority)
                    .ok_or_else(|| ParseError::InvalidSyntax(
                        format!("Invalid output priority: {} (use left, right, or none)", priority)
                    ))?);
            } else if let Some(filter_item) = token.strip_prefix("filter:") {
                filter = Some(filter_item.to_string());
            }
        }

        Ok(Command::Place(PlaceCommand {
            entity_type,
            position: Position::new(x, y),
            direction,
            recipe,
            modules,
            input_priority,
            output_priority,
            filter,
        }))
    }
    
    fn parse_what_at(tokens: Vec<&str>) -> Result<Command, ParseError> {
        if tokens.len() < 3 {
            return Err(ParseError::MissingArgument("x y".to_string()));
        }
        
        let x = Self::parse_number(tokens[1])?;
        let y = Self::parse_number(tokens[2])?;
        
        Ok(Command::Query(QueryCommand::WhatAt(Position::new(x, y))))
    }
    
    fn parse_can_place(tokens: Vec<&str>) -> Result<Command, ParseError> {
        if tokens.len() < 4 {
            return Err(ParseError::MissingArgument("entity x y".to_string()));
        }
        
        let entity_type = tokens[1].to_string();
        let x = Self::parse_number(tokens[2])?;
        let y = Self::parse_number(tokens[3])?;
        
        Ok(Command::Query(QueryCommand::CanPlace {
            entity_type,
            position: Position::new(x, y),
        }))
    }
    
    fn parse_list_area(tokens: Vec<&str>) -> Result<Command, ParseError> {
        if tokens.len() < 5 {
            return Err(ParseError::MissingArgument("x1 y1 x2 y2".to_string()));
        }
        
        let x1 = Self::parse_number(tokens[1])?;
        let y1 = Self::parse_number(tokens[2])?;
        let x2 = Self::parse_number(tokens[3])?;
        let y2 = Self::parse_number(tokens[4])?;
        
        Ok(Command::Query(QueryCommand::ListArea { x1, y1, x2, y2 }))
    }
    
    fn parse_clear(tokens: Vec<&str>) -> Result<Command, ParseError> {
        if tokens.len() < 5 {
            return Err(ParseError::MissingArgument("x1 y1 x2 y2".to_string()));
        }
        
        let x1 = Self::parse_number(tokens[1])?;
        let y1 = Self::parse_number(tokens[2])?;
        let x2 = Self::parse_number(tokens[3])?;
        let y2 = Self::parse_number(tokens[4])?;
        
        Ok(Command::Clear(ClearCommand::ClearArea { x1, y1, x2, y2 }))
    }
    
    fn parse_clear_entity(tokens: Vec<&str>) -> Result<Command, ParseError> {
        if tokens.len() < 6 {
            return Err(ParseError::MissingArgument("entity x1 y1 x2 y2".to_string()));
        }
        
        let entity_type = tokens[1].to_string();
        let x1 = Self::parse_number(tokens[2])?;
        let y1 = Self::parse_number(tokens[3])?;
        let x2 = Self::parse_number(tokens[4])?;
        let y2 = Self::parse_number(tokens[5])?;
        
        Ok(Command::Clear(ClearCommand::ClearEntity { entity_type, x1, y1, x2, y2 }))
    }
    
    fn parse_undo(tokens: Vec<&str>) -> Result<Command, ParseError> {
        let count = if tokens.len() > 1 {
            Self::parse_number(tokens[1])? as usize
        } else {
            1
        };
        
        Ok(Command::Undo { count })
    }
    
    fn parse_save_state(tokens: Vec<&str>) -> Result<Command, ParseError> {
        if tokens.len() < 2 {
            return Err(ParseError::MissingArgument("state name".to_string()));
        }
        
        let name = tokens[1].strip_prefix(':')
            .unwrap_or(tokens[1])
            .to_string();
        
        Ok(Command::SaveState { name })
    }
    
    fn parse_load_state(tokens: Vec<&str>) -> Result<Command, ParseError> {
        if tokens.len() < 2 {
            return Err(ParseError::MissingArgument("state name".to_string()));
        }
        
        let name = tokens[1].strip_prefix(':')
            .unwrap_or(tokens[1])
            .to_string();
        
        Ok(Command::LoadState { name })
    }
    
    fn parse_number(s: &str) -> Result<f64, ParseError> {
        // Remove trailing comma if present (for copy-paste friendliness)
        let s = s.trim_end_matches(',');
        s.parse().map_err(|_| ParseError::InvalidNumber(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_belt() {
        let cmd = DslParser::parse_line("belt-blue 10 20 :n").unwrap();
        match cmd {
            Command::Place(place) => {
                assert_eq!(place.entity_type, "belt-blue");
                assert_eq!(place.position.x, 10);
                assert_eq!(place.position.y, 20);
                assert_eq!(place.direction, Some(Direction::North));
            }
            _ => panic!("Expected Place command"),
        }
    }
    
    #[test]
    fn test_parse_assembler_with_recipe() {
        let cmd = DslParser::parse_line("assembler-2 15 15 recipe:green-circuit").unwrap();
        match cmd {
            Command::Place(place) => {
                assert_eq!(place.entity_type, "assembler-2");
                assert_eq!(place.recipe, Some("green-circuit".to_string()));
            }
            _ => panic!("Expected Place command"),
        }
    }
    
    #[test]
    fn test_parse_comment() {
        let cmd = DslParser::parse_line("; This is a comment").unwrap();
        assert!(matches!(cmd, Command::Comment(_)));
    }

    #[test]
    fn test_parse_splitter_with_output_priority() {
        let cmd = DslParser::parse_line("splitter-blue 10 10 :n out-priority:left").unwrap();
        match cmd {
            Command::Place(place) => {
                assert_eq!(place.entity_type, "splitter-blue");
                assert_eq!(place.output_priority, Some(SplitterPriority::Left));
                assert_eq!(place.input_priority, None);
                assert_eq!(place.filter, None);
            }
            _ => panic!("Expected Place command"),
        }
    }

    #[test]
    fn test_parse_splitter_with_input_priority() {
        let cmd = DslParser::parse_line("splitter-red 15 20 :e in-priority:right").unwrap();
        match cmd {
            Command::Place(place) => {
                assert_eq!(place.entity_type, "splitter-red");
                assert_eq!(place.input_priority, Some(SplitterPriority::Right));
                assert_eq!(place.output_priority, None);
            }
            _ => panic!("Expected Place command"),
        }
    }

    #[test]
    fn test_parse_splitter_with_filter() {
        let cmd = DslParser::parse_line("splitter-yellow 5 5 :s filter:iron-plate").unwrap();
        match cmd {
            Command::Place(place) => {
                assert_eq!(place.entity_type, "splitter-yellow");
                assert_eq!(place.filter, Some("iron-plate".to_string()));
            }
            _ => panic!("Expected Place command"),
        }
    }

    #[test]
    fn test_parse_splitter_with_all_options() {
        let cmd = DslParser::parse_line("splitter-blue 25 30 :w in-priority:left out-priority:right filter:copper-plate").unwrap();
        match cmd {
            Command::Place(place) => {
                assert_eq!(place.entity_type, "splitter-blue");
                assert_eq!(place.input_priority, Some(SplitterPriority::Left));
                assert_eq!(place.output_priority, Some(SplitterPriority::Right));
                assert_eq!(place.filter, Some("copper-plate".to_string()));
            }
            _ => panic!("Expected Place command"),
        }
    }

    #[test]
    fn test_parse_splitter_priority_none() {
        let cmd = DslParser::parse_line("splitter-blue 10 10 :n out-priority:none").unwrap();
        match cmd {
            Command::Place(place) => {
                assert_eq!(place.output_priority, Some(SplitterPriority::None));
            }
            _ => panic!("Expected Place command"),
        }
    }

    #[test]
    fn test_parse_splitter_priority_balanced() {
        let cmd = DslParser::parse_line("splitter-blue 10 10 :n in-priority:balanced").unwrap();
        match cmd {
            Command::Place(place) => {
                assert_eq!(place.input_priority, Some(SplitterPriority::None));
            }
            _ => panic!("Expected Place command"),
        }
    }
}
