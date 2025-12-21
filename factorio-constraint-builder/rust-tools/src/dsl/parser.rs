// DSL Parser
// Parses DSL text into AST

use super::ast::*;
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

        // Check if it's an entity placement command
        if Self::is_entity(command) {
            return Self::parse_place_command(tokens);
        }

        match command {
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

            // Remove command
            "remove" => Self::parse_remove(tokens),

            _ => Err(ParseError::UnknownCommand(command.to_string())),
        }
    }

    /// Check if a command is a known entity type
    fn is_entity(cmd: &str) -> bool {
        // Belt entities
        cmd.starts_with("belt-")
            || cmd.starts_with("underground-belt-")
            || cmd.starts_with("splitter-")
            // Inserters
            || cmd.starts_with("inserter-")
            // Assemblers and factories
            || cmd.starts_with("assembler-")
            || cmd.starts_with("furnace-")
            || cmd.starts_with("chemical-plant")
            || cmd.starts_with("oil-refinery")
            || cmd.starts_with("centrifuge")
            // Power
            || cmd.starts_with("power-pole-")
            || cmd == "substation"
            || cmd.starts_with("solar-panel")
            || cmd.starts_with("accumulator")
            // Fluids
            || cmd == "pipe"
            || cmd.starts_with("pipe-to-ground")
            || cmd == "pump"
            || cmd == "storage-tank"
            || cmd == "offshore-pump"
            // Chests
            || cmd.starts_with("chest-")
            // Labs
            || cmd == "lab"
            // Mining
            || cmd.starts_with("drill-")
            || cmd == "pumpjack"
            // Logistics
            || cmd.starts_with("roboport")
            || cmd.starts_with("logistic-chest-")
            // Beacons
            || cmd == "beacon"
            // Trains
            || cmd.starts_with("rail-")
            || cmd == "train-stop"
            || cmd.starts_with("signal-")
            // Walls and defenses
            || cmd == "wall"
            || cmd.starts_with("turret-")
            // Radar
            || cmd == "radar"
            // Combinators
            || cmd.starts_with("combinator-")
            || cmd == "constant-combinator"
            || cmd == "power-switch"
            || cmd == "programmable-speaker"
            // Lamps
            || cmd == "lamp"
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
                    return Err(ParseError::InvalidSyntax(format!(
                        "Line {}: {}",
                        line_num + 1,
                        e
                    )));
                }
            }
        }

        Ok(commands)
    }

    fn parse_place_command(tokens: Vec<&str>) -> Result<Command, ParseError> {
        if tokens.len() < 3 {
            return Err(ParseError::MissingArgument(
                "entity placement requires: entity x y [direction] [options]".to_string(),
            ));
        }

        let entity_type = tokens[0].to_string();
        let x = Self::parse_number(tokens[1])?;
        let y = Self::parse_number(tokens[2])?;

        // Direction is optional - check if next token starts with ':'
        let (direction, start_options) = if tokens.len() > 3 && tokens[3].starts_with(':') {
            (
                Some(
                    Direction::from_keyword(&tokens[3][1..])
                        .ok_or_else(|| ParseError::InvalidDirection(tokens[3].to_string()))?,
                ),
                4,
            )
        } else {
            (None, 3)
        };

        // Parse optional recipe and modules
        let mut recipe = None;
        let mut modules = Vec::new();

        for token in tokens.iter().skip(start_options) {
            if let Some(recipe_name) = token.strip_prefix("recipe:") {
                recipe = Some(recipe_name.to_string());
            } else if let Some(module_name) = token.strip_prefix("module:") {
                modules.push(module_name.to_string());
            }
        }

        Ok(Command::Place(PlaceCommand {
            entity_type,
            position: Position::new(x, y),
            direction,
            recipe,
            modules,
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

        Ok(Command::Clear(ClearCommand::ClearEntity {
            entity_type,
            x1,
            y1,
            x2,
            y2,
        }))
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

        let name = tokens[1].strip_prefix(':').unwrap_or(tokens[1]).to_string();

        Ok(Command::SaveState { name })
    }

    fn parse_load_state(tokens: Vec<&str>) -> Result<Command, ParseError> {
        if tokens.len() < 2 {
            return Err(ParseError::MissingArgument("state name".to_string()));
        }

        let name = tokens[1].strip_prefix(':').unwrap_or(tokens[1]).to_string();

        Ok(Command::LoadState { name })
    }

    fn parse_remove(tokens: Vec<&str>) -> Result<Command, ParseError> {
        if tokens.len() < 3 {
            return Err(ParseError::MissingArgument("x y".to_string()));
        }

        let x = Self::parse_number(tokens[1])?;
        let y = Self::parse_number(tokens[2])?;

        Ok(Command::Remove(Position::new(x, y)))
    }

    fn parse_number(s: &str) -> Result<i32, ParseError> {
        s.parse()
            .map_err(|_| ParseError::InvalidNumber(s.to_string()))
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
    fn test_parse_belt_no_direction() {
        let cmd = DslParser::parse_line("belt-blue 10 20").unwrap();
        match cmd {
            Command::Place(place) => {
                assert_eq!(place.entity_type, "belt-blue");
                assert_eq!(place.direction, None);
            }
            _ => panic!("Expected Place command"),
        }
    }

    #[test]
    fn test_parse_assembler_with_recipe() {
        let cmd = DslParser::parse_line("assembler-2 15 15 :n recipe:green-circuit").unwrap();
        match cmd {
            Command::Place(place) => {
                assert_eq!(place.entity_type, "assembler-2");
                assert_eq!(place.recipe, Some("green-circuit".to_string()));
                assert_eq!(place.direction, Some(Direction::North));
            }
            _ => panic!("Expected Place command"),
        }
    }

    #[test]
    fn test_parse_assembler_with_modules() {
        let cmd =
            DslParser::parse_line("assembler-3 10 10 :n recipe:copper-cable module:speed-3 module:speed-3")
                .unwrap();
        match cmd {
            Command::Place(place) => {
                assert_eq!(place.entity_type, "assembler-3");
                assert_eq!(place.recipe, Some("copper-cable".to_string()));
                assert_eq!(place.modules, vec!["speed-3".to_string(), "speed-3".to_string()]);
            }
            _ => panic!("Expected Place command"),
        }
    }

    #[test]
    fn test_parse_comment() {
        let cmd = DslParser::parse_line("; This is a comment").unwrap();
        match cmd {
            Command::Comment(s) => assert_eq!(s, "This is a comment"),
            _ => panic!("Expected Comment"),
        }
    }

    #[test]
    fn test_parse_furnace() {
        let cmd = DslParser::parse_line("furnace-steel 5 5 :n").unwrap();
        match cmd {
            Command::Place(place) => {
                assert_eq!(place.entity_type, "furnace-steel");
                assert_eq!(place.position.x, 5);
            }
            _ => panic!("Expected Place command"),
        }
    }

    #[test]
    fn test_parse_remove() {
        let cmd = DslParser::parse_line("remove 10 20").unwrap();
        match cmd {
            Command::Remove(pos) => {
                assert_eq!(pos.x, 10);
                assert_eq!(pos.y, 20);
            }
            _ => panic!("Expected Remove command"),
        }
    }

    #[test]
    fn test_parse_clear() {
        let cmd = DslParser::parse_line("clear 0 0 50 50").unwrap();
        match cmd {
            Command::Clear(ClearCommand::ClearArea { x1, y1, x2, y2 }) => {
                assert_eq!(x1, 0);
                assert_eq!(y1, 0);
                assert_eq!(x2, 50);
                assert_eq!(y2, 50);
            }
            _ => panic!("Expected Clear command"),
        }
    }

    #[test]
    fn test_parse_undo() {
        let cmd = DslParser::parse_line("undo 5").unwrap();
        match cmd {
            Command::Undo { count } => assert_eq!(count, 5),
            _ => panic!("Expected Undo command"),
        }
    }

    #[test]
    fn test_parse_script() {
        let script = r#"
; Basic setup
belt-blue 0 0 :n
belt-blue 0 1 :n
assembler-2 5 5 :n recipe:iron-gear
"#;
        let commands = DslParser::parse_script(script).unwrap();
        assert_eq!(commands.len(), 4); // 1 comment + 3 commands
    }
}
