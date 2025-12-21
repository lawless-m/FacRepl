// Entity Types and Information

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityCategory {
    Assembler,
    Furnace,
    Belt,
    Inserter,
    PowerPole,
    Pipe,
    Storage,
    Splitter,
    UndergroundBelt,
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityInfo {
    pub name: String,
    pub category: EntityCategory,
    pub size: (u32, u32),
    pub crafting_speed: Option<f64>,
    pub module_slots: Option<u32>,
    pub power_usage: Option<f64>,
}

impl EntityInfo {
    pub fn new(name: &str, category: EntityCategory, size: (u32, u32)) -> Self {
        Self {
            name: name.to_string(),
            category,
            size,
            crafting_speed: None,
            module_slots: None,
            power_usage: None,
        }
    }
    
    pub fn with_crafting_speed(mut self, speed: f64) -> Self {
        self.crafting_speed = Some(speed);
        self
    }
    
    pub fn with_module_slots(mut self, slots: u32) -> Self {
        self.module_slots = Some(slots);
        self
    }
}

#[derive(Debug, Clone)]
pub struct EntityType {
    entities: HashMap<String, EntityInfo>,
}

impl EntityType {
    pub fn new() -> Self {
        let mut entities = HashMap::new();
        
        // Belts
        entities.insert("belt-yellow".to_string(), 
            EntityInfo::new("transport-belt", EntityCategory::Belt, (1, 1)));
        entities.insert("belt-red".to_string(), 
            EntityInfo::new("fast-transport-belt", EntityCategory::Belt, (1, 1)));
        entities.insert("belt-blue".to_string(), 
            EntityInfo::new("express-transport-belt", EntityCategory::Belt, (1, 1)));
        
        // Underground belts
        entities.insert("underground-belt-yellow".to_string(), 
            EntityInfo::new("underground-belt", EntityCategory::UndergroundBelt, (1, 1)));
        entities.insert("underground-belt-red".to_string(), 
            EntityInfo::new("fast-underground-belt", EntityCategory::UndergroundBelt, (1, 1)));
        entities.insert("underground-belt-blue".to_string(), 
            EntityInfo::new("express-underground-belt", EntityCategory::UndergroundBelt, (1, 1)));
        
        // Splitters
        entities.insert("splitter-yellow".to_string(), 
            EntityInfo::new("splitter", EntityCategory::Splitter, (2, 1)));
        entities.insert("splitter-red".to_string(), 
            EntityInfo::new("fast-splitter", EntityCategory::Splitter, (2, 1)));
        entities.insert("splitter-blue".to_string(), 
            EntityInfo::new("express-splitter", EntityCategory::Splitter, (2, 1)));
        
        // Inserters
        entities.insert("inserter-burner".to_string(),
            EntityInfo::new("burner-inserter", EntityCategory::Inserter, (1, 1)));
        entities.insert("inserter-basic".to_string(),
            EntityInfo::new("inserter", EntityCategory::Inserter, (1, 1)));
        entities.insert("inserter-fast".to_string(),
            EntityInfo::new("fast-inserter", EntityCategory::Inserter, (1, 1)));
        entities.insert("inserter-long".to_string(),
            EntityInfo::new("long-handed-inserter", EntityCategory::Inserter, (1, 1)));
        entities.insert("inserter-filter".to_string(),
            EntityInfo::new("filter-inserter", EntityCategory::Inserter, (1, 1)));
        entities.insert("inserter-stack".to_string(),
            EntityInfo::new("stack-inserter", EntityCategory::Inserter, (1, 1)));
        entities.insert("inserter-stack-filter".to_string(),
            EntityInfo::new("stack-filter-inserter", EntityCategory::Inserter, (1, 1)));
        
        // Assemblers
        entities.insert("assembler-1".to_string(), 
            EntityInfo::new("assembling-machine-1", EntityCategory::Assembler, (3, 3))
                .with_crafting_speed(0.5));
        entities.insert("assembler-2".to_string(), 
            EntityInfo::new("assembling-machine-2", EntityCategory::Assembler, (3, 3))
                .with_crafting_speed(0.75)
                .with_module_slots(2));
        entities.insert("assembler-3".to_string(), 
            EntityInfo::new("assembling-machine-3", EntityCategory::Assembler, (3, 3))
                .with_crafting_speed(1.25)
                .with_module_slots(4));
        
        // Power poles
        entities.insert("power-pole-small".to_string(), 
            EntityInfo::new("small-electric-pole", EntityCategory::PowerPole, (1, 1)));
        entities.insert("power-pole-medium".to_string(), 
            EntityInfo::new("medium-electric-pole", EntityCategory::PowerPole, (1, 1)));
        entities.insert("power-pole-big".to_string(), 
            EntityInfo::new("big-electric-pole", EntityCategory::PowerPole, (2, 2)));
        
        // Pipes
        entities.insert("pipe".to_string(), 
            EntityInfo::new("pipe", EntityCategory::Pipe, (1, 1)));
        entities.insert("pump".to_string(), 
            EntityInfo::new("pump", EntityCategory::Pipe, (1, 2)));
        entities.insert("storage-tank".to_string(), 
            EntityInfo::new("storage-tank", EntityCategory::Storage, (3, 3)));
        
        Self { entities }
    }
    
    pub fn get(&self, name: &str) -> Option<&EntityInfo> {
        self.entities.get(name)
    }
    
    pub fn factorio_name(&self, dsl_name: &str) -> Option<&str> {
        self.entities.get(dsl_name).map(|e| e.name.as_str())
    }
    
    pub fn list_entities(&self) -> Vec<&str> {
        self.entities.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for EntityType {
    fn default() -> Self {
        Self::new()
    }
}
