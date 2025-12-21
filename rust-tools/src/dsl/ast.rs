// Abstract Syntax Tree for DSL

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

impl Direction {
    pub fn from_keyword(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "n" | "north" => Some(Direction::North),
            "s" | "south" => Some(Direction::South),
            "e" | "east" => Some(Direction::East),
            "w" | "west" => Some(Direction::West),
            "ne" | "northeast" => Some(Direction::NorthEast),
            "nw" | "northwest" => Some(Direction::NorthWest),
            "se" | "southeast" => Some(Direction::SouthEast),
            "sw" | "southwest" => Some(Direction::SouthWest),
            _ => None,
        }
    }
    
    pub fn to_keyword(&self) -> &'static str {
        match self {
            Direction::North => "north",
            Direction::South => "south",
            Direction::East => "east",
            Direction::West => "west",
            Direction::NorthEast => "northeast",
            Direction::NorthWest => "northwest",
            Direction::SouthEast => "southeast",
            Direction::SouthWest => "southwest",
        }
    }
}

/// Priority setting for splitter input/output
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitterPriority {
    Left,
    Right,
    None,
}

impl SplitterPriority {
    pub fn from_keyword(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "left" | "l" => Some(SplitterPriority::Left),
            "right" | "r" => Some(SplitterPriority::Right),
            "none" | "balanced" => Some(SplitterPriority::None),
            _ => None,
        }
    }

    pub fn to_keyword(&self) -> &'static str {
        match self {
            SplitterPriority::Left => "left",
            SplitterPriority::Right => "right",
            SplitterPriority::None => "none",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaceCommand {
    pub entity_type: String,
    pub position: Position,
    pub direction: Option<Direction>,
    pub recipe: Option<String>,
    pub modules: Vec<String>,
    /// Splitter input priority (which input side to prefer)
    pub input_priority: Option<SplitterPriority>,
    /// Splitter output priority (which output side to prefer)
    pub output_priority: Option<SplitterPriority>,
    /// Splitter filter (item name to route to priority side)
    pub filter: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryCommand {
    WhatAt(Position),
    CanPlace { entity_type: String, position: Position },
    ListArea { x1: i32, y1: i32, x2: i32, y2: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClearCommand {
    ClearArea { x1: i32, y1: i32, x2: i32, y2: i32 },
    ClearEntity { entity_type: String, x1: i32, y1: i32, x2: i32, y2: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    Place(PlaceCommand),
    Query(QueryCommand),
    Clear(ClearCommand),
    Remove(Position),
    Undo { count: usize },
    SaveState { name: String },
    LoadState { name: String },
    Comment(String),
}

impl Command {
    /// Check if this command modifies game state
    pub fn is_mutating(&self) -> bool {
        matches!(
            self,
            Command::Place(_) | Command::Clear(_) | Command::Remove(_) | Command::Undo { .. }
        )
    }
    
    /// Check if this is a query command
    pub fn is_query(&self) -> bool {
        matches!(self, Command::Query(_))
    }
}
