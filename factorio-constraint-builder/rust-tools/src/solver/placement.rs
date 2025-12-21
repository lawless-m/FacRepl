// Spatial placement solver
// Implements algorithms for placing entities while satisfying constraints

use crate::dsl::ast::{Command, Direction, PlaceCommand, Position};
use crate::entities::EntityType;
use crate::solver::constraints::{adjacent_position, PlacedEntity, PlacementGrid};
use crate::solver::ratios::{MachineRequirements, RatioCalculator};
use crate::solver::{ProductionGoal, Solution, SolutionStats};

/// Configuration for the placement algorithm
#[derive(Debug, Clone)]
pub struct PlacementConfig {
    /// Starting position for layout
    pub origin: Position,
    /// Maximum width of layout
    pub max_width: Option<u32>,
    /// Maximum height of layout
    pub max_height: Option<u32>,
    /// Spacing between machines
    pub machine_spacing: u32,
    /// Belt type to use
    pub belt_type: String,
    /// Inserter type to use
    pub inserter_type: String,
}

impl Default for PlacementConfig {
    fn default() -> Self {
        Self {
            origin: Position::new(0, 0),
            max_width: None,
            max_height: None,
            machine_spacing: 1,
            belt_type: "belt-blue".to_string(),
            inserter_type: "inserter-fast".to_string(),
        }
    }
}

/// A simple grid-based placement solver
pub struct GridPlacementSolver {
    config: PlacementConfig,
    entities: EntityType,
}

impl GridPlacementSolver {
    pub fn new(config: PlacementConfig) -> Self {
        Self {
            config,
            entities: EntityType::new(),
        }
    }

    /// Solve placement for a production goal
    pub fn solve(&self, goal: &ProductionGoal) -> anyhow::Result<Solution> {
        let calc = RatioCalculator::new();

        // Calculate machine requirements
        let requirements = calc
            .calculate(goal)
            .ok_or_else(|| anyhow::anyhow!("Unknown recipe: {}", goal.item))?;

        // Round up machine counts
        let requirements = RatioCalculator::round_machine_counts(&requirements);

        // Generate placement
        let mut grid = PlacementGrid::new();
        let mut commands = Vec::new();
        let mut stats = SolutionStats::default();

        // Place machines in a grid pattern
        self.place_production_chain(&requirements, &mut grid, &mut commands, &mut stats)?;

        // Calculate used area
        if let Some((min, max)) = grid.used_bounds() {
            stats.area_used = (
                (max.x - min.x + 1) as u32,
                (max.y - min.y + 1) as u32,
            );
        }

        Ok(Solution { commands, statistics: stats })
    }

    /// Place a production chain (recursive for dependencies)
    fn place_production_chain(
        &self,
        req: &MachineRequirements,
        grid: &mut PlacementGrid,
        commands: &mut Vec<Command>,
        stats: &mut SolutionStats,
    ) -> anyhow::Result<Position> {
        // First place all dependencies
        let mut dep_positions = Vec::new();
        for dep in &req.dependencies {
            let pos = self.place_production_chain(dep, grid, commands, stats)?;
            dep_positions.push(pos);
        }

        // Get entity info
        let entity_info = self.entities.get(&req.machine_type).ok_or_else(|| {
            anyhow::anyhow!("Unknown entity type: {}", req.machine_type)
        })?;

        let size = entity_info.size;
        let machine_count = req.machine_count as u32;

        // Find starting position
        let start_pos = if dep_positions.is_empty() {
            self.config.origin.clone()
        } else {
            // Place after dependencies
            let last_dep = dep_positions.last().unwrap();
            Position::new(last_dep.x + 5, last_dep.y)
        };

        // Place machines in a row
        let mut current_x = start_pos.x;
        let mut first_pos = None;

        for _ in 0..machine_count {
            let pos = Position::new(current_x, start_pos.y);

            // Check if we can place here
            let entity = PlacedEntity::new(&req.machine_type, pos.clone(), size);

            if grid.try_place(entity).is_ok() {
                // Generate place command
                let cmd = Command::Place(PlaceCommand {
                    entity_type: req.machine_type.clone(),
                    position: pos.clone(),
                    direction: Some(Direction::North),
                    recipe: Some(req.item.clone()),
                    modules: Vec::new(),
                });
                commands.push(cmd);

                // Track statistics
                *stats.machine_count.entry(req.machine_type.clone()).or_insert(0) += 1;

                if first_pos.is_none() {
                    first_pos = Some(pos.clone());
                }

                // Add inserters on both sides
                self.add_inserters(&pos, size, grid, commands)?;

                current_x += size.0 as i32 + self.config.machine_spacing as i32;
            }
        }

        // Add input belt line (if we have dependencies)
        if !dep_positions.is_empty() && first_pos.is_some() {
            let belt_start = Position::new(
                first_pos.as_ref().unwrap().x - 1,
                first_pos.as_ref().unwrap().y - 1,
            );
            let belt_count = (current_x - belt_start.x) as u32;
            self.add_belt_line(&belt_start, belt_count, Direction::East, grid, commands)?;
        }

        // Add output belt line
        if let Some(ref pos) = first_pos {
            let belt_start = Position::new(
                pos.x - 1,
                pos.y + size.1 as i32,
            );
            let belt_count = (current_x - belt_start.x) as u32;
            self.add_belt_line(&belt_start, belt_count, Direction::East, grid, commands)?;
        }

        Ok(first_pos.unwrap_or(start_pos))
    }

