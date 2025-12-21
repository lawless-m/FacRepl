// Recipe definitions and calculations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ingredient {
    pub name: String,
    pub amount: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeInfo {
    pub name: String,
    pub ingredients: Vec<Ingredient>,
    pub products: Vec<Ingredient>,
    pub crafting_time: f64,
    pub category: String,
}

impl RecipeInfo {
    pub fn new(name: &str, crafting_time: f64, category: &str) -> Self {
        Self {
            name: name.to_string(),
            ingredients: Vec::new(),
            products: Vec::new(),
            crafting_time,
            category: category.to_string(),
        }
    }
    
    pub fn add_ingredient(mut self, name: &str, amount: f64) -> Self {
        self.ingredients.push(Ingredient {
            name: name.to_string(),
            amount,
        });
        self
    }
    
    pub fn add_product(mut self, name: &str, amount: f64) -> Self {
        self.products.push(Ingredient {
            name: name.to_string(),
            amount,
        });
        self
    }
    
    /// Calculate items per second for a given crafting speed
    pub fn items_per_second(&self, crafting_speed: f64) -> f64 {
        let products = self.products.first().map(|p| p.amount).unwrap_or(1.0);
        products * crafting_speed / self.crafting_time
    }
}

#[derive(Debug, Clone)]
pub struct Recipe {
    recipes: HashMap<String, RecipeInfo>,
}

impl Recipe {
    pub fn new() -> Self {
        let mut recipes = HashMap::new();
        
        // Basic recipes
        recipes.insert("iron-gear".to_string(),
            RecipeInfo::new("iron-gear-wheel", 0.5, "crafting")
                .add_ingredient("iron-plate", 2.0)
                .add_product("iron-gear-wheel", 1.0));
        
        recipes.insert("copper-cable".to_string(),
            RecipeInfo::new("copper-cable", 0.5, "crafting")
                .add_ingredient("copper-plate", 1.0)
                .add_product("copper-cable", 2.0));
        
        recipes.insert("green-circuit".to_string(),
            RecipeInfo::new("electronic-circuit", 0.5, "crafting")
                .add_ingredient("iron-plate", 1.0)
                .add_ingredient("copper-cable", 3.0)
                .add_product("electronic-circuit", 1.0));
        
        recipes.insert("red-circuit".to_string(),
            RecipeInfo::new("advanced-circuit", 6.0, "crafting")
                .add_ingredient("electronic-circuit", 2.0)
                .add_ingredient("plastic-bar", 2.0)
                .add_ingredient("copper-cable", 4.0)
                .add_product("advanced-circuit", 1.0));
        
        recipes.insert("blue-circuit".to_string(),
            RecipeInfo::new("processing-unit", 10.0, "crafting")
                .add_ingredient("electronic-circuit", 20.0)
                .add_ingredient("advanced-circuit", 2.0)
                .add_ingredient("sulfuric-acid", 5.0)
                .add_product("processing-unit", 1.0));
        
        // Science packs
        recipes.insert("automation-science-pack".to_string(),
            RecipeInfo::new("automation-science-pack", 5.0, "crafting")
                .add_ingredient("copper-plate", 1.0)
                .add_ingredient("iron-gear-wheel", 1.0)
                .add_product("automation-science-pack", 1.0));
        
        recipes.insert("logistic-science-pack".to_string(),
            RecipeInfo::new("logistic-science-pack", 6.0, "crafting")
                .add_ingredient("inserter", 1.0)
                .add_ingredient("transport-belt", 1.0)
                .add_product("logistic-science-pack", 1.0));
        
        Self { recipes }
    }
    
    pub fn get(&self, name: &str) -> Option<&RecipeInfo> {
        self.recipes.get(name)
    }
    
    pub fn list_recipes(&self) -> Vec<&str> {
        self.recipes.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for Recipe {
    fn default() -> Self {
        Self::new()
    }
}
