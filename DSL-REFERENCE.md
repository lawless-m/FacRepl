# DSL Syntax Reference

Quick reference for the Factorio Constraint Builder DSL.

## Basic Syntax

```
<command> <arguments>
```

Comments start with `;`:
```
; This is a comment
```

## Entity Placement

### Format
```
<entity-type> <x> <y> <:direction> [options]
```

### Directions
```
Full:           Short:
:north          :n
:south          :s
:east           :e
:west           :w
:northeast      :ne
:northwest      :nw
:southeast      :se
:southwest      :sw
```

### Transport Belts
```
belt-yellow <x> <y> <:dir>
belt-red <x> <y> <:dir>
belt-blue <x> <y> <:dir>

Examples:
belt-yellow 10 20 :n
belt-red 15 20 :e
belt-blue 20 20 :s
```

### Underground Belts
```
underground-belt-yellow <x> <y> <:dir>
underground-belt-red <x> <y> <:dir>
underground-belt-blue <x> <y> <:dir>

Examples:
underground-belt-blue 10 10 :n
underground-belt-blue 15 10 :n  ; pairs with above
```

### Splitters
```
splitter-yellow <x> <y> <:dir>
splitter-red <x> <y> <:dir>
splitter-blue <x> <y> <:dir>

Examples:
splitter-blue 10 10 :n
```

### Inserters
```
inserter-burner <x> <y> <:dir>
inserter-basic <x> <y> <:dir>
inserter-fast <x> <y> <:dir>
inserter-long <x> <y> <:dir>
inserter-stack <x> <y> <:dir>

Examples:
inserter-fast 10 10 :e
inserter-stack 15 15 :w
```

### Assemblers
```
assembler-1 <x> <y> [recipe:<name>] [module:<name>]...
assembler-2 <x> <y> [recipe:<name>] [module:<name>]...
assembler-3 <x> <y> [recipe:<name>] [module:<name>]...

Examples:
assembler-2 10 10 recipe:iron-gear
assembler-3 20 20 recipe:green-circuit module:speed-3 module:speed-3
```

### Power Poles
```
power-pole-small <x> <y>
power-pole-medium <x> <y>
power-pole-big <x> <y>

Examples:
power-pole-medium 0 0
power-pole-big 50 50
```

### Pipes and Fluids
```
pipe <x> <y>
pump <x> <y> <:dir>
storage-tank <x> <y>

Examples:
pipe 10 10
pump 15 15 :n
storage-tank 20 20
```

## Query Commands

### What's at a position?
```
what-at <x> <y>

Example:
what-at 10 20
```

### Can we place here?
```
can-place <entity-type> <x> <y>

Example:
can-place belt-blue 15 25
```

### Query area
```
area <x1> <y1> <x2> <y2>

Example:
area 0 0 50 50
```

## Area Operations

### Clear area
```
clear <x1> <y1> <x2> <y2>

Example:
clear 0 0 20 20
```

### Clear specific entity type
```
clear-entity <entity-type> <x1> <y1> <x2> <y2>

Example:
clear-entity belt-yellow 10 10 30 30
```

## State Management

### Undo
```
undo
undo <count>

Examples:
undo        ; undo last command
undo 5      ; undo last 5 commands
```

### Save/Load State
```
save-state :<name>
load-state :<name>

Examples:
save-state :checkpoint1
load-state :checkpoint1
```

## REPL-Only Commands

These commands only work in the REPL, not in scripts:

```
:help                 ; Show help
:help <command>       ; Help for specific command
:load <file>         ; Load and execute script
:save <file>         ; Save history to file
:quit or :q          ; Exit REPL
:clear               ; Clear screen
:history             ; Show command history
:entities            ; List available entities
```

## Common Recipes

### Production
```
recipe:iron-gear
recipe:copper-cable
recipe:green-circuit
recipe:red-circuit
recipe:blue-circuit
recipe:steel-plate
recipe:plastic-bar
recipe:sulfur
recipe:battery
```

### Science
```
recipe:automation-science-pack
recipe:logistic-science-pack
recipe:military-science-pack
recipe:chemical-science-pack
recipe:production-science-pack
recipe:utility-science-pack
```

## Common Modules

### Speed Modules
```
module:speed-1
module:speed-2
module:speed-3
```

### Productivity Modules
```
module:productivity-1
module:productivity-2
module:productivity-3
```

### Efficiency Modules
```
module:efficiency-1
module:efficiency-2
module:efficiency-3
```

## Example Scripts

### Simple Belt Line
```
; Horizontal belt line
belt-blue 0 0 :e
belt-blue 1 0 :e
belt-blue 2 0 :e
belt-blue 3 0 :e
belt-blue 4 0 :e
```

### Basic Assembler Setup
```
; Power
power-pole-medium 0 0

; Input belt
belt-blue 5 10 :e

; Pick up from belt
inserter-fast 6 10 :n

; Assembler
assembler-2 6 11 recipe:iron-gear

; Output
inserter-fast 6 12 :s
belt-blue 5 12 :e
```

### Green Circuit Production
```
; Power infrastructure
power-pole-medium 0 0
power-pole-medium 10 0

; Copper cable assemblers
assembler-2 5 5 recipe:copper-cable
inserter-fast 4 5 :w
inserter-long 6 5 :e

; Green circuit assemblers
assembler-2 10 5 recipe:green-circuit
inserter-fast 9 5 :w
inserter-fast 11 5 :e

; Belt network
belt-blue 0 5 :e
belt-blue 1 5 :e
belt-blue 2 5 :e
belt-blue 3 5 :e
belt-blue 15 5 :e
belt-blue 16 5 :e
```

### Organized by Steps
```
; Step 1: Clear the area
clear 0 0 30 30

; Step 2: Power grid
power-pole-medium 5 5
power-pole-medium 15 5
power-pole-medium 25 5

; Step 3: Main bus
belt-blue 0 10 :e
belt-blue 1 10 :e
; ... continue belt line

; Step 4: Production
assembler-2 10 15 recipe:iron-gear
inserter-fast 10 14 :n
inserter-fast 10 16 :s
```

## Tips

1. **Comments are your friend**: Use `;` to document your scripts
2. **Test incrementally**: Build small sections and verify
3. **Use save-state**: Checkpoint before major changes
4. **Query before placing**: Use `can-place` to verify
5. **Clear carefully**: Double-check coordinates before clearing
6. **Power first**: Place power poles before machines
7. **Watch directions**: Belt and inserter directions matter!
8. **Module slots**: Check entity module slots before adding modules
