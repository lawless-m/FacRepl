# Scryer Prolog Integration in Rust

This guide covers incorporating Scryer Prolog into Rust code, dynamically inserting Horn clauses at runtime, and exploring use cases beyond simple factorial examples.

## Table of Contents

1. [Adding Scryer Prolog Dependency](#adding-scryer-prolog-dependency)
2. [Creating a Prolog Machine](#creating-a-prolog-machine)
3. [Loading Prolog Files](#loading-prolog-files)
4. [Inserting Horn Clauses at Runtime](#inserting-horn-clauses-at-runtime)
5. [Running Queries](#running-queries)
6. [Parsing Results into Rust Types](#parsing-results-into-rust-types)
7. [Alternative Use Cases](#alternative-use-cases)

---

## Adding Scryer Prolog Dependency

Add `scryer-prolog` to your `Cargo.toml`:

```toml
[dependencies]
scryer-prolog = "0.10"
```

**Note**: Scryer Prolog is a native Rust implementation of ISO Prolog, making it ideal for embedding in Rust applications without FFI overhead.

---

## Creating a Prolog Machine

The `Machine` is the core execution engine for Prolog programs:

```rust
use scryer_prolog::Machine;
use scryer_prolog::MachineBuilder;

// Create a default machine
let machine = Machine::new();

// Or customize with MachineBuilder
let machine = MachineBuilder::new()
    .build();
```

The Machine instance maintains the Prolog knowledge base and handles all queries.

---

## Loading Prolog Files

### Loading from a File Path

```rust
use scryer_prolog::Machine;

let mut machine = Machine::new();

// Load a Prolog file
machine.consult_module_file("prolog/factorio.pl")?;
```

### Loading from a String

For embedded Prolog code or dynamically generated programs:

```rust
let prolog_code = r#"
    :- use_module(library(clpz)).

    entity(assembler_2, 3, 3).
    entity(belt_blue, 1, 1).

    no_overlap(X1, Y1, W1, H1, X2, Y2, W2, H2) :-
        X1 + W1 #=< X2 #\/ X2 + W2 #=< X1 #\/
        Y1 + H1 #=< Y2 #\/ Y2 + H2 #=< Y1.
"#;

machine.consult_module_string("user", prolog_code)?;
```

### Loading Multiple Modules

Prolog programs can be organized into modules:

```rust
// Load core definitions
machine.consult_module_file("prolog/entities.pl")?;

// Load constraint rules
machine.consult_module_file("prolog/constraints.pl")?;

// Load solver logic
machine.consult_module_file("prolog/solver.pl")?;
```

---

## Inserting Horn Clauses at Runtime

Horn clauses (facts and rules) can be dynamically added to the knowledge base. This is powerful for:

- Adding problem-specific facts before solving
- Updating the knowledge base based on user input
- Building incremental reasoning systems

### Method 1: Using `consult_module_string`

Generate Prolog code as a string and load it:

```rust
fn add_production_facts(machine: &mut Machine, items: &[(&str, u32)]) -> Result<()> {
    let mut facts = String::new();

    for (item, quantity) in items {
        // Each line is a Horn clause (a fact)
        facts.push_str(&format!("production_request({}, {}).\n", item, quantity));
    }

    machine.consult_module_string("user", &facts)?;
    Ok(())
}

// Usage
add_production_facts(&mut machine, &[
    ("green_circuit", 4500),
    ("iron_gear", 1000),
])?;
```

### Method 2: Using `assertz` via Query

For adding individual clauses during execution:

```rust
fn assert_fact(machine: &mut Machine, fact: &str) -> Result<()> {
    let query = format!("assertz({}).", fact);
    let _ = machine.run_query(&query)?;
    Ok(())
}

// Add a fact
assert_fact(&mut machine, "entity(custom_machine, 2, 2)")?;

// Add a rule
assert_fact(&mut machine, "(custom_rule(X) :- entity(X, _, _), X \\= assembler_2)")?;
```

### Method 3: Bulk Dynamic Fact Generation

For large datasets, generate a complete Prolog program:

```rust
struct FactGenerator {
    facts: Vec<String>,
}

impl FactGenerator {
    fn new() -> Self {
        Self { facts: Vec::new() }
    }

    fn add_entity(&mut self, name: &str, width: u32, height: u32) {
        self.facts.push(format!("entity({}, {}, {}).", name, width, height));
    }

    fn add_recipe(&mut self, name: &str, inputs: &[(&str, u32)], output_count: u32) {
        let inputs_str: Vec<String> = inputs
            .iter()
            .map(|(item, count)| format!("({}, {})", item, count))
            .collect();
        self.facts.push(format!(
            "recipe({}, [{}], {}).",
            name,
            inputs_str.join(", "),
            output_count
        ));
    }

    fn to_prolog(&self) -> String {
        self.facts.join("\n")
    }
}

// Usage
let mut gen = FactGenerator::new();
gen.add_entity("custom_assembler", 4, 4);
gen.add_recipe("custom_item", &[("iron_plate", 5)], 1);

machine.consult_module_string("user", &gen.to_prolog())?;
```

### Method 4: Retracting and Replacing Clauses

To modify existing facts at runtime:

```rust
// Remove all matching clauses
fn retract_all(machine: &mut Machine, pattern: &str) -> Result<()> {
    let query = format!("retractall({}).", pattern);
    let _ = machine.run_query(&query)?;
    Ok(())
}

// Example: Update production request
retract_all(&mut machine, "production_request(green_circuit, _)")?;
assert_fact(&mut machine, "production_request(green_circuit, 9000)")?;
```

---

## Running Queries

### Simple Queries

```rust
// Run a query and iterate over solutions
let solutions = machine.run_query("entity(Name, Width, Height).")?;

for solution in solutions {
    match solution {
        Ok(leaf_answer) => {
            println!("Found: {:?}", leaf_answer);
        }
        Err(e) => {
            eprintln!("Query error: {:?}", e);
        }
    }
}
```

### Queries with Variables

```rust
// Find all 3x3 entities
let query = "entity(Name, 3, 3).";
let solutions = machine.run_query(query)?;
```

### Compound Queries

```rust
// Complex constraint query
let query = r#"
    entity(E1, W1, H1),
    entity(E2, W2, H2),
    E1 \= E2,
    W1 =:= W2,
    H1 =:= H2.
"#;
let solutions = machine.run_query(query)?;
```

---

## Parsing Results into Rust Types

### Extracting Variable Bindings

```rust
use scryer_prolog::LeafAnswer;

fn parse_placement(answer: &LeafAnswer) -> Option<Placement> {
    // Extract bindings from the answer
    let bindings = answer.bindings();

    let entity_type = bindings.get("Type")?.as_atom()?;
    let x = bindings.get("X")?.as_integer()?;
    let y = bindings.get("Y")?.as_integer()?;

    Some(Placement {
        entity_type: entity_type.to_string(),
        x: x as i32,
        y: y as i32,
    })
}
```

### Parsing List Results

When the Prolog solution returns a list (like a layout):

```rust
fn parse_layout(answer: &LeafAnswer) -> Result<Vec<Placement>> {
    let layout_term = answer.bindings().get("Layout")?;

    let mut placements = Vec::new();

    // Iterate through list structure
    for item in layout_term.as_list()? {
        if let Some(placement) = parse_placement_term(&item) {
            placements.push(placement);
        }
    }

    Ok(placements)
}

fn parse_placement_term(term: &Term) -> Option<Placement> {
    // Term structure: placement(Type, X, Y, Extra)
    let args = term.as_compound()?;

    Some(Placement {
        entity_type: args[0].as_atom()?.to_string(),
        x: args[1].as_integer()? as i32,
        y: args[2].as_integer()? as i32,
    })
}
```

---

## Working with Our Prolog Files

This project has Prolog files in the `prolog/` directory that can be loaded and manipulated:

### Loading the Factory Constraint Model

```rust
let mut machine = Machine::new();

// Load the main constraint model
machine.consult_module_file("prolog/factorio.pl")?;

// Add problem-specific facts
let goal_facts = format!(
    "solve_goal({}, {}).",
    "green_circuit",
    4500  // items per second * 100
);
machine.consult_module_string("user", &goal_facts)?;

// Run the solver
let solutions = machine.run_query("solve(goal(green_circuit, 4500), Layout).")?;
```

### Extending the Knowledge Base

```rust
// Add a new entity type at runtime
let new_entity = r#"
    entity(modded_assembler, assembler, 4, 4, 200, 6).
"#;
machine.consult_module_string("user", new_entity)?;

// Add a new recipe
let new_recipe = r#"
    recipe(advanced_circuit,
           [(green_circuit, 4), (copper_cable, 8)],
           [(advanced_circuit, 2)],
           100, assembling).
"#;
machine.consult_module_string("user", new_recipe)?;
```

---

## Alternative Use Cases

Beyond factorial calculations, Prolog excels at many problem domains. Here are practical applications with code examples:

### 1. Scheduling and Resource Allocation

```prolog
%% Job scheduling with CLP(Z)
:- use_module(library(clpz)).

% job(Name, Duration, RequiredMachine)
job(cutting, 3, cutter).
job(welding, 5, welder).
job(painting, 2, paint_booth).
job(assembly, 4, workbench).

% Schedule jobs on machines with no overlap
schedule(Jobs, Schedule) :-
    findall(job(N,D,M), job(N,D,M), Jobs),
    maplist(job_vars, Jobs, Schedule),
    no_machine_conflicts(Schedule),
    flatten_times(Schedule, AllTimes),
    labeling([min(Makespan)], AllTimes).

job_vars(job(Name, Duration, Machine), task(Name, Start, Duration, Machine)) :-
    Start in 0..100.

no_machine_conflicts([]).
no_machine_conflicts([Task|Rest]) :-
    maplist(no_conflict(Task), Rest),
    no_machine_conflicts(Rest).

no_conflict(task(_, S1, D1, M1), task(_, S2, D2, M2)) :-
    M1 \= M2, !.
no_conflict(task(_, S1, D1, M), task(_, S2, D2, M)) :-
    S1 + D1 #=< S2 #\/ S2 + D2 #=< S1.
```

**Rust Integration:**

```rust
fn schedule_jobs(machine: &mut Machine, jobs: &[Job]) -> Result<Schedule> {
    // Add jobs as facts
    for job in jobs {
        machine.consult_module_string("user",
            &format!("job({}, {}, {}).", job.name, job.duration, job.machine))?;
    }

    // Run scheduler
    let solutions = machine.run_query("schedule(Jobs, Schedule).")?;
    parse_schedule(solutions.next())
}
```

### 2. Configuration Validation

```prolog
%% Validate system configurations
:- use_module(library(lists)).

% Valid component combinations
compatible(cpu_intel, motherboard_asus).
compatible(cpu_intel, motherboard_gigabyte).
compatible(cpu_amd, motherboard_msi).
compatible(ram_ddr4, motherboard_asus).
compatible(ram_ddr4, motherboard_gigabyte).
compatible(ram_ddr4, motherboard_msi).
compatible(gpu_nvidia, psu_750w).
compatible(gpu_amd, psu_650w).

% Validate a full configuration
valid_config(Config) :-
    member(cpu(CPU), Config),
    member(motherboard(MB), Config),
    member(ram(RAM), Config),
    member(gpu(GPU), Config),
    member(psu(PSU), Config),
    compatible(CPU, MB),
    compatible(RAM, MB),
    compatible(GPU, PSU).

% Find all valid configs for given components
suggest_config(AvailableParts, ValidConfig) :-
    sublist_of(ValidConfig, AvailableParts),
    valid_config(ValidConfig).
```

**Rust Integration:**

```rust
fn validate_configuration(machine: &mut Machine, config: &Config) -> Result<bool> {
    let config_prolog = config.to_prolog_list();
    let query = format!("valid_config({}).", config_prolog);

    machine.run_query(&query)?
        .next()
        .is_some()
}
```

### 3. Dependency Resolution (Package Manager)

```prolog
%% Package dependency resolver
:- use_module(library(clpz)).

% package(Name, Version, Dependencies)
package(web_framework, 2, [(database, 1), (http_client, 1)]).
package(database, 1, [(connection_pool, 1)]).
package(database, 2, [(connection_pool, 2)]).
package(http_client, 1, []).
package(connection_pool, 1, []).
package(connection_pool, 2, []).

% Resolve dependencies
resolve(Requested, Resolved) :-
    resolve_all(Requested, [], Resolved).

resolve_all([], Acc, Acc).
resolve_all([Pkg|Rest], Acc, Resolved) :-
    \+ member(Pkg, Acc),
    package(Pkg, Version, Deps),
    dep_names(Deps, DepNames),
    append(DepNames, Rest, NewRest),
    resolve_all(NewRest, [(Pkg, Version)|Acc], Resolved).
resolve_all([Pkg|Rest], Acc, Resolved) :-
    member(Pkg, Acc),
    resolve_all(Rest, Acc, Resolved).

dep_names([], []).
dep_names([(Name, _)|Rest], [Name|Names]) :-
    dep_names(Rest, Names).
```

### 4. Graph Pathfinding

```prolog
%% Pathfinding with cycle detection
:- use_module(library(lists)).

% Graph edges
edge(a, b, 5).
edge(b, c, 3).
edge(c, d, 2).
edge(a, d, 10).
edge(b, d, 7).

% Find path with cost
path(Start, End, Path, Cost) :-
    path(Start, End, [Start], RevPath, 0, Cost),
    reverse(RevPath, Path).

path(End, End, Visited, Visited, Cost, Cost).
path(Current, End, Visited, Path, AccCost, TotalCost) :-
    edge(Current, Next, EdgeCost),
    \+ member(Next, Visited),
    NewCost is AccCost + EdgeCost,
    path(Next, End, [Next|Visited], Path, NewCost, TotalCost).

% Find shortest path
shortest_path(Start, End, Path, Cost) :-
    findall((P, C), path(Start, End, P, C), Paths),
    sort(2, @=<, Paths, [(Path, Cost)|_]).
```

### 5. Type Checking and Inference

```prolog
%% Simple type checker for expressions
:- use_module(library(lists)).

% Type rules
type(int(_), int).
type(bool(_), bool).
type(var(X), T) :- var_type(X, T).
type(add(E1, E2), int) :- type(E1, int), type(E2, int).
type(eq(E1, E2), bool) :- type(E1, T), type(E2, T).
type(if(Cond, Then, Else), T) :-
    type(Cond, bool),
    type(Then, T),
    type(Else, T).
type(lambda(X, Body), arrow(ArgT, RetT)) :-
    assert(var_type(X, ArgT)),
    type(Body, RetT),
    retract(var_type(X, ArgT)).
type(app(Func, Arg), RetT) :-
    type(Func, arrow(ArgT, RetT)),
    type(Arg, ArgT).

% Check if expression is well-typed
well_typed(Expr) :- type(Expr, _).
```

**Rust Integration:**

```rust
fn check_type(machine: &mut Machine, expr: &Expr) -> Result<Type> {
    let prolog_expr = expr.to_prolog();
    let query = format!("type({}, T).", prolog_expr);

    let solution = machine.run_query(&query)?.next()?;
    parse_type(&solution.bindings().get("T")?)
}
```

### 6. Rule-Based Access Control

```prolog
%% RBAC with Prolog
:- use_module(library(lists)).

% Role hierarchy
role_inherits(admin, editor).
role_inherits(editor, viewer).

% Permissions per role
permission(viewer, read).
permission(editor, write).
permission(admin, delete).

% User assignments
user_role(alice, admin).
user_role(bob, editor).
user_role(charlie, viewer).

% Check if role has permission (with inheritance)
role_has_permission(Role, Perm) :-
    permission(Role, Perm).
role_has_permission(Role, Perm) :-
    role_inherits(Role, ParentRole),
    role_has_permission(ParentRole, Perm).

% Check if user can perform action
can_access(User, Action) :-
    user_role(User, Role),
    role_has_permission(Role, Action).

% Dynamic role assignment at runtime
grant_role(User, Role) :-
    assertz(user_role(User, Role)).

revoke_role(User, Role) :-
    retract(user_role(User, Role)).
```

**Rust Integration:**

```rust
struct AccessControl {
    machine: Machine,
}

impl AccessControl {
    fn can_access(&mut self, user: &str, action: &str) -> Result<bool> {
        let query = format!("can_access({}, {}).", user, action);
        Ok(self.machine.run_query(&query)?.next().is_some())
    }

    fn grant_role(&mut self, user: &str, role: &str) -> Result<()> {
        let query = format!("grant_role({}, {}).", user, role);
        self.machine.run_query(&query)?;
        Ok(())
    }
}
```

### 7. Natural Language Parsing (DCG)

```prolog
%% Definite Clause Grammar for simple English
:- use_module(library(dcgs)).

sentence(s(NP, VP)) --> noun_phrase(NP), verb_phrase(VP).

noun_phrase(np(Det, N)) --> determiner(Det), noun(N).
noun_phrase(np(N)) --> noun(N).

verb_phrase(vp(V, NP)) --> verb(V), noun_phrase(NP).
verb_phrase(vp(V)) --> verb(V).

determiner(the) --> [the].
determiner(a) --> [a].

noun(cat) --> [cat].
noun(dog) --> [dog].
noun(mouse) --> [mouse].

verb(chases) --> [chases].
verb(eats) --> [eats].
verb(runs) --> [runs].

% Parse a sentence
parse(Words, Tree) :-
    phrase(sentence(Tree), Words).
```

**Rust Integration:**

```rust
fn parse_sentence(machine: &mut Machine, words: &[&str]) -> Result<ParseTree> {
    let words_prolog: Vec<String> = words.iter()
        .map(|w| w.to_string())
        .collect();
    let words_list = format!("[{}]", words_prolog.join(", "));

    let query = format!("parse({}, Tree).", words_list);
    let solution = machine.run_query(&query)?.next()?;

    parse_tree(&solution.bindings().get("Tree")?)
}
```

### 8. Constraint-Based Layout (This Project)

The factory layout solver in this project demonstrates constraint satisfaction:

```prolog
%% Spatial constraint solving
:- use_module(library(clpz)).

% No overlap constraint
no_overlap(X1, Y1, W1, H1, X2, Y2, W2, H2) :-
    X1 + W1 #=< X2 #\/ X2 + W2 #=< X1 #\/
    Y1 + H1 #=< Y2 #\/ Y2 + H2 #=< Y1.

% Solve for layout
solve(goal(Item, Throughput), Layout) :-
    machines_needed(Item, assembler_2, Throughput, Count),
    generate_placements(Count, Placements),
    all_no_overlap(Placements),
    label_positions(Placements),
    Layout = Placements.
```

---

## Best Practices

### 1. Separate Stable and Dynamic Knowledge

```rust
// Load stable rules once
machine.consult_module_file("prolog/core_rules.pl")?;

// Add dynamic facts per query
machine.consult_module_string("user", &dynamic_facts)?;

// Clean up after query
machine.run_query("retractall(dynamic_fact(_)).")?;
```

### 2. Use Modules for Organization

```prolog
%% In entities.pl
:- module(entities, [entity/6, entity_size/3]).

%% In constraints.pl
:- module(constraints, [no_overlap/8]).
:- use_module(entities).
```

### 3. Handle No Solutions Gracefully

```rust
fn solve_with_fallback(machine: &mut Machine, goal: &str) -> Result<Solution> {
    let solutions = machine.run_query(&format!("solve({}, Layout).", goal))?;

    match solutions.next() {
        Some(Ok(answer)) => parse_solution(&answer),
        Some(Err(e)) => Err(anyhow!("Query error: {:?}", e)),
        None => {
            // No solution found - try relaxed constraints
            let relaxed = machine.run_query(
                &format!("solve_relaxed({}, Layout).", goal)
            )?;
            relaxed.next()
                .ok_or(anyhow!("No solution found"))?
                .map_err(|e| anyhow!("Query error: {:?}", e))
                .and_then(|a| parse_solution(&a))
        }
    }
}
```

### 4. Validate Prolog Code at Compile Time

```rust
// Embed Prolog as a static string for compile-time inclusion
const CORE_PROLOG: &str = include_str!("../prolog/core.pl");

fn init_machine() -> Result<Machine> {
    let mut machine = Machine::new();
    machine.consult_module_string("core", CORE_PROLOG)?;
    Ok(machine)
}
```

---

## Summary

Scryer Prolog provides a powerful way to embed declarative logic programming in Rust. Key capabilities:

| Feature | Method |
|---------|--------|
| Load .pl files | `consult_module_file()` |
| Load from string | `consult_module_string()` |
| Add facts at runtime | `assertz()` via query |
| Remove facts | `retractall()` via query |
| Run queries | `run_query()` |
| Iterate solutions | Iterator over `LeafAnswer` |

Use Prolog when your problem involves:
- **Search** with backtracking (pathfinding, scheduling)
- **Constraints** over domains (layout, resource allocation)
- **Rules** that compose (access control, validation)
- **Pattern matching** on structures (parsing, type checking)
- **What not how** - declare constraints, let Prolog find solutions
