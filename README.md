# FacRepl - Factorio Constraint Builder

A system for generating and placing Factorio factory layouts using constraint satisfaction and a custom DSL. Control Factorio in real-time via RCON, write placement scripts, or let the Prolog-based constraint solver generate layouts automatically from production goals.

## Architecture

```
Constraint Solver (Scryer Prolog / CLP(Z))
        │ generates
        v
   DSL Scripts (.dsl/.fcb)
        │ loaded by
        v
  REPL (Rust, interactive) ──> DSL Executor ──> RCON Bridge ──> Factorio Mod (Lua)
```

**Components:**
- **Factorio Mod** (Lua) - Receives commands via RCON, places/queries/removes entities in-game
- **RCON Bridge** (Rust) - TCP connection to Factorio's RCON server
- **DSL Parser & Executor** (Rust) - Parses DSL syntax, executes commands, manages undo/state
- **REPL** (Rust) - Interactive command-line interface with history
- **Constraint Solver** (Rust + Prolog) - Generates layouts from high-level production goals using CLP(Z)

## Quick Start

### Prerequisites

- Factorio 2.0+
- Rust toolchain (1.70+)
- Scryer Prolog (for constraint solver, optional)

### 1. Install the Mod

Copy `factorio-mod/` to your Factorio mods folder and rename to `factorio-constraint-builder_0.1.0`:

- Linux: `~/.factorio/mods/`
- Windows: `%appdata%\Factorio\mods\`
- Mac: `~/Library/Application Support/factorio/mods/`

Enable the mod in Factorio.

### 2. Start Factorio with RCON

```bash
factorio --rcon-port 27015 --rcon-password yourpassword
```

### 3. Build and Run

```bash
cd rust-tools
cargo build --release
```

```bash
export FACTORIO_RCON_PASSWORD=yourpassword
./target/release/fcb-repl
```

Or pass connection details explicitly:
```bash
./target/release/fcb-repl --host localhost --port 27015 --password yourpassword
```

## DSL Overview

See `DSL-REFERENCE.md` for the complete syntax reference.

### Entity Placement

```
belt-blue 10 20 :north
inserter-fast 15 20 :e
assembler-3 20 20 recipe:green-circuit module:speed-3 module:speed-3
splitter-blue 25 10 :n out-priority:right filter:iron-plate
power-pole-medium 0 0
pipe 10 10
```

Supported entities: belts (yellow/red/blue), underground belts, splitters (with priority and filter options), inserters (burner/basic/fast/long/filter/stack/stack-filter), assemblers (1/2/3), power poles (small/medium/big), pipes, pumps, storage tanks.

Directions: `:north`/`:n`, `:south`/`:s`, `:east`/`:e`, `:west`/`:w` (and diagonals).

### Queries and Area Operations

```
what-at 10 20           ; inspect entity at position
can-place belt-blue 15 25  ; check if placement is valid
area 0 0 50 50          ; list entities in region
clear 0 0 20 20         ; remove all entities in area
```

### State Management

```
save-state :before-belts
; ... make changes ...
load-state :before-belts   ; revert
undo                       ; undo last command
undo 5                     ; undo last 5
```

### REPL Commands

```
:help              show help
:load file.dsl     load and execute a script
:entities          list available entity types
:player / :pos     get player position
:history           show command history
:quit              exit
```

### Scripts

```bash
# Execute a DSL script file
./target/release/fcb-repl --script examples/green-circuits.fcb

# Or load from within the REPL
fcb> :load examples/green-circuits.fcb
```

## Constraint Solver

**Status: Experimental / Work in Progress** - The Rust-to-Prolog integration and the constraint model itself are still under active development. The solver can be invoked but does not yet reliably produce valid, placeable layouts.

The goal is to use Scryer Prolog with CLP(Z) (Constraint Logic Programming over integers) to generate factory layouts from production goals.

```bash
./target/release/fcb-solver --item green-circuit --throughput 45 --prolog-path /path/to/scryer-prolog
```

The Prolog model (`prolog/factorio.pl`) encodes:
- Entity footprints and spatial constraints (no overlap)
- Recipe databases with crafting times and ratios
- Throughput calculations and machine count requirements
- Belt connectivity and inserter reach constraints

Solutions are intended to be output as DSL scripts that can be loaded into the REPL.

## Project Structure

```
├── factorio-mod/           Factorio Lua mod (RCON command handlers)
├── rust-tools/             Rust implementation
│   └── src/
│       ├── bin/repl.rs     Interactive REPL
│       ├── bin/solver.rs   Solver CLI
│       ├── dsl/            Parser, AST, executor
│       ├── rcon/           RCON bridge
│       ├── solver/         Prolog solver integration
│       └── entities/       Entity and recipe databases
├── prolog/                 Constraint model (CLP(Z))
├── examples/               Example DSL scripts
├── DSL-REFERENCE.md        Complete DSL syntax reference
└── PLANNING.md             Architecture and design document
```

## Development Status

**Working:**
- Full RCON communication with Factorio 2.0
- DSL parser and executor with undo/save-load state
- Interactive REPL with history and script loading
- Entity placement, queries, and area operations
- Splitter priority settings and item filters
- Floating-point coordinates for sub-tile precision

**In Progress:**
- Prolog-based constraint solver (Rust/Prolog bridge exists but solver doesn't yet produce reliable layouts)
- Advanced solver strategies and layout optimization
- Blueprint generation support

## License

MIT

## Resources

- `PLANNING.md` - Architecture and design decisions
- `DSL-REFERENCE.md` - Complete DSL syntax
- `CONTENTS.md` - Detailed file descriptions
- [Factorio Mod API](https://lua-api.factorio.com/)
- [Scryer Prolog](https://github.com/mthom/scryer-prolog)
