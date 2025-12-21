// Factorio Constraint Builder Library
// Core modules for DSL parsing, RCON communication, and constraint solving

pub mod rcon;
pub mod dsl;
pub mod entities;
pub mod solver;

// Re-export commonly used types
pub use rcon::{RconBridge, RconConfig, RconError};
pub use dsl::{Command, DslParser, DslExecutor, ParseError};
pub use entities::{EntityType, Position, Direction, Recipe};
pub use solver::{ConstraintSolver, ProductionGoal, Solution};

/// Result type for library operations
pub type Result<T> = std::result::Result<T, anyhow::Error>;
