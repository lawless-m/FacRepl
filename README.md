# Factorio Constraint Builder

An experimental system for generating and placing Factorio factory layouts using constraint satisfaction and a custom DSL.

## Quick Start

### Prerequisites

- Factorio (1.1+)
- Rust toolchain (1.70+)
- Factorio running with RCON enabled

### Enable RCON in Factorio

Start Factorio with RCON enabled:
```bash
factorio --rcon-port 27015 --rcon-password yourpassword
```

Or add to your server settings.

### Install the Mod

1. Copy the `factorio-mod` directory to your Factorio mods folder:
   - Windows: `%appdata%\Factorio\mods\`
   - Linux: `~/.factorio/mods/`
   - Mac: `~/Library/Application Support/factorio/mods/`

2. Rename the directory to `factorio-constraint-builder_0.1.0`

3. Enable the mod in Factorio

### Build the Rust Tools

```bash
cd rust-tools
cargo build --release
```

### Run the REPL

```bash
./target/release/fcb-repl --host localhost --port 27015 --password yourpassword
```

Or set the password via environment variable:
```bash
export FACTORIO_RCON_PASSWORD=yourpassword
./target/release/fcb-repl
```

## Usage

### Interactive REPL

```
fcb> belt-blue 10 20 :n
Placed transport-belt at (10, 20)

fcb> assembler-2 15 15 recipe:green-circuit
Placed assembling-machine-2 at (15, 15)

fcb> what-at 10 20
{
  "name": "transport-belt",
  "position": {"x": 10, "y": 20},
  "direction": 0
}

fcb> :help
[shows help]

fcb> :quit
```

### Load and Execute Scripts

```bash
./target/release/fcb-repl --script examples/green-circuits.fcb
```

Or from within the REPL:
```
fcb> :load examples/green-circuits.fcb
```

### Constraint Solver (Coming Soon)

```bash
./target/release/fcb-solver --item green-circuit --throughput 1-blue-belt --output solution.fcb
```

## DSL Syntax

See `DSL-REFERENCE.md` for complete syntax documentation.

### Quick Examples

```
; Place entities
belt-blue 10 20 :n
inserter-fast 15 20 :e
assembler-3 20 20 recipe:green-circuit module:speed-3

; Query the game
what-at 10 20
can-place belt-blue 15 25
area 0 0 50 50

; Area operations
clear 0 0 20 20
```

## Architecture

- **Factorio Mod (Lua)**: Receives commands via RCON and places entities
- **RCON Bridge (Rust)**: Communicates with Factorio
- **DSL Parser (Rust)**: Parses DSL syntax into commands
- **DSL Executor (Rust)**: Executes commands via RCON
- **REPL (Rust)**: Interactive interface
- **Constraint Solver (Rust)**: Generates layouts from goals

See `PLANNING.md` for detailed architecture documentation.

## Development Status

This is an experimental project in early development.

**Currently Working:**
- Basic DSL syntax
- RCON communication (stub)
- Factorio mod command handlers
- REPL interface structure

**In Progress:**
- Full RCON implementation
- Complete entity database
- Constraint solver

**Planned:**
- Multiple solver strategies
- Advanced layout features
- Blueprint integration

## Contributing

This is a personal experimental project, but ideas and suggestions are welcome!

## License

MIT

## Resources

- Planning Document: `PLANNING.md`
- DSL Reference: `DSL-REFERENCE.md`
- Example Scripts: `examples/`
- Factorio Mod API: https://lua-api.factorio.com/
