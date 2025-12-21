# Factorio Constraint-Based Machine Builder

## Project Overview

An experimental system for generating and placing Factorio factory layouts using constraint satisfaction. The system allows specifying high-level goals (e.g., "produce 1 blue belt of green circuits") and automatically generates valid factory layouts that satisfy the constraints.

### Key Features
- **In-game placement**: Real-time factory building via mod integration
- **DSL-based control**: Simple, readable command language for entity placement
- **Constraint solver**: Generates valid layouts from throughput requirements
- **Multiple interfaces**: REPL, scripts, or programmatic solver output
- **Visual feedback**: Watch the solver work in real-time

### Goals
- **Satisfaction over optimization**: Find *a* valid solution, not the *best* solution
- **Throughput-focused**: Respect specified production rates (e.g., "1 blue belt")
- **Experimental platform**: Easy to iterate on solver approaches
- **Fun to watch**: Real-time in-game building is more engaging than blueprint generation

## Architecture

```
┌─────────────────────┐
│ Constraint Solver   │────> Outputs DSL scripts
│ (Rust)              │
└─────────────────────┘

┌─────────────────────┐
│ DSL Script Files    │────> Can be loaded/executed
└─────────────────────┘

┌─────────────────────┐
│ REPL                │────> Interactive DSL execution
│ (Rust)              │
└─────────────────────┘
          │
          │  All feed into DSL Executor
          v
┌─────────────────────┐
│ DSL Executor        │────> Parses and executes DSL commands
│ (Rust)              │
└─────────────────────┘
          │
          v
┌─────────────────────┐
│ RCON Bridge         │────> Communicates with Factorio
│ (Rust)              │
└─────────────────────┘
          │
          v
┌─────────────────────┐
│ Factorio Mod        │────> Places entities in-game
│ (Lua)               │
└─────────────────────┘
```

### Component Responsibilities

