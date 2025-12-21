%% Factorio Constraint Builder - Prolog Constraint Model
%%
%% This module implements constraint satisfaction for Factorio factory layouts.
%% Uses CLP(Z) for integer constraints over positions and throughput.
%%
%% Usage (in Scryer Prolog):
%%   ?- consult('factorio.pl').
%%   ?- solve(goal(green_circuit, 45), Layout).

:- use_module(library(clpz)).
:- use_module(library(lists)).
:- use_module(library(dcgs)).
:- use_module(library(format)).

%% =============================================================================
%% Entity Database
%% =============================================================================

%% entity(Name, Category, Width, Height, CraftingSpeed, ModuleSlots)
%% CraftingSpeed and ModuleSlots are 0 for non-crafting entities

% Assemblers
entity(assembler_1, assembler, 3, 3, 50, 0).   % 0.5 speed as integer * 100
entity(assembler_2, assembler, 3, 3, 75, 2).   % 0.75 speed
entity(assembler_3, assembler, 3, 3, 125, 4).  % 1.25 speed

% Furnaces
entity(stone_furnace, furnace, 2, 2, 100, 0).
entity(steel_furnace, furnace, 2, 2, 200, 0).
entity(electric_furnace, furnace, 2, 2, 200, 2).

% Belts (throughput in items/sec * 100)
entity(belt_yellow, belt, 1, 1, 1500, 0).   % 15 items/sec
entity(belt_red, belt, 1, 1, 3000, 0).      % 30 items/sec
entity(belt_blue, belt, 1, 1, 4500, 0).     % 45 items/sec

% Inserters
entity(inserter_burner, inserter, 1, 1, 60, 0).   % items/sec * 100
entity(inserter_basic, inserter, 1, 1, 83, 0).
entity(inserter_long, inserter, 1, 1, 120, 0).
entity(inserter_fast, inserter, 1, 1, 231, 0).
entity(inserter_stack, inserter, 1, 1, 461, 0).   % with capacity bonus

% Power poles (supply area as "speed")
entity(power_pole_small, power, 1, 1, 5, 0).   % 5x5 area
entity(power_pole_medium, power, 1, 1, 7, 0).  % 7x7 area
entity(power_pole_big, power, 1, 1, 4, 0).     % 4x4 area (but long wire)
entity(power_pole_substation, power, 2, 2, 18, 0).  % 18x18 area

%% =============================================================================
%% Recipe Database
%% =============================================================================

%% recipe(Name, Inputs, Outputs, CraftingTime, Category)
%% CraftingTime in ticks (60 ticks = 1 second), stored as centiseconds
%% Inputs/Outputs: list of (Item, Count) pairs

% Basic intermediates
recipe(iron_gear,
       [(iron_plate, 2)],
       [(iron_gear, 1)],
       50, assembling).  % 0.5 seconds

recipe(copper_cable,
       [(copper_plate, 1)],
       [(copper_cable, 2)],
       50, assembling).

recipe(iron_stick,
       [(iron_plate, 1)],
       [(iron_stick, 2)],
       50, assembling).

% Circuits
recipe(green_circuit,
       [(iron_plate, 1), (copper_cable, 3)],
       [(green_circuit, 1)],
       50, assembling).

recipe(red_circuit,
       [(green_circuit, 2), (plastic, 2), (copper_cable, 4)],
       [(red_circuit, 1)],
       600, assembling).  % 6 seconds

recipe(blue_circuit,
       [(red_circuit, 2), (green_circuit, 20), (sulfuric_acid, 5)],
       [(blue_circuit, 1)],
       1000, assembling).  % 10 seconds

% Science packs
recipe(red_science,
       [(copper_plate, 1), (iron_gear, 1)],
       [(red_science, 1)],
       500, assembling).

recipe(green_science,
       [(inserter_basic, 1), (belt_yellow, 1)],
       [(green_science, 1)],
       600, assembling).

% Smelting
recipe(iron_plate,
       [(iron_ore, 1)],
       [(iron_plate, 1)],
       350, smelting).  % 3.5 seconds in stone furnace

