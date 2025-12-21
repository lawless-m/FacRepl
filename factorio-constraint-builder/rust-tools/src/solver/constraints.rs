// Constraint definitions for factory layout
// Defines spatial and connectivity constraints for entity placement

use crate::dsl::ast::{Direction, Position};

/// A constraint that must be satisfied during placement
#[derive(Debug, Clone)]
pub enum Constraint {
    /// Entity must not overlap with any other entity
    NoOverlap,
    /// Inserter must be adjacent to both source and destination
    InserterReach { inserter_pos: Position, source_pos: Position, dest_pos: Position },
    /// Belt must connect to adjacent belt in flow direction
    BeltConnection { from: Position, to: Position, direction: Direction },
    /// Power pole must be within range of power network
    PowerConnection { pole_pos: Position, max_wire_distance: u32 },
    /// Entities must fit within bounding box
    WithinBounds { min: Position, max: Position },
    /// Minimum spacing between entities
    MinSpacing { entity_type: String, spacing: u32 },
}

/// Result of checking a constraint
#[derive(Debug, Clone)]
pub enum ConstraintResult {
    Satisfied,
    Violated { reason: String },
}

/// A placed entity in the grid
#[derive(Debug, Clone)]
pub struct PlacedEntity {
    pub entity_type: String,
    pub position: Position,
    pub size: (u32, u32),
    pub direction: Option<Direction>,
}

impl PlacedEntity {
    pub fn new(entity_type: &str, position: Position, size: (u32, u32)) -> Self {
        Self {
            entity_type: entity_type.to_string(),
            position,
            size,
            direction: None,
        }
    }

    pub fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = Some(direction);
        self
    }

    /// Get the bounding box of this entity
    pub fn bounds(&self) -> (Position, Position) {
        let min = self.position.clone();
        let max = Position::new(
            self.position.x + self.size.0 as i32 - 1,
            self.position.y + self.size.1 as i32 - 1,
        );
        (min, max)
    }

    /// Check if this entity overlaps with another
    pub fn overlaps(&self, other: &PlacedEntity) -> bool {
        let (self_min, self_max) = self.bounds();
        let (other_min, other_max) = other.bounds();

        !(self_max.x < other_min.x
            || self_min.x > other_max.x
            || self_max.y < other_min.y
            || self_min.y > other_max.y)
    }

    /// Check if a position is adjacent to this entity
    pub fn is_adjacent(&self, pos: &Position) -> bool {
        let (min, max) = self.bounds();

        // Check if position is exactly 1 tile away from the bounding box
        let dx = if pos.x < min.x {
            min.x - pos.x
        } else if pos.x > max.x {
            pos.x - max.x
        } else {
            0
        };

        let dy = if pos.y < min.y {
            min.y - pos.y
        } else if pos.y > max.y {
            pos.y - max.y
        } else {
            0
        };

        // Adjacent means exactly 1 tile away in one direction, 0 in the other
        (dx == 1 && dy == 0) || (dx == 0 && dy == 1)
    }
}

/// The placement grid that tracks placed entities
#[derive(Debug, Clone)]
pub struct PlacementGrid {
    entities: Vec<PlacedEntity>,
    bounds: Option<(Position, Position)>,
}

impl PlacementGrid {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            bounds: None,
        }
    }

    pub fn with_bounds(min: Position, max: Position) -> Self {
        Self {
            entities: Vec::new(),
            bounds: Some((min, max)),
        }
    }

    /// Try to place an entity, returns error if constraints violated
    pub fn try_place(&mut self, entity: PlacedEntity) -> Result<(), String> {
        // Check bounds
        if let Some((min, max)) = &self.bounds {
            let (e_min, e_max) = entity.bounds();
            if e_min.x < min.x || e_min.y < min.y || e_max.x > max.x || e_max.y > max.y {
                return Err(format!(
                    "Entity {} at ({},{}) is out of bounds",
                    entity.entity_type, entity.position.x, entity.position.y
                ));
            }
        }

        // Check overlaps
        for existing in &self.entities {
            if entity.overlaps(existing) {
                return Err(format!(
                    "Entity {} at ({},{}) overlaps with {} at ({},{})",
                    entity.entity_type, entity.position.x, entity.position.y,
                    existing.entity_type, existing.position.x, existing.position.y
                ));
            }
        }

        self.entities.push(entity);
        Ok(())
    }

    /// Force place an entity (no constraint checking)
    pub fn force_place(&mut self, entity: PlacedEntity) {
        self.entities.push(entity);
    }

    /// Get entity at a position
    pub fn get_at(&self, pos: &Position) -> Option<&PlacedEntity> {
        for entity in &self.entities {
            let (min, max) = entity.bounds();
            if pos.x >= min.x && pos.x <= max.x && pos.y >= min.y && pos.y <= max.y {
                return Some(entity);
            }
        }
        None
    }

    /// Check if a position is occupied
    pub fn is_occupied(&self, pos: &Position) -> bool {
        self.get_at(pos).is_some()
    }

    /// Check if an area is clear
    pub fn is_area_clear(&self, min: &Position, max: &Position) -> bool {
        for x in min.x..=max.x {
            for y in min.y..=max.y {
                if self.is_occupied(&Position::new(x, y)) {
                    return false;
                }
            }
        }
        true
    }

    /// Find first clear position for an entity of given size
    pub fn find_clear_position(&self, size: (u32, u32), start: &Position) -> Option<Position> {
        let search_radius = 100; // Max search distance

        for radius in 0..search_radius {
            for dx in -(radius as i32)..=(radius as i32) {
                for dy in -(radius as i32)..=(radius as i32) {
                    if dx.abs() != radius && dy.abs() != radius {
                        continue; // Only check perimeter
                    }

                    let pos = Position::new(start.x + dx, start.y + dy);
                    let max = Position::new(pos.x + size.0 as i32 - 1, pos.y + size.1 as i32 - 1);

                    if self.is_area_clear(&pos, &max) {
                        return Some(pos);
                    }
                }
            }
        }

        None
    }

    /// Get all placed entities
    pub fn entities(&self) -> &[PlacedEntity] {
        &self.entities
    }

    /// Get the bounding box of all placed entities
    pub fn used_bounds(&self) -> Option<(Position, Position)> {
        if self.entities.is_empty() {
            return None;
        }

        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;

        for entity in &self.entities {
            let (e_min, e_max) = entity.bounds();
            min_x = min_x.min(e_min.x);
            min_y = min_y.min(e_min.y);
            max_x = max_x.max(e_max.x);
            max_y = max_y.max(e_max.y);
        }

        Some((Position::new(min_x, min_y), Position::new(max_x, max_y)))
    }
}