**Factorio Mod (Lua)**
- Registers RCON command handlers
- Places/removes entities based on commands
- Validates placement (can we build here?)
- Queries game state (what's at this location?)
- Provides visual feedback (highlighting, error indicators)
- Returns success/failure responses

**RCON Bridge (Rust)**
- Manages connection to Factorio's RCON server
- Sends Lua commands
- Receives and parses responses
- Handles connection errors and retries
- Provides async interface for commands

**DSL Executor (Rust)**
- Parses DSL syntax into commands
- Validates command structure
- Executes commands via RCON bridge
- Handles errors and retries
- Maintains execution state

**REPL (Rust)**
- Interactive command-line interface
- Command history and editing
- Tab completion
- Help system
- Load/execute script files

**Constraint Solver (Rust)**
- Takes high-level production goals
- Calculates required machines and ratios
- Solves placement constraints
- Generates DSL scripts as output
- Multiple solver strategies available

## DSL Specification

### Design Principles
- **Keyword-oriented**: Keywords start with `:` (e.g., `:north`, `:blue`)
- **Prefix notation**: Command comes first
- **Readable**: Full words preferred over abbreviations (except directions)
- **Consistent**: Predictable patterns across commands
- **Extensible**: Easy to add new commands

### Core Syntax

#### Entity Placement
```
<entity-type> <x> <y> <:direction>
<entity-type> <x> <y> <:direction> <options>
```

**Examples:**
```
belt-yellow 10 20 :n
belt-red 10 21 :n
belt-blue 10 22 :n

inserter-basic 15 20 :e
inserter-fast 16 20 :e
inserter-stack 17 20 :e
inserter-long 18 20 :w

assembler-1 10 10 recipe:iron-gear
assembler-2 15 15 recipe:green-circuit
assembler-3 20 20 recipe:blue-circuit

power-pole-small 10 10
power-pole-medium 20 20
power-pole-big 30 30

splitter-blue 25 25 :n
underground-belt-blue 30 30 :n
underground-belt-blue 35 30 :n

pipe 40 40
pump 45 45 :e
storage-tank 50 50
```

#### Direction Keywords
```
Full form:       Short alias:
:north           :n
:south           :s
:east            :e
:west            :w
:northeast       :ne
:northwest       :nw
:southeast       :se
:southwest       :sw
```

#### Entity Naming Convention
Uses Factorio's internal naming with hyphens:
- `belt-yellow`, `belt-red`, `belt-blue`
- `inserter-burner`, `inserter-basic`, `inserter-fast`, `inserter-stack`, `inserter-long`
- `assembler-1`, `assembler-2`, `assembler-3`
- `splitter-yellow`, `splitter-red`, `splitter-blue`
- `underground-belt-yellow`, `underground-belt-red`, `underground-belt-blue`
- `power-pole-small`, `power-pole-medium`, `power-pole-big`

#### Recipe and Module Specification
```
recipe:<recipe-name>
module:<module-name>

Examples:
assembler-3 10 10 recipe:green-circuit
assembler-3 15 15 recipe:green-circuit module:speed-3 module:speed-3
```

#### Query Commands
```
what-at <x> <y>
can-place <entity-type> <x> <y>
area <x1> <y1> <x2> <y2>
```

**Examples:**
```
what-at 10 20
can-place belt-blue 15 25
area 0 0 50 50
```

#### Area Operations
```
clear <x1> <y1> <x2> <y2>
clear-entity <entity-type> <x1> <y1> <x2> <y2>
```

**Examples:**
```
clear 0 0 20 20
clear-entity belt-blue 10 10 30 30
```

#### State Management
```
undo
undo <count>
save-state :<name>
load-state :<name>
```

**Examples:**
```
undo
undo 5
save-state :checkpoint1
load-state :checkpoint1
```

#### Higher-Level Commands (Future)
```
connect <x1> <y1> to <x2> <y2> with <entity-type>
fill <entity-type> <x1> <y1> to <x2> <y2> <:direction>
```

**Examples:**
```
connect 10 20 to 30 20 with belt-blue
fill belt-blue 0 0 to 10 0 :e
```

#### Comments
```
; This is a comment
;; This is also a comment
```

### Script File Format

DSL scripts are plain text files with `.fcb` extension (Factorio Constraint Builder):

```
; green-circuit-setup.fcb
; Basic green circuit production line

; Power infrastructure
power-pole-medium 0 0
power-pole-medium 10 0

; Copper wire production
assembler-2 5 5 recipe:copper-cable
inserter-fast 4 5 :w
inserter-fast 6 5 :e

; Green circuit production
assembler-2 10 5 recipe:green-circuit
inserter-fast 9 5 :w
inserter-fast 11 5 :e

; Belts for input/output
belt-blue 0 5 :e
belt-blue 15 5 :e
```

### REPL Commands

In addition to DSL commands, the REPL supports:

```
:help                    ; Show help
:help <command>          ; Help for specific command
:load <file>            ; Load and execute script file
:save <file>            ; Save command history to file
:quit or :q             ; Exit REPL
:clear                  ; Clear screen
:history                ; Show command history
:entities               ; List available entity types
```

## Factorio Mod Design

### Mod Structure
```
factorio-constraint-builder/
├── info.json
├── control.lua
├── placement/
│   ├── validator.lua
│   ├── executor.lua
│   └── query.lua
├── feedback/
│   └── visual.lua
└── utils/
    └── response.lua
```

### Core Functionality

#### Command Registration
The mod registers custom commands that can be called via RCON:

```lua
-- /fcb-place entity-name x y direction [options]
-- /fcb-remove x y
-- /fcb-query x y
-- /fcb-can-place entity-name x y
-- /fcb-clear x1 y1 x2 y2
```

#### Entity Placement Logic

**Validation Steps:**
1. Check if coordinates are valid
2. Check if entity type exists
3. Check if area is clear (or can be mined)
4. Check if player has items (optional - could use cheat mode)
5. Verify connections would be valid (belt to belt, inserter reach, etc.)

**Placement Process:**
1. Validate placement
2. Clear area if needed
3. Create entity with specified parameters
4. Set recipe/modules if applicable
5. Return success/failure with details

**Response Format (JSON-like):**
```lua
{
  success = true/false,
  entity = entity-name,
  position = {x, y},
  error = "error message if failed",
  details = "additional information"
}
```

#### Query Capabilities

**what-at:**
- Returns entity name, direction, recipe, modules
- Returns nil if nothing at location

**can-place:**
- Returns true/false
- Returns reason if false (blocked, invalid, etc.)

**area:**
- Returns list of all entities in area
- Includes positions, types, configurations

#### Visual Feedback

**Highlight System:**
- Green highlight: Successful placement
- Red highlight: Failed placement
- Yellow highlight: Next planned placement
- Blue highlight: Query result

**Duration:** 2-5 seconds for feedback, persistent for planning

#### Error Handling
- Invalid coordinates → error response
- Invalid entity type → error response with suggestions
- Blocked placement → error response with blocking entity info
- Connection issues → detailed connection failure info

### RCON Integration

**Factorio RCON Setup:**
1. Enable RCON in server settings or command line:
   ```
   --rcon-port 27015
   --rcon-password your-password
   ```

2. For local testing, can run Factorio in headless mode or connect to running game

**Mod Command Format:**
Commands sent via RCON execute Lua code directly:
```
/silent-command remote.call("fcb", "place", {
  entity = "transport-belt",
  position = {x = 10, y = 20},
  direction = defines.direction.north
})
```

The mod provides a `remote.add_interface` for cleaner calls.

## Rust Components

### Project Structure
```
factorio-constraint-builder/
├── Cargo.toml
├── src/
│   ├── main.rs              ; Entry point, CLI argument parsing
│   ├── lib.rs               ; Library exports
│   ├── rcon/
│   │   ├── mod.rs           ; RCON bridge module
│   │   ├── client.rs        ; RCON client implementation
│   │   └── command.rs       ; Command builders
│   ├── dsl/
│   │   ├── mod.rs           ; DSL module
│   │   ├── parser.rs        ; DSL parser
│   │   ├── ast.rs           ; Abstract syntax tree
│   │   ├── executor.rs      ; DSL executor
│   │   └── validator.rs     ; Command validation
│   ├── repl/
│   │   ├── mod.rs           ; REPL module
│   │   ├── interface.rs     ; Interactive interface
│   │   └── completion.rs    ; Tab completion
│   ├── solver/
│   │   ├── mod.rs           ; Solver module
│   │   ├── constraints.rs   ; Constraint definitions
│   │   ├── ratios.rs        ; Production ratio calculations
│   │   ├── placement.rs     ; Spatial placement solver
│   │   └── generator.rs     ; DSL script generation
│   └── entities/
│       ├── mod.rs           ; Entity definitions
│       ├── types.rs         ; Entity type system
│       └── recipes.rs       ; Recipe database
└── tests/
    ├── integration.rs
    └── dsl_parsing.rs
```

### Dependencies (Cargo.toml)

```toml
[package]
name = "factorio-constraint-builder"
version = "0.1.0"
edition = "2021"

[dependencies]
# RCON client
rcon = "0.5"
tokio = { version = "1", features = ["full"] }

# DSL parsing
nom = "7"
pest = "2"  # Alternative parser if preferred

# REPL
rustyline = "13"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Error handling
anyhow = "1"
thiserror = "1"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# CLI
clap = { version = "4", features = ["derive"] }

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
```

### RCON Bridge

**Core Types:**
```rust
pub struct RconBridge {
    client: RconClient,
    config: RconConfig,
}

pub struct RconConfig {
    pub host: String,
    pub port: u16,
    pub password: String,
    pub timeout: Duration,
}

pub enum RconResponse {
    Success { data: String },
    Error { message: String },
}
```

**Key Methods:**
```rust
impl RconBridge {
    pub async fn connect(config: RconConfig) -> Result<Self>;
    pub async fn execute_command(&mut self, cmd: &str) -> Result<RconResponse>;
    pub async fn place_entity(&mut self, placement: EntityPlacement) -> Result<PlacementResult>;
    pub async fn query_position(&mut self, pos: Position) -> Result<QueryResult>;
    pub async fn clear_area(&mut self, area: Area) -> Result<ClearResult>;
    pub fn disconnect(&mut self) -> Result<()>;
}
```

### DSL Parser

**AST Types:**
```rust
pub enum Command {
    Place(PlaceCommand),
    Query(QueryCommand),
    Clear(ClearCommand),
    Undo(UndoCommand),
    SaveState(String),
    LoadState(String),
}

pub struct PlaceCommand {
    pub entity_type: EntityType,
    pub position: Position,
    pub direction: Direction,
    pub options: PlaceOptions,
}

pub struct Position {
    pub x: i32,
    pub y: i32,
}

pub enum Direction {
    North, South, East, West,
    NorthEast, NorthWest, SouthEast, SouthWest,
}

pub struct PlaceOptions {
    pub recipe: Option<String>,
    pub modules: Vec<String>,
}
```

**Parser Interface:**
```rust
pub struct DslParser;

impl DslParser {
    pub fn parse_line(input: &str) -> Result<Command, ParseError>;
    pub fn parse_script(input: &str) -> Result<Vec<Command>, ParseError>;
}
```

**Parsing Strategy:**
- Use `nom` for combinator-based parsing
- Tokenize on whitespace
- Keywords identified by `:` prefix
- Position as command discriminator
- Simple recursive descent for nested structures

### DSL Executor

**Core Interface:**
```rust
pub struct DslExecutor {
    rcon: RconBridge,
    state: ExecutionState,
}

pub struct ExecutionState {
    pub history: Vec<Command>,
    pub saved_states: HashMap<String, Vec<Command>>,
}

impl DslExecutor {
    pub async fn execute(&mut self, cmd: Command) -> Result<ExecutionResult>;
    pub async fn execute_script(&mut self, commands: Vec<Command>) -> Result<Vec<ExecutionResult>>;
    pub async fn undo(&mut self, count: usize) -> Result<()>;
    pub fn save_state(&mut self, name: String);
    pub async fn load_state(&mut self, name: String) -> Result<()>;
}
```

### REPL

**Interface:**
```rust
pub struct Repl {
    executor: DslExecutor,
    history: Vec<String>,
    config: ReplConfig,
}

pub struct ReplConfig {
    pub prompt: String,
    pub history_file: Option<PathBuf>,
    pub enable_completion: bool,
}

impl Repl {
    pub async fn run(&mut self) -> Result<()>;
    pub fn load_script(&mut self, path: &Path) -> Result<Vec<Command>>;
    pub fn save_history(&self, path: &Path) -> Result<()>;
}
```

**Features:**
- Command history (up/down arrows)
- Tab completion for entity types and keywords
- Multi-line editing for complex commands
- Syntax highlighting (optional, using colored output)
- Help system with command examples

### Constraint Solver

**Problem Definition:**
```rust
pub struct ProductionGoal {
    pub item: String,
    pub throughput: Throughput,
}

pub enum Throughput {
    BeltFull(BeltType),      // e.g., "1 blue belt"
    ItemsPerSecond(f64),
    ItemsPerMinute(f64),
}

pub struct ConstraintSet {
    pub goal: ProductionGoal,
    pub available_recipes: Vec<String>,
    pub allowed_entities: Vec<EntityType>,
    pub layout_constraints: LayoutConstraints,
}

pub struct LayoutConstraints {
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub tileable: bool,
    pub beacon_coverage: bool,
}
```

**Solver Interface:**
```rust
pub trait ConstraintSolver {
    fn solve(&self, constraints: ConstraintSet) -> Result<Solution>;
}

pub struct Solution {
    pub entities: Vec<EntityPlacement>,
    pub script: Vec<Command>,
    pub statistics: SolutionStats,
}

pub struct SolutionStats {
    pub machine_count: HashMap<EntityType, u32>,
    pub area_used: u32,
    pub power_required: f64,
}
```

**Solver Strategies:**

**1. Backtracking Solver (Initial Implementation):**
- Calculate required machines from ratios
- Place machines in grid pattern
- Backtrack if connections fail
- Simple but effective for small layouts

**2. Greedy Placement:**
- Place machines in order of dependency
- Use heuristics for positioning (minimize distance)
- Fast but may not find solutions

**3. Constraint Propagation:**
- Define constraints (inserter reach, belt connections, etc.)
- Propagate constraints to reduce search space
- Search for valid assignments

**4. Random Search with Validation:**
- Randomly place entities
- Validate all constraints
- Keep valid solutions, retry on failure
- Good for exploration

### Entity System

**Entity Database:**
```rust
pub struct EntityDatabase {
    entities: HashMap<String, EntityInfo>,
    recipes: HashMap<String, RecipeInfo>,
}

pub struct EntityInfo {
    pub name: String,
    pub size: (u32, u32),
    pub entity_type: EntityCategory,
    pub crafting_speed: Option<f64>,
    pub module_slots: Option<u32>,
}

pub enum EntityCategory {
    Assembler,
    Furnace,
    Belt,
    Inserter,
    PowerPole,
    Pipe,
    Storage,
}

pub struct RecipeInfo {
    pub name: String,
    pub ingredients: Vec<(String, f64)>,
    pub products: Vec<(String, f64)>,
    pub crafting_time: f64,
    pub category: String,
}
```

**Data Loading:**
- Load from JSON files (extracted from Factorio data)
- Or hardcode common entities for MVP
- Extensible to full game data

## Communication Protocol

### RCON Command Format

All commands sent via RCON execute Lua through the mod's remote interface:

**Placement Command:**
```lua
/silent-command remote.call("fcb", "place", {
  entity = "transport-belt",
  position = {x = 10, y = 20},
  direction = 0,  -- defines.direction.north
  recipe = "green-circuit",  -- optional
  modules = {"speed-module-3", "speed-module-3"}  -- optional
})
```

**Query Command:**
```lua
/silent-command remote.call("fcb", "query", {
  position = {x = 10, y = 20}
})
```

**Clear Command:**
```lua
/silent-command remote.call("fcb", "clear", {
  area = {
    left_top = {x = 0, y = 0},
    right_bottom = {x = 20, y = 20}
  }
})
```

### Response Format

Mod returns JSON-formatted strings via RCON:

**Success Response:**
```json
{
  "success": true,
  "action": "place",
  "entity": "transport-belt",
  "position": {"x": 10, "y": 20},
  "direction": 0
}
```

**Error Response:**
```json
{
  "success": false,
  "action": "place",
  "error": "blocked",
  "message": "Cannot place entity: position occupied by stone-furnace",
  "blocking_entity": {
    "name": "stone-furnace",
    "position": {"x": 10, "y": 20}
  }
}
```

**Query Response:**
```json
{
  "success": true,
  "action": "query",
  "position": {"x": 10, "y": 20},
  "entity": {
    "name": "assembling-machine-2",
    "direction": 0,
    "recipe": "green-circuit",
    "modules": ["speed-module-3", "speed-module-3"]
  }
}
```

## Development Phases

### Phase 1: Foundation (RCON + Basic Mod)
**Goal:** Establish communication between Rust and Factorio

**Tasks:**
1. Create minimal Factorio mod
   - Register remote interface
   - Implement single placement command
   - Basic error handling
2. Implement Rust RCON bridge
   - Connect to Factorio RCON
   - Send test commands
   - Parse responses
3. Test end-to-end placement
   - Place a single belt from Rust
   - Verify in-game

**Success Criteria:** Can place a transport belt from Rust code

### Phase 2: DSL Parser + Executor
**Goal:** Create DSL language and execution engine

**Tasks:**
1. Design and implement DSL parser
   - Parse placement commands
   - Handle directions and positions
   - Support comments
2. Implement DSL executor
   - Translate DSL to RCON commands
   - Handle responses
   - Basic error reporting
3. Create test scripts
   - Simple belt lines
   - Basic assembler setups

**Success Criteria:** Can execute DSL script to build simple factory

### Phase 3: REPL Interface
**Goal:** Interactive command-line tool

**Tasks:**
1. Implement REPL core
   - Read-eval-print loop
   - Command history
   - Basic help system
2. Add convenience features
   - Tab completion
   - Command editing
   - Script loading
3. Polish user experience
   - Better error messages
   - Helpful prompts
   - Examples in help

**Success Criteria:** Can interactively build factory from command line

### Phase 4: Enhanced Mod Capabilities
**Goal:** Full-featured mod with validation and feedback

**Tasks:**
1. Expand mod commands
   - Query commands
   - Area operations
   - Validation checks
2. Visual feedback system
   - Highlight placements
   - Show errors visually
   - Planning overlays
3. Robust error handling
   - Detailed error messages
   - Suggestions for fixes
   - Graceful degradation

**Success Criteria:** Mod provides rich feedback and handles edge cases

### Phase 5: Basic Constraint Solver
**Goal:** Generate simple factory layouts automatically

**Tasks:**
1. Implement ratio calculator
   - Parse recipe data
   - Calculate machine requirements
   - Handle assembler speeds/modules
2. Simple placement algorithm
   - Grid-based layout
   - Connect with belts
   - Add inserters
3. Generate DSL output
   - Convert solution to DSL script
   - Validate generated script
   - Test in-game

**Success Criteria:** Can generate and build a working green circuit setup

### Phase 6: Advanced Solver Features
**Goal:** More sophisticated constraint solving

**Tasks:**
1. Implement multiple solver strategies
   - Backtracking
   - Constraint propagation
   - Greedy heuristics
2. Layout optimization
   - Minimize footprint
   - Beacon coverage
   - Belt efficiency
3. Complex production chains
   - Multi-step recipes
   - Shared resources
   - Byproduct handling

**Success Criteria:** Can solve complex production scenarios

### Phase 7: Polish and Extensions
**Goal:** Refinement and additional features

**Tasks:**
1. Performance optimization
   - Batch RCON commands
   - Parallel placement
   - Caching
2. Additional DSL features
   - Variables
   - Loops
   - Conditionals
3. Documentation and examples
   - Tutorial scripts
   - Common patterns
   - Troubleshooting guide

**Success Criteria:** Production-ready tool with good documentation

## Testing Strategy

### Unit Tests

**DSL Parser:**
- Test individual command parsing
- Test error cases
- Test script parsing with comments

**Entity System:**
- Test ratio calculations
- Test recipe lookups
- Test entity property queries

**Solver:**
- Test constraint validation
- Test simple layouts
- Test edge cases (no solution exists)

### Integration Tests

**RCON Communication:**
- Test connection handling
- Test command execution
- Test error responses
- Test reconnection logic

**End-to-End:**
- Load script, execute, verify in-game
- Test undo/redo functionality
- Test state save/load

### Manual Testing

**In-Game Verification:**
- Visual inspection of layouts
- Verify entity configurations
- Test production rates
- Check edge cases (terrain, existing buildings)

**REPL Usability:**
- Interactive session testing
- Error message clarity
- Help system usefulness
- Tab completion accuracy

## Future Extensions

### Advanced Features
- **Blueprint integration**: Generate both DSL and blueprints
- **Train networks**: Constraint-based train station placement
- **Megabase optimization**: UPS-conscious layouts
- **Circuit networks**: Automated circuit logic generation
- **Defensive layouts**: Wall and turret placement
- **Mall builders**: Automated mall generation

### Solver Improvements
- **Machine learning**: Learn from successful layouts
- **Genetic algorithms**: Evolve optimal solutions
- **Multi-objective optimization**: Balance multiple goals
- **Template library**: Reusable patterns
- **Parameterized designs**: Configurable templates

### User Interface
- **Web interface**: Browser-based control
- **Visual editor**: Drag-and-drop DSL generation
- **Preview mode**: Show planned layout before building
- **Animation**: Visualize solver progress
- **Statistics dashboard**: Production metrics

### Integration
- **Factorio data extraction**: Auto-generate entity database
- **Mod compatibility**: Work with popular mods
- **Multiplayer support**: Collaborative building
- **Save game analysis**: Optimize existing factories
- **Blueprint library**: Share solutions

## Getting Started

### Prerequisites
- Factorio (stable version)
- Rust toolchain (1.70+)
- Basic familiarity with command line

### Initial Setup

1. **Install Rust:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Enable Factorio RCON:**
   - Add to Factorio startup:
     ```
     --rcon-port 27015
     --rcon-password yourpassword
     ```
   - Or configure in server settings

3. **Install the Mod:**
   - Copy mod to Factorio mods directory
   - Enable in-game

4. **Build Rust Tools:**
   ```bash
   cargo build --release
   ```

5. **Test Connection:**
   ```bash
   ./target/release/fcb-repl --host localhost --port 27015 --password yourpassword
   ```

### First Commands

Try these in the REPL:
```
; Place a single belt
belt-yellow 0 0 :n

; Place an assembler with recipe
assembler-2 10 10 recipe:iron-gear

; Query what's at a position
what-at 0 0

; Clear an area
clear 0 0 20 20
```

### Example Workflow

1. **Start Factorio** with RCON enabled
2. **Start REPL** and connect
3. **Load a script** to build initial layout
4. **Interactively adjust** using REPL commands
5. **Save state** for later modification
6. **Run solver** to generate new sections
7. **Execute solver output** to build automatically

## Notes and Considerations

### Limitations
- **RCON latency**: Commands have network overhead
- **Game tick rate**: Placement speed limited by game updates
- **Mod complexity**: Lua API has learning curve
- **Solver complexity**: Some layouts may be unsolvable
- **Resource requirements**: Large layouts need significant computation

### Design Decisions
- **DSL over API**: DSL provides human-readable intermediate format
- **Rust over Python**: Performance, type safety, tooling
- **RCON over mod API**: Cleaner separation, external control
- **Satisfaction over optimization**: Simpler problem, faster results
- **In-game over blueprints**: More engaging, better feedback

### Performance Considerations
- Batch RCON commands where possible
- Cache entity database queries
- Parallelize independent placements
- Use async I/O for RCON communication
- Consider command queue with rate limiting

### Error Handling Philosophy
- Fail fast on parsing errors
- Retry on transient RCON errors
- Provide actionable error messages
- Log all errors for debugging
- Never silently ignore failures

## References

### Factorio Resources
- Mod API: https://lua-api.factorio.com/
- RCON protocol: https://wiki.factorio.com/RCON
- Blueprint format: https://wiki.factorio.com/Blueprint_string_format
- Recipe data: Extract from game or use community tools

### Rust Crates
- RCON client: https://crates.io/crates/rcon
- Parser: https://crates.io/crates/nom
- REPL: https://crates.io/crates/rustyline
- Async runtime: https://tokio.rs/

### Solver Techniques
- Constraint satisfaction: Classic AI textbooks
- Factorio ratios: Community calculators and tools
- Layout algorithms: Graph theory, packing problems

---

## Scryer Prolog CSP Architecture

### Overview

The constraint solver uses [Scryer Prolog](https://github.com/mthom/scryer-prolog) as the backend for constraint satisfaction. Scryer Prolog is a modern ISO Prolog implementation written in Rust, making it an ideal choice for integration with our Rust codebase.

**Why Scryer Prolog?**
- **Native Rust integration**: Available as a crate (`scryer-prolog`) - can be embedded directly
- **CLP(Z) built-in**: Constraint Logic Programming over integers is a first-class library
- **Battle-tested paradigm**: Prolog has been solving constraint problems for decades
- **Reusable knowledge**: CSP skills transfer to other domains beyond Factorio
- **Declarative constraints**: Express *what* must be true, not *how* to find it

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Rust (FacRepl)                          │
├─────────────────────────────────────────────────────────────────┤
│  ProductionGoal        │  Parse solution  │  DSL Commands       │
│  (item, throughput)    │  from Prolog     │  (entity placements)│
│         │              │       ▲          │         │           │
│         ▼              │       │          │         ▼           │
│  ┌─────────────────────┴───────┴──────────┴─────────────────┐   │
│  │              Problem Generator / Solution Parser          │   │
│  │  - Converts goals to Prolog facts                        │   │
│  │  - Parses Prolog terms back to Rust structs              │   │
│  └─────────────────────┬───────┬──────────┬─────────────────┘   │
│                        │       │          │                     │
└────────────────────────┼───────┼──────────┼─────────────────────┘
                         │       │          │
                         ▼       │          ▲
┌────────────────────────────────┴──────────────────────────────┐
│                     Scryer Prolog (embedded)                   │
├────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────────────┐ │
│  │   clpz       │  │   Facts DB   │  │   Constraint Rules    │ │
│  │  (integers)  │  │  - entities  │  │  - spatial placement  │ │
│  │              │  │  - recipes   │  │  - connectivity       │ │
│  │  X in 0..100 │  │  - sizes     │  │  - throughput         │ │
│  │  X #< Y      │  │  - speeds    │  │  - power coverage     │ │
│  └──────────────┘  └──────────────┘  └───────────────────────┘ │
│                              │                                  │
│                              ▼                                  │
│                    ┌─────────────────┐                         │
│                    │  labeling/1     │                         │
│                    │  (search for    │                         │
│                    │   solutions)    │                         │
│                    └─────────────────┘                         │
└────────────────────────────────────────────────────────────────┘
```

### Rust ↔ Scryer Prolog Integration

**Adding the dependency:**
```toml
[dependencies]
scryer-prolog = "0.10"
```

**Basic usage pattern:**
```rust
use scryer_prolog::Machine;

// Create a Prolog machine
let mut machine = Machine::new();

// Load constraint definitions
machine.consult_module_string("user", FACTORIO_CONSTRAINTS)?;

// Load problem-specific facts
let facts = generate_problem_facts(&production_goal);
machine.consult_module_string("user", &facts)?;

// Run the solver query
let solutions = machine.run_query("solve(Layout).")?;

// Parse the first solution
for answer in solutions {
    let layout = parse_layout_term(&answer)?;
    return Ok(layout);
}
```

**Key Scryer Prolog types:**
- `Machine` - An instance of Scryer Prolog
- `MachineBuilder` - Configures machine creation
- `QueryResolution` - Iterator over query solutions
- `LeafAnswer` - A single solution binding
- `Term` - Represents a Prolog term (for parsing results)

### Constraint Model Design

The Prolog constraint model is organized into layers:

#### 1. Entity Database (Facts)

```prolog
% Entity definitions
% entity(Name, Category, Width, Height, CraftingSpeed, ModuleSlots)
entity(assembler_1, assembler, 3, 3, 0.5, 0).
entity(assembler_2, assembler, 3, 3, 0.75, 2).
entity(assembler_3, assembler, 3, 3, 1.25, 4).
entity(belt_yellow, belt, 1, 1, _, _).
entity(belt_red, belt, 1, 1, _, _).
entity(belt_blue, belt, 1, 1, _, _).
entity(inserter_fast, inserter, 1, 1, _, _).

% Recipe definitions
% recipe(Name, Inputs, Outputs, CraftingTime, Category)
recipe(green_circuit,
       [(iron_plate, 1), (copper_cable, 3)],
       [(green_circuit, 1)],
       0.5, assembling).

recipe(copper_cable,
       [(copper_plate, 1)],
       [(copper_cable, 2)],
       0.5, assembling).
```

#### 2. Spatial Constraints (CLP(Z))

```prolog
:- use_module(library(clpz)).

% Position variables have bounded domains
position_in_bounds(X, Y, MaxX, MaxY) :-
    X in 0..MaxX,
    Y in 0..MaxY.

% No two entities overlap
no_overlap(X1, Y1, W1, H1, X2, Y2, W2, H2) :-
    X1 + W1 #=< X2 #\/ X2 + W2 #=< X1 #\/
    Y1 + H1 #=< Y2 #\/ Y2 + H2 #=< Y1.

% All entities pairwise non-overlapping
all_no_overlap([]).
all_no_overlap([_]).
all_no_overlap([E1|Rest]) :-
    maplist(no_overlap_with(E1), Rest),
    all_no_overlap(Rest).
```

#### 3. Connectivity Constraints

```prolog
% Inserter can reach between two positions
inserter_reaches(InsX, InsY, InsDir, FromX, FromY, ToX, ToY) :-
    % Inserter picks from behind, places in front
    (InsDir = north ->
        FromX #= InsX, FromY #= InsY + 1,
        ToX #= InsX, ToY #= InsY - 1
    ; InsDir = south ->
        FromX #= InsX, FromY #= InsY - 1,
        ToX #= InsX, ToY #= InsY + 1
    ; InsDir = east ->
        FromX #= InsX - 1, FromY #= InsY,
        ToX #= InsX + 1, ToY #= InsY
    ; InsDir = west ->
        FromX #= InsX + 1, FromY #= InsY,
        ToX #= InsX - 1, ToY #= InsY
    ).

% Belt connects to adjacent belt
belt_connects(X1, Y1, Dir1, X2, Y2) :-
    (Dir1 = north -> X2 #= X1, Y2 #= Y1 - 1
    ; Dir1 = south -> X2 #= X1, Y2 #= Y1 + 1
    ; Dir1 = east -> X2 #= X1 + 1, Y2 #= Y1
    ; Dir1 = west -> X2 #= X1 - 1, Y2 #= Y1
    ).
```

#### 4. Production Constraints

```prolog
% Calculate required machines for throughput
machines_needed(Recipe, TargetPerSec, MachineType, Count) :-
    recipe(Recipe, _, _, CraftTime, _),
    entity(MachineType, assembler, _, _, Speed, _),
    ItemsPerMachine is Speed / CraftTime,
    Count is ceiling(TargetPerSec / ItemsPerMachine).

% Production chain satisfies throughput
production_satisfies(Machines, Recipe, TargetPerSec) :-
    findall(Speed, (
        member(machine(Type, _, _, Recipe), Machines),
        entity(Type, assembler, _, _, Speed, _),
        recipe(Recipe, _, _, CraftTime, _)
    ), Speeds),
    sum_list(Speeds, TotalSpeed),
    recipe(Recipe, _, _, CraftTime, _),
    TotalOutput is TotalSpeed / CraftTime,
    TotalOutput >= TargetPerSec.
```

#### 5. Main Solver

```prolog
% Main entry point
solve(Goal, Layout) :-
    Goal = goal(Item, Throughput),

    % Calculate what we need
    production_chain(Item, Chain),
    required_machines(Chain, Throughput, MachineList),

    % Create position variables
    create_placements(MachineList, Placements),

    % Apply constraints
    all_positions_bounded(Placements, 100, 100),
    all_no_overlap(Placements),
    all_connected(Placements),
    production_satisfies(Placements, Item, Throughput),

    % Search for solution
    extract_variables(Placements, Vars),
    labeling([ff], Vars),  % first-fail heuristic

    Layout = Placements.
```

### File Organization

```
rust-tools/src/
├── solver/
│   ├── mod.rs              # Solver interface
│   ├── prolog/
│   │   ├── mod.rs          # Prolog integration
│   │   ├── bridge.rs       # Rust ↔ Prolog communication
│   │   ├── facts.rs        # Generate Prolog facts from Rust
│   │   └── parser.rs       # Parse Prolog terms to Rust
│   └── constraints/        # Prolog source files
│       ├── entities.pl     # Entity database
│       ├── recipes.pl      # Recipe database
│       ├── spatial.pl      # Spatial constraints
│       ├── connectivity.pl # Connection constraints
│       ├── production.pl   # Throughput constraints
│       └── solver.pl       # Main solver logic

prolog/                     # Standalone Prolog files for testing
├── factorio.pl             # Complete constraint model
├── test_green_circuit.pl   # Test case: green circuits
└── test_simple.pl          # Minimal test case
```

### Development Workflow

1. **Develop constraints in Prolog first**
   - Use Scryer Prolog CLI to test interactively
   - Iterate on constraint model without recompiling Rust

2. **Integrate with Rust**
   - Embed Prolog source files using `include_str!`
   - Generate problem-specific facts from Rust structs
   - Parse solutions back to Rust types

3. **Test end-to-end**
   - Run solver from Rust
   - Generate DSL commands
   - Execute in Factorio

### Example: Solving Green Circuits

**Input (Rust):**
```rust
let goal = ProductionGoal {
    item: "green_circuit".to_string(),
    throughput: Throughput::ItemsPerSecond(45.0), // 1 blue belt
};
```

**Generated Prolog query:**
```prolog
?- solve(goal(green_circuit, 45.0), Layout).
```

**Prolog solution:**
```prolog
Layout = [
    machine(assembler_2, 5, 5, green_circuit),
    machine(assembler_2, 5, 10, green_circuit),
    machine(assembler_2, 5, 15, copper_cable),
    inserter(inserter_fast, 4, 5, east),
    inserter(inserter_fast, 6, 5, east),
    belt(belt_blue, 3, 5, east),
    belt(belt_blue, 7, 5, east),
    ...
].
```

**Generated DSL:**
```
assembler-2 5 5 recipe:green-circuit
assembler-2 5 10 recipe:green-circuit
assembler-2 5 15 recipe:copper-cable
inserter-fast 4 5 :e
inserter-fast 6 5 :e
belt-blue 3 5 :e
belt-blue 7 5 :e
```

### Benefits of This Approach

1. **Declarative**: Express constraints naturally - "inserters must reach", "no overlap"
2. **Complete search**: Prolog explores all possibilities, guaranteed to find solution if one exists
3. **Backtracking built-in**: No manual search tree management
4. **Reusable**: Same approach works for other CSP domains
5. **Debuggable**: Can test Prolog constraints independently
6. **Extensible**: Add new constraints without rewriting solver
