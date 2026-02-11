// Constraint Solver Module
// Generates factory layouts from production goals

pub mod constraints;
pub mod ratios;
pub mod placement;

use crate::dsl::Command;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Throughput {
    BeltFull(BeltType),
    ItemsPerSecond(f64),
    ItemsPerMinute(f64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BeltType {
    Yellow,  // 15 items/sec
    Red,     // 30 items/sec
    Blue,    // 45 items/sec
}

impl BeltType {
    pub fn items_per_second(&self) -> f64 {
        match self {
            BeltType::Yellow => 15.0,
            BeltType::Red => 30.0,
            BeltType::Blue => 45.0,
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
}

#[derive(Debug, Clone)]
pub struct Solution {
    pub commands: Vec<Command>,
    pub statistics: SolutionStats,
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

// Prolog-based solver implementation
pub struct PrologSolver {
    prolog_file: String,
}

impl PrologSolver {
    pub fn new(prolog_file: String) -> Self {
        Self { prolog_file }
    }
}

impl ConstraintSolver for PrologSolver {
    fn solve(&self, goal: ProductionGoal) -> anyhow::Result<Solution> {
        use std::process::Command as ProcessCommand;

        // Convert goal to Prolog query
        let throughput_scaled = (goal.items_per_second() * 100.0) as i32;
        let query = format!(
            "solve(goal({}, {}), Layout), print_layout(Layout).",
            goal.item, throughput_scaled
        );

        // Call scryer-prolog
        let output = ProcessCommand::new("scryer-prolog")
            .arg("-g")
            .arg(&query)
            .arg(&self.prolog_file)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Prolog solver failed: {}", stderr));
        }

        // Parse DSL commands from stdout
        let stdout = String::from_utf8_lossy(&output.stdout);
        let commands = parse_prolog_output(&stdout)?;

        // Count machines for statistics
        let mut machine_count = HashMap::new();
        for cmd in &commands {
            if let Command::Place(ref place) = cmd {
                *machine_count.entry(place.entity_type.clone()).or_insert(0) += 1;
            }
        }

        Ok(Solution {
            commands,
            statistics: SolutionStats {
                machine_count,
                ..Default::default()
            },
        })
    }
}

// Parse Prolog solver output (DSL commands)
fn parse_prolog_output(output: &str) -> anyhow::Result<Vec<Command>> {
    use crate::dsl::DslParser;

    let mut commands = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('%') || line.starts_with("true") {
            continue;
        }

        // Try to parse as DSL
        match DslParser::parse_line(line) {
            Ok(cmd) => commands.push(cmd),
            Err(_) => {
                // Skip lines that aren't DSL commands (Prolog output)
                continue;
            }
        }
    }

    Ok(commands)
}

// Placeholder solver implementation
pub struct BasicSolver;

impl ConstraintSolver for BasicSolver {
    fn solve(&self, _goal: ProductionGoal) -> anyhow::Result<Solution> {
        // TODO: Implement actual constraint solving
        // For now, return empty solution
        Ok(Solution {
            commands: Vec::new(),
            statistics: SolutionStats::default(),
        })
    }
}
