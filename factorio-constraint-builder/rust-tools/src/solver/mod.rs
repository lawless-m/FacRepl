// Constraint Solver Module
// Generates factory layouts from production goals

pub mod constraints;
pub mod placement;
pub mod ratios;

use crate::dsl::Command;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export key types
pub use constraints::{PlacedEntity, PlacementGrid};
pub use placement::{commands_to_script, GridPlacementSolver, PlacementConfig};
pub use ratios::{MachineRequirements, RatioCalculator};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Throughput {
    BeltFull(BeltType),
    ItemsPerSecond(f64),
    ItemsPerMinute(f64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BeltType {
    Yellow, // 15 items/sec
    Red,    // 30 items/sec
    Blue,   // 45 items/sec
}

impl BeltType {
    pub fn items_per_second(&self) -> f64 {
        match self {
            BeltType::Yellow => 15.0,
            BeltType::Red => 30.0,
            BeltType::Blue => 45.0,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            BeltType::Yellow => "belt-yellow",
            BeltType::Red => "belt-red",
            BeltType::Blue => "belt-blue",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductionGoal {
    pub item: String,
    pub throughput: Throughput,
}

impl ProductionGoal {
    pub fn items_per_second(&self) -> f64 {
        match &self.throughput {
            Throughput::BeltFull(belt) => belt.items_per_second(),
            Throughput::ItemsPerSecond(rate) => *rate,
            Throughput::ItemsPerMinute(rate) => rate / 60.0,
        }
    }

    /// Create a goal for a full belt of items
    pub fn full_belt(item: &str, belt_type: BeltType) -> Self {
        Self {
            item: item.to_string(),
            throughput: Throughput::BeltFull(belt_type),
        }
    }

    /// Create a goal for items per second
    pub fn per_second(item: &str, rate: f64) -> Self {
        Self {
            item: item.to_string(),
            throughput: Throughput::ItemsPerSecond(rate),
        }
    }

    /// Create a goal for items per minute
    pub fn per_minute(item: &str, rate: f64) -> Self {
        Self {
            item: item.to_string(),
            throughput: Throughput::ItemsPerMinute(rate),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Solution {
    pub commands: Vec<Command>,
    pub statistics: SolutionStats,
}

impl Solution {
    /// Convert the solution to a DSL script
    pub fn to_script(&self) -> String {
        commands_to_script(&self.commands)
    }

    /// Print a summary of the solution
    pub fn summary(&self) -> String {
        let mut lines = vec![
            format!(
                "Area: {}x{}",
                self.statistics.area_used.0, self.statistics.area_used.1
            ),
            format!("Machines:"),
        ];

        for (machine, count) in &self.statistics.machine_count {
            lines.push(format!("  {} x{}", machine, count));
        }

        lines.push(format!("Commands: {}", self.commands.len()));

        lines.join("\n")
    }
}

#[derive(Debug, Clone, Default)]
pub struct SolutionStats {
    pub machine_count: HashMap<String, u32>,
    pub area_used: (u32, u32),
    pub power_required: f64,
}

pub trait ConstraintSolver {
    fn solve(&self, goal: ProductionGoal) -> anyhow::Result<Solution>;
}

/// Main solver interface that uses GridPlacementSolver internally
pub struct FactorySolver {
    config: PlacementConfig,
}

impl FactorySolver {
    pub fn new() -> Self {
        Self {
            config: PlacementConfig::default(),
        }
    }

    pub fn with_config(config: PlacementConfig) -> Self {
        Self { config }
    }

    /// Solve for a production goal and return placement commands
    pub fn solve(&self, goal: &ProductionGoal) -> anyhow::Result<Solution> {
        let solver = GridPlacementSolver::new(self.config.clone());
        solver.solve(goal)
    }

    /// Calculate machine requirements without placement
    pub fn calculate_requirements(&self, goal: &ProductionGoal) -> Option<MachineRequirements> {
        let calc = RatioCalculator::new();
        calc.calculate(goal)
    }
}

impl Default for FactorySolver {
    fn default() -> Self {
        Self::new()
    }
}

impl ConstraintSolver for FactorySolver {
    fn solve(&self, goal: ProductionGoal) -> anyhow::Result<Solution> {
        self.solve(&goal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_production_goal() {
        let goal = ProductionGoal::full_belt("iron-gear", BeltType::Blue);
        assert_eq!(goal.items_per_second(), 45.0);

        let goal = ProductionGoal::per_second("copper-cable", 10.0);
        assert_eq!(goal.items_per_second(), 10.0);

        let goal = ProductionGoal::per_minute("green-circuit", 600.0);
        assert_eq!(goal.items_per_second(), 10.0);
    }

    #[test]
    fn test_factory_solver() {
        let solver = FactorySolver::new();
        let goal = ProductionGoal::per_second("iron-gear", 2.0);

        let solution = solver.solve(&goal).unwrap();
        assert!(!solution.commands.is_empty());
    }
}