recipe(copper_plate,
       [(copper_ore, 1)],
       [(copper_plate, 1)],
       350, smelting).

recipe(steel_plate,
       [(iron_plate, 5)],
       [(steel_plate, 1)],
       1750, smelting).  % 17.5 seconds

%% =============================================================================
%% Direction Handling
%% =============================================================================

direction(north).
direction(south).
direction(east).
direction(west).

% Direction to delta X, delta Y
direction_delta(north, 0, -1).
direction_delta(south, 0, 1).
direction_delta(east, 1, 0).
direction_delta(west, -1, 0).

opposite_direction(north, south).
opposite_direction(south, north).
opposite_direction(east, west).
opposite_direction(west, east).

%% =============================================================================
%% Spatial Constraints
%% =============================================================================

%% Position must be within bounds
position_bounded(X, Y, MaxX, MaxY) :-
    X in 0..MaxX,
    Y in 0..MaxY.

%% Entity footprint at position
%% entity_footprint(EntityType, X, Y, X1, Y1, X2, Y2)
%% Returns bounding box (X1,Y1) to (X2,Y2) for entity at (X,Y)
entity_footprint(EntityType, X, Y, X1, Y1, X2, Y2) :-
    entity(EntityType, _, W, H, _, _),
    X1 #= X,
    Y1 #= Y,
    X2 #= X + W - 1,
    Y2 #= Y + H - 1.

%% No overlap between two entities
%% Uses disjunctive constraint: at least one separation must hold
no_overlap(X1, Y1, W1, H1, X2, Y2, W2, H2) :-
    X1 + W1 #=< X2 #\/ X2 + W2 #=< X1 #\/
    Y1 + H1 #=< Y2 #\/ Y2 + H2 #=< Y1.

%% Check two placements don't overlap
placements_no_overlap(placement(Type1, X1, Y1, _), placement(Type2, X2, Y2, _)) :-
    entity(Type1, _, W1, H1, _, _),
    entity(Type2, _, W2, H2, _, _),
    no_overlap(X1, Y1, W1, H1, X2, Y2, W2, H2).

%% All placements pairwise non-overlapping
all_no_overlap([]).
all_no_overlap([_]).
all_no_overlap([P1|Rest]) :-
    maplist(placements_no_overlap(P1), Rest),
    all_no_overlap(Rest).

%% =============================================================================
%% Connectivity Constraints
%% =============================================================================

%% Inserter pickup and dropoff positions
%% inserter_reaches(InsX, InsY, Dir, PickupX, PickupY, DropoffX, DropoffY)
inserter_reaches(InsX, InsY, north, PickupX, PickupY, DropoffX, DropoffY) :-
    PickupX #= InsX, PickupY #= InsY + 1,
    DropoffX #= InsX, DropoffY #= InsY - 1.

inserter_reaches(InsX, InsY, south, PickupX, PickupY, DropoffX, DropoffY) :-
    PickupX #= InsX, PickupY #= InsY - 1,
    DropoffX #= InsX, DropoffY #= InsY + 1.

inserter_reaches(InsX, InsY, east, PickupX, PickupY, DropoffX, DropoffY) :-
    PickupX #= InsX - 1, PickupY #= InsY,
    DropoffX #= InsX + 1, DropoffY #= InsY.

inserter_reaches(InsX, InsY, west, PickupX, PickupY, DropoffX, DropoffY) :-
    PickupX #= InsX + 1, PickupY #= InsY,
    DropoffX #= InsX - 1, DropoffY #= InsY.

%% Long inserter reaches (2 tiles instead of 1)
long_inserter_reaches(InsX, InsY, north, PickupX, PickupY, DropoffX, DropoffY) :-
    PickupX #= InsX, PickupY #= InsY + 2,
    DropoffX #= InsX, DropoffY #= InsY - 1.

long_inserter_reaches(InsX, InsY, south, PickupX, PickupY, DropoffX, DropoffY) :-
    PickupX #= InsX, PickupY #= InsY - 2,
    DropoffX #= InsX, DropoffY #= InsY + 1.

long_inserter_reaches(InsX, InsY, east, PickupX, PickupY, DropoffX, DropoffY) :-
    PickupX #= InsX - 2, PickupY #= InsY,
    DropoffX #= InsX + 1, DropoffY #= InsY.

