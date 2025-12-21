# Factorio Constraint Builder - Contents

This archive contains the complete planning and scaffolding for the Factorio Constraint Builder project.

## Where to Start

1. **Read `README.md`** - Project overview and quick start guide
2. **Read `PLANNING.md`** - Comprehensive planning document with architecture, design decisions, and development phases
3. **Review `DSL-REFERENCE.md`** - Complete DSL syntax reference
4. **Explore the code structure** below

## Directory Structure

```
factorio-constraint-builder/
│
├── README.md                           # Project overview and quick start
├── PLANNING.md                         # Comprehensive planning document
├── DSL-REFERENCE.md                    # DSL syntax reference
├── CONTENTS.md                         # This file
│
├── factorio-mod/                       # Factorio mod (Lua)
│   ├── info.json                       # Mod metadata
│   └── control.lua                     # Main mod logic with RCON handlers
│
├── rust-tools/                         # Rust implementation
│   ├── Cargo.toml                      # Rust project configuration
│   │
│   ├── src/
│   │   ├── lib.rs                      # Library exports
│   │   │
│   │   ├── rcon/                       # RCON communication module
│   │   │   └── mod.rs                  # RCON bridge implementation
│   │   │
│   │   ├── dsl/                        # DSL parsing and execution
│   │   │   ├── mod.rs                  # Module exports
│   │   │   ├── ast.rs                  # Abstract syntax tree definitions
│   │   │   ├── parser.rs               # DSL parser
│   │   │   └── executor.rs             # DSL executor
│   │   │
│   │   ├── entities/                   # Entity and recipe data
│   │   │   ├── mod.rs                  # Module exports
│   │   │   ├── types.rs                # Entity type definitions
│   │   │   └── recipes.rs              # Recipe database
│   │   │
│   │   ├── solver/                     # Constraint solver
│   │   │   ├── mod.rs                  # Solver interface
│   │   │   ├── constraints.rs          # Constraint definitions (stub)
│   │   │   ├── ratios.rs               # Ratio calculations (stub)
│   │   │   └── placement.rs            # Placement algorithm (stub)
│   │   │
│   │   └── bin/                        # Binary executables
│   │       ├── repl.rs                 # Interactive REPL
│   │       └── solver.rs               # Solver CLI tool
│   │
│   └── tests/                          # Tests (to be added)
│
└── examples/                           # Example DSL scripts
    └── green-circuits.fcb              # Basic green circuit production example
```

## Key Files Explained

### Documentation

- **PLANNING.md**: The master planning document. Contains:
  - Project overview and goals
  - Complete architecture design
  - DSL specification
  - Factorio mod design
  - Rust component details
  - Communication protocol
  - Development phases
  - Testing strategy
  - Future extensions

- **DSL-REFERENCE.md**: Quick reference for DSL syntax
  - All commands with examples
  - Entity types and naming
  - Direction keywords
  - Recipe and module syntax
  - Common patterns

- **README.md**: Getting started guide
  - Installation instructions
  - Basic usage examples
  - Development status

### Factorio Mod

- **info.json**: Mod metadata (name, version, dependencies)

- **control.lua**: Complete mod implementation
  - Remote interface registration
  - Entity placement with validation
  - Query commands
  - Area operations
  - JSON response formatting
  - Error handling

### Rust Tools

#### Core Library (`src/lib.rs`)

Exports all major modules and types.

#### RCON Module (`src/rcon/`)

- **mod.rs**: RCON bridge for communicating with Factorio
  - Connection management
  - Command execution
  - Response parsing
  - High-level helper methods

#### DSL Module (`src/dsl/`)

- **ast.rs**: Abstract syntax tree types
  - Command enums
  - Position, Direction types
  - PlaceCommand, QueryCommand, etc.

- **parser.rs**: DSL parser
  - Line-by-line parsing
  - Script file parsing
  - Comprehensive test coverage
  - Error reporting

- **executor.rs**: DSL executor
  - Command execution via RCON
  - State management (history, saved states)
  - Error handling

#### Entities Module (`src/entities/`)

- **types.rs**: Entity type database
  - All Factorio entities
  - Size, speed, module slots
  - DSL name to Factorio name mapping

- **recipes.rs**: Recipe database
  - Recipe definitions
  - Ingredient/product tracking
  - Items-per-second calculations

#### Solver Module (`src/solver/`)

- **mod.rs**: Solver interface and basic types
  - ProductionGoal, Throughput
  - Solution structure
  - Solver trait

- **constraints.rs**: (stub) Constraint definitions
- **ratios.rs**: (stub) Ratio calculations
- **placement.rs**: (stub) Placement algorithms

#### Binaries (`src/bin/`)

- **repl.rs**: Interactive REPL
  - Command-line interface
  - History management
  - Script loading
  - Help system

- **solver.rs**: Solver CLI
  - Parse production goals
  - Run constraint solver
  - Output DSL scripts

### Examples

- **green-circuits.fcb**: Example DSL script showing:
  - Comments
  - Entity placement
  - Power infrastructure
  - Production chain setup

## Development Phases (from PLANNING.md)

1. **Phase 1**: Foundation (RCON + Basic Mod)
2. **Phase 2**: DSL Parser + Executor
3. **Phase 3**: REPL Interface
4. **Phase 4**: Enhanced Mod Capabilities
5. **Phase 5**: Basic Constraint Solver
6. **Phase 6**: Advanced Solver Features
7. **Phase 7**: Polish and Extensions

## Next Steps

### To start developing:

1. **Set up Factorio with RCON**
   ```bash
   factorio --rcon-port 27015 --rcon-password test123
   ```

2. **Install the mod**
   - Copy `factorio-mod/` to Factorio mods directory
   - Rename to `factorio-constraint-builder_0.1.0`
   - Enable in-game

3. **Build the Rust tools**
   ```bash
   cd rust-tools
   cargo build
   ```

4. **Test the connection**
   ```bash
   cargo run --bin fcb-repl -- --password test123
   ```

5. **Start implementing**
   - Begin with Phase 1: complete RCON implementation
   - Use the actual `rcon` crate to replace stubs
   - Test basic placement commands
   - Iterate from there

### To extend the project:

- Add more entity types in `entities/types.rs`
- Add more recipes in `entities/recipes.rs`
- Implement solver algorithms in `solver/`
- Add more example scripts in `examples/`
- Enhance the REPL with more commands
- Add tests in `tests/`

## Notes

- This is the **planning and scaffolding** phase
- Code contains stubs and TODOs marked for implementation
- Architecture is designed to be modular and extensible
- DSL syntax can be refined based on actual usage
- All files are ready to be transferred to Claude Code for implementation

## Questions or Issues?

Refer to:
- Architecture details → `PLANNING.md`
- Syntax questions → `DSL-REFERENCE.md`
- Getting started → `README.md`
- Code structure → This file

Happy building! 🏗️
