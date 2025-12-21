// Factorio Constraint Builder Library
// Core modules for DSL parsing, RCON communication, and constraint solving

pub mod dsl;
pub mod entities;
pub mod rcon;
pub mod solver;

// Re-export commonly used types
pub use dsl::{Command, DslExecutor, DslParser, ParseError};
pub use entities::{Direction, EntityType, Position, Recipe};
pub use rcon::{RconBridge, RconConfig, RconError};
pub use solver::{ConstraintSolver, FactorySolver, ProductionGoal, Solution};

/// Result type for library operations
pub type Result<T> = std::result::Result<T, anyhow::Error>;
