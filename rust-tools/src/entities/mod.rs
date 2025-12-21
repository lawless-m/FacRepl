// Entities Module
// Entity types, recipes, and game data

pub mod types;
pub mod recipes;

pub use types::{EntityType, EntityInfo, EntityCategory};
pub use recipes::{Recipe, RecipeInfo, Ingredient};

// Re-export from DSL for convenience
pub use crate::dsl::ast::{Position, Direction};