impl Default for PlacementGrid {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate Manhattan distance between two positions
pub fn manhattan_distance(a: &Position, b: &Position) -> u32 {
    ((a.x - b.x).abs() + (a.y - b.y).abs()) as u32
}

/// Check if inserter can reach between two positions
pub fn inserter_can_reach(inserter_pos: &Position, source: &Position, dest: &Position) -> bool {
    let d_source = manhattan_distance(inserter_pos, source);
    let d_dest = manhattan_distance(inserter_pos, dest);

    // Basic inserter can reach 1 tile, long inserter can reach 2
    d_source <= 1 && d_dest <= 1
}

/// Get adjacent position in a direction
pub fn adjacent_position(pos: &Position, direction: Direction) -> Position {
    match direction {
        Direction::North => Position::new(pos.x, pos.y - 1),
        Direction::South => Position::new(pos.x, pos.y + 1),
        Direction::East => Position::new(pos.x + 1, pos.y),
        Direction::West => Position::new(pos.x - 1, pos.y),
        Direction::NorthEast => Position::new(pos.x + 1, pos.y - 1),
        Direction::NorthWest => Position::new(pos.x - 1, pos.y - 1),
        Direction::SouthEast => Position::new(pos.x + 1, pos.y + 1),
        Direction::SouthWest => Position::new(pos.x - 1, pos.y + 1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_overlap() {
        let e1 = PlacedEntity::new("assembler-3", Position::new(0, 0), (3, 3));
        let e2 = PlacedEntity::new("assembler-3", Position::new(2, 2), (3, 3));
        let e3 = PlacedEntity::new("assembler-3", Position::new(5, 5), (3, 3));

        assert!(e1.overlaps(&e2));
        assert!(!e1.overlaps(&e3));
    }

    #[test]
    fn test_grid_placement() {
        let mut grid = PlacementGrid::new();

        let e1 = PlacedEntity::new("belt-blue", Position::new(0, 0), (1, 1));
        assert!(grid.try_place(e1).is_ok());

        let e2 = PlacedEntity::new("belt-blue", Position::new(0, 0), (1, 1));
        assert!(grid.try_place(e2).is_err()); // Overlap

        let e3 = PlacedEntity::new("belt-blue", Position::new(1, 0), (1, 1));
        assert!(grid.try_place(e3).is_ok());
    }

    #[test]
    fn test_find_clear_position() {
        let mut grid = PlacementGrid::new();
        grid.force_place(PlacedEntity::new("test", Position::new(0, 0), (3, 3)));

        let pos = grid.find_clear_position((3, 3), &Position::new(0, 0));
        assert!(pos.is_some());
        let found = pos.unwrap();
        // The found position should not overlap with the original entity
        // Original entity is at (0,0) to (2,2), so new 3x3 entity
        // can start at x >= 3 or y >= 3, or at negative positions
        let entity = PlacedEntity::new("test2", found.clone(), (3, 3));
        let original = PlacedEntity::new("test", Position::new(0, 0), (3, 3));
        assert!(!entity.overlaps(&original));
    }

    #[test]
    fn test_manhattan_distance() {
        let a = Position::new(0, 0);
        let b = Position::new(3, 4);
        assert_eq!(manhattan_distance(&a, &b), 7);
    }

    #[test]
    fn test_adjacent_position() {
        let p = Position::new(5, 5);
        assert_eq!(adjacent_position(&p, Direction::North), Position::new(5, 4));
        assert_eq!(adjacent_position(&p, Direction::South), Position::new(5, 6));
        assert_eq!(adjacent_position(&p, Direction::East), Position::new(6, 5));
        assert_eq!(adjacent_position(&p, Direction::West), Position::new(4, 5));
    }
}