long_inserter_reaches(InsX, InsY, west, PickupX, PickupY, DropoffX, DropoffY) :-
    PickupX #= InsX + 2, PickupY #= InsY,
    DropoffX #= InsX - 1, DropoffY #= InsY.

%% Belt continuation - next belt position given direction
belt_next(X, Y, Dir, NextX, NextY) :-
    direction_delta(Dir, DX, DY),
    NextX #= X + DX,
    NextY #= Y + DY.

%% Check if position is adjacent to an assembler (for inserter placement)
adjacent_to_assembler(AsmX, AsmY, PosX, PosY) :-
    % Assembler is 3x3, check if PosX,PosY is adjacent to its border
    (PosX #= AsmX - 1, PosY in AsmY..AsmY+2) ;
    (PosX #= AsmX + 3, PosY in AsmY..AsmY+2) ;
    (PosY #= AsmY - 1, PosX in AsmX..AsmX+2) ;
    (PosY #= AsmY + 3, PosX in AsmX..AsmX+2).

%% =============================================================================
%% Production Calculations
%% =============================================================================

%% Items produced per second (scaled by 100 for integer math)
%% items_per_second(Recipe, MachineType, ItemsPerSec)
items_per_second(Recipe, MachineType, ItemsPerSec) :-
    recipe(Recipe, _, Outputs, CraftTime, _),
    entity(MachineType, _, _, _, Speed, _),
    member((_, OutputCount), Outputs),
    % ItemsPerSec = (OutputCount * Speed * 100) / CraftTime
    ItemsPerSec is (OutputCount * Speed * 100) div CraftTime.

%% Machines needed for target throughput
%% machines_needed(Recipe, MachineType, TargetPerSec, Count)
%% TargetPerSec is items/sec * 100
machines_needed(Recipe, MachineType, TargetPerSec, Count) :-
    items_per_second(Recipe, MachineType, PerMachine),
    PerMachine > 0,
    Count is (TargetPerSec + PerMachine - 1) div PerMachine.  % ceiling division

%% =============================================================================
%% Production Chain Analysis
%% =============================================================================

%% Get all ingredients for a recipe (recursively)
recipe_ingredients(Recipe, Ingredients) :-
    recipe(Recipe, Inputs, _, _, _),
    Ingredients = Inputs.

%% Check if an item is a raw resource (no recipe to make it)
is_raw_resource(Item) :-
    \+ recipe(Item, _, _, _, _).

%% Build production chain for an item
%% production_chain(Item, Chain)
%% Chain is a list of recipes needed (in dependency order)
production_chain(Item, []) :-
    is_raw_resource(Item), !.

production_chain(Item, [Item|RestChain]) :-
    recipe(Item, Inputs, _, _, _),
    findall(SubChain, (
        member((SubItem, _), Inputs),
        production_chain(SubItem, SubChain)
    ), SubChains),
    append(SubChains, RestChain).

%% =============================================================================
%% Placement Generation
%% =============================================================================

%% A placement is: placement(EntityType, X, Y, Extra)
%% Extra can be: recipe(RecipeName), direction(Dir), or none

%% Generate N placements of a machine type
generate_machine_placements(_, 0, []) :- !.
generate_machine_placements(MachineType, N, [placement(MachineType, X, Y, recipe(Recipe))|Rest]) :-
    N > 0,
    N1 is N - 1,
    generate_machine_placements(MachineType, N1, Rest).

%% =============================================================================
%% Simple Layout Solver
%% =============================================================================

%% Solve for a simple row of assemblers with input/output belts
%% This is a minimal example - real solver would be more sophisticated

solve_simple_row(Recipe, TargetPerSec, Layout) :-
    % Determine machine type and count
    MachineType = assembler_2,
    machines_needed(Recipe, MachineType, TargetPerSec, MachineCount),

    % Create machine placements in a row
    generate_row_machines(MachineType, Recipe, MachineCount, 0, 0, MachinePlacements),

    % Create inserter placements
    generate_row_inserters(MachineCount, 0, InserterPlacements),

    % Create belt placements
    generate_row_belts(MachineCount, 0, BeltPlacements),

    % Combine all placements
    append([MachinePlacements, InserterPlacements, BeltPlacements], Layout).

%% Generate machines in a row (spaced for inserters)
generate_row_machines(_, _, 0, _, _, []) :- !.
generate_row_machines(Type, Recipe, N, X, Y, [placement(Type, X, Y, recipe(Recipe))|Rest]) :-
    N > 0,
    N1 is N - 1,
    X1 is X + 5,  % 3 for machine + 2 for spacing
    generate_row_machines(Type, Recipe, N1, X1, Y, Rest).

%% Generate inserters for row layout
generate_row_inserters(0, _, []) :- !.
generate_row_inserters(N, X, [
    placement(inserter_fast, InsX1, -1, direction(south)),  % input inserter
    placement(inserter_fast, InsX2, 3, direction(south))   % output inserter
    |Rest]) :-
    N > 0,
    InsX1 is X + 1,
    InsX2 is X + 1,
    N1 is N - 1,
    X1 is X + 5,
    generate_row_inserters(N1, X1, Rest).

%% Generate belts for row layout
generate_row_belts(MachineCount, StartX, Belts) :-
    TotalWidth is MachineCount * 5,
    EndX is StartX + TotalWidth,
    generate_belt_line(StartX, EndX, -2, east, InputBelts),
    generate_belt_line(StartX, EndX, 4, east, OutputBelts),
    append(InputBelts, OutputBelts, Belts).

generate_belt_line(X, EndX, _, _, []) :- X >= EndX, !.
generate_belt_line(X, EndX, Y, Dir, [placement(belt_blue, X, Y, direction(Dir))|Rest]) :-
    X < EndX,
    X1 is X + 1,
    generate_belt_line(X1, EndX, Y, Dir, Rest).

%% =============================================================================
%% Main Solver Entry Point
%% =============================================================================

%% solve(Goal, Layout)
%% Goal = goal(Item, ThroughputPerSec)  where throughput is items/sec * 100
%% Layout = list of placement/4 terms

solve(goal(Item, Throughput), Layout) :-
    % For now, use simple row solver
    solve_simple_row(Item, Throughput, Layout).

%% =============================================================================
%% Output Formatting
%% =============================================================================

%% Convert placement to DSL command string
placement_to_dsl(placement(Type, X, Y, recipe(Recipe)), Command) :-
    atom_codes(Type, TypeCodes),
    atom_codes(Recipe, RecipeCodes),
    format_to_codes(Command, "~a ~d ~d recipe:~a", [TypeCodes, X, Y, RecipeCodes]).

placement_to_dsl(placement(Type, X, Y, direction(Dir)), Command) :-
    direction_short(Dir, Short),
    format_to_codes(Command, "~a ~d ~d :~a", [Type, X, Y, Short]).

placement_to_dsl(placement(Type, X, Y, none), Command) :-
    format_to_codes(Command, "~a ~d ~d", [Type, X, Y]).

direction_short(north, n).
direction_short(south, s).
direction_short(east, e).
direction_short(west, w).

%% Print layout as DSL
print_layout([]).
print_layout([P|Rest]) :-
    placement_to_dsl(P, Cmd),
    format("~s~n", [Cmd]),
    print_layout(Rest).

%% =============================================================================
%% Test Queries
%% =============================================================================

%% Test: solve for green circuits at 45 items/sec (1 blue belt)
test_green_circuits :-
    format("Solving for green circuits at 45/sec...~n", []),
    solve(goal(green_circuit, 4500), Layout),
    format("Solution found with ~d placements:~n", [Layout]),
    print_layout(Layout).

%% Test: calculate machines needed
test_machine_count :-
    machines_needed(green_circuit, assembler_2, 4500, Count),
    format("Need ~d assembler-2 for green circuits at 45/sec~n", [Count]).

%% Test: production rate
test_production_rate :-
    items_per_second(green_circuit, assembler_2, Rate),
    RateFloat is Rate / 100,
    format("Assembler-2 produces ~2f green circuits/sec~n", [RateFloat]).
