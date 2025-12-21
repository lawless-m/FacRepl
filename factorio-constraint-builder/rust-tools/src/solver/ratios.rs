// Production ratio calculations
// Calculates required machines based on recipes and crafting speeds

use crate::entities::{EntityType, Recipe};
use crate::solver::{BeltType, ProductionGoal};
use std::collections::HashMap;

/// Machine requirements for producing an item
#[derive(Debug, Clone)]
pub struct MachineRequirements {
    /// The item being produced
    pub item: String,
    /// Required items per second
    pub items_per_second: f64,
    /// Machine type to use (e.g., "assembler-3")
    pub machine_type: String,
    /// Number of machines needed (fractional)
    pub machine_count: f64,
    /// Dependencies - other items needed to produce this
    pub dependencies: Vec<MachineRequirements>,
}

/// Calculator for production ratios
pub struct RatioCalculator {
    recipes: Recipe,
    entities: EntityType,
}

impl RatioCalculator {
    pub fn new() -> Self {
        Self {
            recipes: Recipe::new(),
            entities: EntityType::new(),
        }
    }

    /// Calculate machine requirements for a production goal
    pub fn calculate(&self, goal: &ProductionGoal) -> Option<MachineRequirements> {
        let target_rate = goal.items_per_second();
        self.calculate_for_item(&goal.item, target_rate, "assembler-3")
    }

    /// Calculate requirements for a specific item at a given rate
    fn calculate_for_item(
        &self,
        item: &str,
        items_per_second: f64,
        default_machine: &str,
    ) -> Option<MachineRequirements> {
        let recipe = self.recipes.get(item)?;

        // Get machine info
        let machine_info = self.entities.get(default_machine)?;
        let crafting_speed = machine_info.crafting_speed.unwrap_or(1.0);

        // Calculate items produced per machine per second
        let products_per_craft = recipe
            .products
            .first()
            .map(|p| p.amount)
            .unwrap_or(1.0);
        let items_per_machine_per_second = products_per_craft * crafting_speed / recipe.crafting_time;

        // Calculate required machines
        let machine_count = items_per_second / items_per_machine_per_second;

        // Calculate dependencies recursively
        let mut dependencies = Vec::new();
        for ingredient in &recipe.ingredients {
            let ingredient_rate = (ingredient.amount / products_per_craft) * items_per_second;

            if let Some(dep) = self.calculate_for_item(&ingredient.name, ingredient_rate, default_machine) {
                dependencies.push(dep);
            }
        }

        Some(MachineRequirements {
            item: item.to_string(),
            items_per_second,
            machine_type: default_machine.to_string(),
            machine_count,
            dependencies,
        })
    }

    /// Get a summary of all machines needed
    pub fn summarize_machines(&self, req: &MachineRequirements) -> HashMap<String, f64> {
        let mut summary: HashMap<String, f64> = HashMap::new();
        self.collect_machines(req, &mut summary);
        summary
    }

    fn collect_machines(&self, req: &MachineRequirements, summary: &mut HashMap<String, f64>) {
        *summary.entry(format!("{} ({})", req.machine_type, req.item)).or_insert(0.0) += req.machine_count;
        for dep in &req.dependencies {
            self.collect_machines(dep, summary);
        }
    }

    /// Round up machine counts to whole numbers
    pub fn round_machine_counts(req: &MachineRequirements) -> MachineRequirements {
        MachineRequirements {
            item: req.item.clone(),
            items_per_second: req.items_per_second,
            machine_type: req.machine_type.clone(),
            machine_count: req.machine_count.ceil(),
            dependencies: req.dependencies.iter().map(Self::round_machine_counts).collect(),
        }
    }
}

impl Default for RatioCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Belt throughput helper
pub fn belt_capacity(belt_type: BeltType) -> f64 {
    belt_type.items_per_second()
}

/// Calculate how many belts are needed for a given throughput
pub fn belts_needed(items_per_second: f64, belt_type: BeltType) -> f64 {
    items_per_second / belt_capacity(belt_type)
}

/// Calculate inserter throughput (approximate)
pub fn inserter_throughput(inserter_type: &str) -> f64 {
    match inserter_type {
        "inserter-burner" => 0.6,
        "inserter-basic" => 0.83,
        "inserter-long" => 1.2,
        "inserter-fast" => 2.31,
        "inserter-stack" => 4.44, // with stack size 12
        _ => 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::Throughput;

    #[test]
    fn test_iron_gear_ratio() {
        let calc = RatioCalculator::new();
        let goal = ProductionGoal {
            item: "iron-gear".to_string(),
            throughput: Throughput::ItemsPerSecond(2.0),
        };

        let req = calc.calculate(&goal).unwrap();
        assert_eq!(req.item, "iron-gear");
        // Assembler-3 with speed 1.25, recipe 0.5s
        // 1 machine produces 1.25/0.5 = 2.5 gears/sec
        // For 2.0/sec, need 2.0/2.5 = 0.8 machines
        assert!(req.machine_count > 0.7 && req.machine_count < 0.9);
    }

    #[test]
    fn test_copper_cable_ratio() {
        let calc = RatioCalculator::new();
        let goal = ProductionGoal {
            item: "copper-cable".to_string(),
            throughput: Throughput::ItemsPerSecond(10.0),
        };

        let req = calc.calculate(&goal).unwrap();
        assert_eq!(req.item, "copper-cable");
        // Copper cable produces 2 per craft, so 10/sec needs fewer machines
    }

    #[test]
    fn test_green_circuit_ratio() {
        let calc = RatioCalculator::new();
        let goal = ProductionGoal {
            item: "green-circuit".to_string(),
            throughput: Throughput::BeltFull(BeltType::Blue),
        };

        let req = calc.calculate(&goal).unwrap();
        assert_eq!(req.item, "green-circuit");
        // Should have copper-cable as dependency
        assert!(!req.dependencies.is_empty());
    }

    #[test]
    fn test_belt_capacity() {
        assert_eq!(belt_capacity(BeltType::Yellow), 15.0);
        assert_eq!(belt_capacity(BeltType::Red), 30.0);
        assert_eq!(belt_capacity(BeltType::Blue), 45.0);
    }
}