    /// Add inserters around a machine
    fn add_inserters(
        &self,
        machine_pos: &Position,
        machine_size: (u32, u32),
        grid: &mut PlacementGrid,
        commands: &mut Vec<Command>,
    ) -> anyhow::Result<()> {
        // Input inserter (north side, facing south to insert into machine)
        let input_pos = Position::new(
            machine_pos.x + (machine_size.0 / 2) as i32,
            machine_pos.y - 1,
        );

        let input_entity = PlacedEntity::new(&self.config.inserter_type, input_pos.clone(), (1, 1));
        if grid.try_place(input_entity).is_ok() {
            commands.push(Command::Place(PlaceCommand {
                entity_type: self.config.inserter_type.clone(),
                position: input_pos,
                direction: Some(Direction::South),
                recipe: None,
                modules: Vec::new(),
            }));
        }

        // Output inserter (south side, facing south to extract from machine)
        let output_pos = Position::new(
            machine_pos.x + (machine_size.0 / 2) as i32,
            machine_pos.y + machine_size.1 as i32,
        );

        let output_entity = PlacedEntity::new(&self.config.inserter_type, output_pos.clone(), (1, 1));
        if grid.try_place(output_entity).is_ok() {
            commands.push(Command::Place(PlaceCommand {
                entity_type: self.config.inserter_type.clone(),
                position: output_pos,
                direction: Some(Direction::South),
                recipe: None,
                modules: Vec::new(),
            }));
        }

        Ok(())
    }

    /// Add a line of belts
    fn add_belt_line(
        &self,
        start: &Position,
        count: u32,
        direction: Direction,
        grid: &mut PlacementGrid,
        commands: &mut Vec<Command>,
    ) -> anyhow::Result<()> {
        let mut pos = start.clone();

        for _ in 0..count {
            let belt_entity = PlacedEntity::new(&self.config.belt_type, pos.clone(), (1, 1));
            if grid.try_place(belt_entity).is_ok() {
                commands.push(Command::Place(PlaceCommand {
                    entity_type: self.config.belt_type.clone(),
                    position: pos.clone(),
                    direction: Some(direction),
                    recipe: None,
                    modules: Vec::new(),
                }));
            }

            pos = adjacent_position(&pos, direction);
        }

        Ok(())
    }
}

/// Generate DSL script from commands
pub fn commands_to_script(commands: &[Command]) -> String {
    let mut lines = Vec::new();

    for cmd in commands {
        match cmd {
            Command::Place(place) => {
                let mut line = format!(
                    "{} {} {}",
                    place.entity_type, place.position.x, place.position.y
                );

                if let Some(dir) = &place.direction {
                    line.push_str(&format!(" :{}", dir.to_keyword()));
                }

                if let Some(recipe) = &place.recipe {
                    line.push_str(&format!(" recipe:{}", recipe));
                }

                for module in &place.modules {
                    line.push_str(&format!(" module:{}", module));
                }

                lines.push(line);
            }
            Command::Comment(text) => {
                if !text.is_empty() {
                    lines.push(format!("; {}", text));
                }
            }
            _ => {}
        }
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::{BeltType, Throughput};

    #[test]
    fn test_solve_iron_gear() {
        let solver = GridPlacementSolver::new(PlacementConfig::default());
        let goal = ProductionGoal {
            item: "iron-gear".to_string(),
            throughput: Throughput::ItemsPerSecond(2.0),
        };

        let solution = solver.solve(&goal).unwrap();
        assert!(!solution.commands.is_empty());
        assert!(solution.statistics.machine_count.values().sum::<u32>() > 0);
    }

    #[test]
    fn test_solve_copper_cable() {
        let solver = GridPlacementSolver::new(PlacementConfig::default());
        let goal = ProductionGoal {
            item: "copper-cable".to_string(),
            throughput: Throughput::ItemsPerSecond(10.0),
        };

        let solution = solver.solve(&goal).unwrap();
        assert!(!solution.commands.is_empty());
    }

    #[test]
    fn test_commands_to_script() {
        let commands = vec![
            Command::Place(PlaceCommand {
                entity_type: "belt-blue".to_string(),
                position: Position::new(0, 0),
                direction: Some(Direction::North),
                recipe: None,
                modules: Vec::new(),
            }),
            Command::Place(PlaceCommand {
                entity_type: "assembler-3".to_string(),
                position: Position::new(5, 5),
                direction: Some(Direction::North),
                recipe: Some("iron-gear".to_string()),
                modules: vec!["speed-3".to_string()],
            }),
        ];

        let script = commands_to_script(&commands);
        assert!(script.contains("belt-blue 0 0 :north"));
        assert!(script.contains("assembler-3 5 5 :north recipe:iron-gear module:speed-3"));
    }
}
