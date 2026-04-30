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
:- use_module(library(between)).
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
    ItemsPerSec #= (OutputCount * Speed * 100) // CraftTime.

%% Machines needed for target throughput
%% machines_needed(Recipe, MachineType, TargetPerSec, Count)
%% TargetPerSec is items/sec * 100
machines_needed(Recipe, MachineType, TargetPerSec, Count) :-
    items_per_second(Recipe, MachineType, PerMachine),
    PerMachine #> 0,
    Count #= (TargetPerSec + PerMachine - 1) // PerMachine.  % ceiling division

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
generate_machine_placements(MachineType, N, [placement(MachineType, _X, _Y, recipe(_Recipe))|Rest]) :-
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
    % Assemblers at Y=1 so they occupy Y=0,1,2 (3x3 centered)
    generate_row_machines(MachineType, Recipe, MachineCount, 0, 1, MachinePlacements),

    % Create inserter placements
    generate_row_inserters(MachineCount, 0, InserterPlacements),

    % Create belt placements
    generate_row_belts(MachineCount, 0, BeltPlacements),

    % Place power poles to cover the row
    TotalWidth is MachineCount * 5,
    generate_power_poles(TotalWidth, PowerPoles),

    % Combine all placements
    append([MachinePlacements, InserterPlacements, BeltPlacements, PowerPoles], Layout).

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

%% Generate power poles to cover a row
%% Medium power poles have 7x7 coverage (3.5 tile radius)
%% Place them every 6 tiles to ensure overlap
generate_power_poles(TotalWidth, Poles) :-
    generate_power_pole_line(0, TotalWidth, 6, -5, Poles).

generate_power_pole_line(X, MaxX, _, _, []) :- X >= MaxX, !.
generate_power_pole_line(X, MaxX, Spacing, Y, [placement(power_pole_medium, X, Y, none)|Rest]) :-
    X < MaxX,
    X1 is X + Spacing,
    generate_power_pole_line(X1, MaxX, Spacing, Y, Rest).

%% =============================================================================
%% Simple Belt Grid Solver
%% =============================================================================

%% Generate an NxN square of belts
%% solve_belt_square(N, Layout)
solve_belt_square(N, Layout) :-
    generate_belt_grid(N, N, 0, 0, Layout).

%% Generate belts around the perimeter of an NxN square (continuous loop)
%% solve_belt_perimeter(N, Layout) - hard-coded pattern (no constraints)
solve_belt_perimeter(N, Layout) :-
    N1 is N - 1,
    N2 is N - 2,

    % Top row: going east (X=0..N-2, Y=0)
    generate_belt_line(0, N1, 0, east, TopRow),

    % Top-right corner: turn south (X=N-1, Y=0)
    TopRightCorner = [placement(belt_blue, N1, 0, direction(south))],

    % Right column: going south (X=N-1, Y=1..N-2)
    findall(placement(belt_blue, N1, Y, direction(south)),
            between(1, N2, Y),
            RightCol),

    % Bottom-right corner: turn west (X=N-1, Y=N-1)
    BottomRightCorner = [placement(belt_blue, N1, N1, direction(west))],

    % Bottom row: going west (X=N-2..1, Y=N-1) - reversed order
    findall(placement(belt_blue, X, N1, direction(west)),
            (between(1, N2, X1), X is N2 - X1 + 1),
            BottomRow),

    % Bottom-left corner: turn north (X=0, Y=N-1)
    BottomLeftCorner = [placement(belt_blue, 0, N1, direction(north))],

    % Left column: going north (X=0, Y=N-2..1) - reversed order
    findall(placement(belt_blue, 0, Y, direction(north)),
            (between(1, N2, Y1), Y is N2 - Y1 + 1),
            LeftCol),

    append([TopRow, TopRightCorner, RightCol, BottomRightCorner, BottomRow, BottomLeftCorner, LeftCol], Layout).

%% Constrained version using CLP(Z) to test constraint solving
%% solve_belt_perimeter_constrained(N, Layout)
solve_belt_perimeter_constrained(N, Layout) :-
    NumBelts is (N - 1) * 4,  % Perimeter = 4 * (N-1)

    % Generate list of belt positions and directions
    length(Belts, NumBelts),

    % Constrain each belt to perimeter and assign direction
    constrain_perimeter_belts(Belts, N),

    % All positions must be distinct (no duplicates)
    all_positions_distinct(Belts),

    % Constrain belts to form continuous loop (including last->first)
    constrain_belt_loop_closed(Belts),

    % Label variables (search for solution)
    label_belts(Belts),

    % Convert to placement format
    belts_to_placements(Belts, Layout).

%% Constrain each belt to be on perimeter with valid direction
constrain_perimeter_belts([], _).
constrain_perimeter_belts([belt(X, Y, Dir)|Rest], N) :-
    N1 is N - 1,
    % Position must be on perimeter
    X in 0..N1,
    Y in 0..N1,
    (X #= 0 #\/ X #= N1 #\/ Y #= 0 #\/ Y #= N1),

    % Direction is 0=north, 1=east, 2=south, 3=west
    Dir in 0..3,

    constrain_perimeter_belts(Rest, N).

%% All positions must be distinct
all_positions_distinct([]).
all_positions_distinct([belt(X, Y, _)|Rest]) :-
    no_duplicate_position(X, Y, Rest),
    all_positions_distinct(Rest).

no_duplicate_position(_, _, []).
no_duplicate_position(X1, Y1, [belt(X2, Y2, _)|Rest]) :-
    (X1 #\= X2 #\/ Y1 #\= Y2),
    no_duplicate_position(X1, Y1, Rest).

%% Constrain belts to form a continuous loop (including last->first)
constrain_belt_loop_closed([]) :- !.
constrain_belt_loop_closed([First|Rest]) :-
    constrain_belt_loop_internal(First, [First|Rest]).

constrain_belt_loop_internal(_, [_]) :- !.
constrain_belt_loop_internal(First, [B1, B2|Rest]) :-
    constrain_belt_connection(B1, B2),
    (Rest = [] -> constrain_belt_connection(B2, First) ; true),
    constrain_belt_loop_internal(First, [B2|Rest]).

%% Constrain two adjacent belts to connect
constrain_belt_connection(belt(X1, Y1, Dir1), belt(X2, Y2, _Dir2)) :-
    % Dir1: 0=north, 1=east, 2=south, 3=west
    % Output of belt 1 must equal position of belt 2
    (Dir1 #= 0 #==> (X2 #= X1 #/\ Y2 #= Y1 - 1)),  % north
    (Dir1 #= 1 #==> (X2 #= X1 + 1 #/\ Y2 #= Y1)),  % east
    (Dir1 #= 2 #==> (X2 #= X1 #/\ Y2 #= Y1 + 1)),  % south
    (Dir1 #= 3 #==> (X2 #= X1 - 1 #/\ Y2 #= Y1)).  % west

%% Label belt variables
label_belts([]).
label_belts([belt(X, Y, Dir)|Rest]) :-
    label([X, Y, Dir]),
    label_belts(Rest).

%% Convert belt structures to placements
belts_to_placements([], []).
belts_to_placements([belt(X, Y, DirNum)|Rest], [placement(belt_blue, X, Y, direction(Dir))|PRest]) :-
    dir_num_to_atom(DirNum, Dir),
    belts_to_placements(Rest, PRest).

%% Direction number to atom
dir_num_to_atom(0, north).
dir_num_to_atom(1, east).
dir_num_to_atom(2, south).
dir_num_to_atom(3, west).

%% Direction atom to number
dir_atom_to_num(north, 0).
dir_atom_to_num(east, 1).
dir_atom_to_num(south, 2).
dir_atom_to_num(west, 3).
dir_atom_to_num(n, 0).
dir_atom_to_num(e, 1).
dir_atom_to_num(s, 2).
dir_atom_to_num(w, 3).

%% =============================================================================
%% Path Finding with Constraints
%% =============================================================================

%% Find a continuous belt path from (X1,Y1) to (X2,Y2)
%% solve_belt_path(X1, Y1, X2, Y2, MaxLength, Layout)
solve_belt_path(X1, Y1, X2, Y2, MaxLength, Layout) :-
    % Try paths of increasing length (Manhattan distance is minimum)
    ManhattanDist is abs(X2 - X1) + abs(Y2 - Y1),
    between(ManhattanDist, MaxLength, PathLength),

    % Build path recursively
    build_path(X1, Y1, X2, Y2, PathLength, Belts),

    % Convert to placements
    belts_to_placements(Belts, Layout),
    !.  % Stop at first solution

%% Find path with constrained start and end directions
%% solve_belt_path_directed(X1, Y1, Dir1, X2, Y2, Dir2, MaxLength, Layout)
solve_belt_path_directed(X1, Y1, Dir1Name, X2, Y2, Dir2Name, MaxLength, Layout) :-
    % Convert direction names to numbers
    dir_atom_to_num(Dir1Name, Dir1),
    dir_atom_to_num(Dir2Name, Dir2),

    % Try paths of increasing length
    ManhattanDist is abs(X2 - X1) + abs(Y2 - Y1),
    between(ManhattanDist, MaxLength, PathLength),

    % Build path with constrained directions
    build_path_directed(X1, Y1, Dir1, X2, Y2, Dir2, PathLength, Belts),

    % Convert to placements
    belts_to_placements(Belts, Layout),
    !.  % Stop at first solution

%% Check if two belts are continuous (second belt receives from first belt)
belts_continuous(belt(X1, Y1, Dir1), belt(X2, Y2, _Dir2)) :-
    % First belt's output must reach second belt's position
    next_position(X1, Y1, Dir1, X2, Y2).

%% Check if a list of belts forms a continuous path
path_continuous([_]).
path_continuous([B1, B2|Rest]) :-
    belts_continuous(B1, B2),
    path_continuous([B2|Rest]).

%% Build path of specific length
build_path(X1, Y1, X2, Y2, 1, [belt(X1, Y1, Dir)]) :-
    % Single belt must reach target - try each direction
    member(Dir, [0, 1, 2, 3]),
    next_position(X1, Y1, Dir, X2, Y2).

build_path(X1, Y1, X2, Y2, Length, [belt(X1, Y1, Dir)|Rest]) :-
    Length > 1,
    % Try each direction
    member(Dir, [0, 1, 2, 3]),  % north, east, south, west

    % Calculate next position based on direction
    next_position(X1, Y1, Dir, NextX, NextY),

    % Build rest of path
    Length1 is Length - 1,
    build_path(NextX, NextY, X2, Y2, Length1, Rest).

%% Calculate next position given current position and direction
next_position(X, Y, 0, X, Y1) :- Y1 #= Y - 1.  % north
next_position(X, Y, 1, X1, Y) :- X1 #= X + 1.  % east
next_position(X, Y, 2, X, Y1) :- Y1 #= Y + 1.  % south
next_position(X, Y, 3, X1, Y) :- X1 #= X - 1.  % west

%% Build path with constrained start and end directions - CLP(Z) VERSION
%% Start at (X1,Y1) facing Dir1, end at (X2,Y2) facing Dir2
build_path_directed(X1, Y1, Dir1, X2, Y2, Dir2, Length, Path) :-
    % Calculate bounds
    ManhattanDist is abs(X2 - X1) + abs(Y2 - Y1),
    Buffer is ManhattanDist,
    MinX is min(X1, X2) - Buffer,
    MaxX is max(X1, X2) + Buffer,
    MinY is min(Y1, Y2) - Buffer,
    MaxY is max(Y1, Y2) + Buffer,

    % Create path structure
    length(Path, Length),
    Path = [belt(X1, Y1, Dir1)|_],

    % Constrain all belt domains
    constrain_all_belts_in_bounds(Path, MinX, MaxX, MinY, MaxY),

    % Constrain final belt position and direction
    append(_, [belt(X2, Y2, Dir2)], Path),

    % Constrain penultimate belt
    (Length > 1 ->
        (reverse_position(X2, Y2, Dir2, PenX, PenY),
         append(_, [belt(PenX, PenY, _PenDir), belt(X2, Y2, Dir2)], Path))
    ; true),

    % Constrain continuity
    constrain_path_continuous(Path),

    % Constrain distinct positions
    constrain_positions_distinct(Path),

    % Label variables
    extract_belt_vars(Path, Vars),
    label(Vars).

%% Count turns in a path (direction changes)
count_turns(Path, Turns) :-
    count_turns_acc(Path, 0, Turns).

count_turns_acc([], Acc, Acc).
count_turns_acc([_], Acc, Acc).
count_turns_acc([belt(_, _, Dir1), belt(_, _, Dir2)|Rest], Acc, Turns) :-
    % If direction changes, increment turn counter
    (Dir1 #\= Dir2 #<==> TurnHappened),
    AccNext #= Acc + TurnHappened,
    count_turns_acc([belt(_, _, Dir2)|Rest], AccNext, Turns).

%% Constrain all belts to be within bounds
constrain_all_belts_in_bounds([], _, _, _, _).
constrain_all_belts_in_bounds([belt(X, Y, Dir)|Rest], MinX, MaxX, MinY, MaxY) :-
    X in MinX..MaxX,
    Y in MinY..MaxY,
    Dir in 0..3,
    constrain_all_belts_in_bounds(Rest, MinX, MaxX, MinY, MaxY).

%% Constrain belt domains with specific bounds
constrain_belt_domains_bounded([], _MinX, _MaxX, _MinY, _MaxY).
constrain_belt_domains_bounded([belt(X, Y, Dir)|Rest], MinX, MaxX, MinY, MaxY) :-
    % Position constraints within bounding box
    X in MinX..MaxX,
    Y in MinY..MaxY,
    % Direction constraints
    Dir in 0..3,  % north=0, east=1, south=2, west=3
    constrain_belt_domains_bounded(Rest, MinX, MaxX, MinY, MaxY).

%% Constrain belt domains (positions and directions)
constrain_belt_domains([]).
constrain_belt_domains([belt(X, Y, Dir)|Rest]) :-
    % Position constraints (reasonable bounds for pathfinding)
    X in -20..20,
    Y in -20..20,
    % Direction constraints
    Dir in 0..3,  % north=0, east=1, south=2, west=3
    constrain_belt_domains(Rest).

%% Constrain positions to be distinct
constrain_positions_distinct([]).
constrain_positions_distinct([belt(X, Y, _)|Rest]) :-
    constrain_position_not_in(X, Y, Rest),
    constrain_positions_distinct(Rest).

constrain_position_not_in(_X, _Y, []).
constrain_position_not_in(X1, Y1, [belt(X2, Y2, _)|Rest]) :-
    (X1 #\= X2 #\/ Y1 #\= Y2),
    constrain_position_not_in(X1, Y1, Rest).

%% Constrain path to be continuous
constrain_path_continuous([_]).
constrain_path_continuous([B1, B2|Rest]) :-
    constrain_belt_connection(B1, B2),
    constrain_path_continuous([B2|Rest]).

%% Label belt variables
label_belt_path(Path) :-
    extract_belt_vars(Path, Vars),
    label([ff], Vars).  % Use first-fail strategy for better search

%% Extract all variables from belt path
extract_belt_vars([], []).
extract_belt_vars([belt(X, Y, Dir)|Rest], [X, Y, Dir|RestVars]) :-
    extract_belt_vars(Rest, RestVars).

%% Build path from (X1,Y1) with start direction Dir1 to (TargetX,TargetY)
build_path_with_start_dir(X1, Y1, Dir1, TargetX, TargetY, 1, [belt(X1, Y1, Dir1)]) :-
    % Single belt
    X1 = TargetX, Y1 = TargetY.

build_path_with_start_dir(X1, Y1, Dir1, TargetX, TargetY, Length, [belt(X1, Y1, Dir1)|Rest]) :-
    Length > 1,
    next_position(X1, Y1, Dir1, NextX, NextY),
    Length1 is Length - 1,
    build_path_ending_at(NextX, NextY, TargetX, TargetY, Length1, Rest).

%% Build path ending at target, avoiding a specific position
build_path_avoiding(X1, Y1, X2, Y2, AvoidX, AvoidY, Length, [belt(X1, Y1, Dir)|Rest]) :-
    % Try each direction
    member(Dir, [0, 1, 2, 3]),
    next_position(X1, Y1, Dir, NextX, NextY),
    % Make sure we don't hit the avoid position (unless it's the final target)
    ((NextX = AvoidX, NextY = AvoidY) -> (NextX = X2, NextY = Y2) ; true),
    % Build rest of path
    (Length = 1 ->
        (NextX = X2, NextY = Y2, Rest = [])
    ;
        (Length1 is Length - 1,
         build_path_avoiding(NextX, NextY, X2, Y2, AvoidX, AvoidY, Length1, Rest))
    ).

%% Calculate the position that would reach target with given direction
reverse_position(X, Y, 0, X, Y1) :- Y1 #= Y + 1.  % north: came from south
reverse_position(X, Y, 1, X1, Y) :- X1 #= X - 1.  % east: came from west
reverse_position(X, Y, 2, X, Y1) :- Y1 #= Y - 1.  % south: came from north
reverse_position(X, Y, 3, X1, Y) :- X1 #= X + 1.  % west: came from east

%% Find direction needed to reach target from source
direction_to_reach(X1, Y1, X2, Y2, Dir) :-
    member(Dir, [0, 1, 2, 3]),
    next_position(X1, Y1, Dir, X2, Y2).

%% Build path where last belt IS at target position (not just outputs to it)
build_path_ending_at(_X1, _Y1, _X2, _Y2, 0, []) :-
    % Zero-length path is empty
    !.

build_path_ending_at(X1, Y1, X2, Y2, 1, [belt(X1, Y1, Dir)]) :-
    % Single belt: must be at both positions (direction doesn't matter for connectivity to next)
    X1 = X2, Y1 = Y2,
    member(Dir, [0, 1, 2, 3]).

build_path_ending_at(X1, Y1, X2, Y2, Length, [belt(X1, Y1, Dir)|Rest]) :-
    Length > 1,
    % Try each direction
    member(Dir, [0, 1, 2, 3]),
    next_position(X1, Y1, Dir, NextX, NextY),
    % Build rest of path
    Length1 is Length - 1,
    build_path_ending_at(NextX, NextY, X2, Y2, Length1, Rest).

%% Constrain that a belt can reach target position
constrain_belt_reaches(belt(X, Y, Dir), TargetX, TargetY) :-
    % Belt output must reach target
    (Dir #= 0 #==> (TargetX #= X #/\ TargetY #= Y - 1)),  % north
    (Dir #= 1 #==> (TargetX #= X + 1 #/\ TargetY #= Y)),  % east
    (Dir #= 2 #==> (TargetX #= X #/\ TargetY #= Y + 1)),  % south
    (Dir #= 3 #==> (TargetX #= X - 1 #/\ TargetY #= Y)).  % west

%% Constrain belts to form continuous path (not a loop)
constrain_belt_path([]).
constrain_belt_path([_]).
constrain_belt_path([B1, B2|Rest]) :-
    constrain_belt_connection(B1, B2),
    constrain_belt_path([B2|Rest]).

%% Generate an MxN grid of belts starting at (StartX, StartY)
generate_belt_grid(Width, Height, StartX, StartY, Belts) :-
    EndX is StartX + Width,
    EndY is StartY + Height,
    findall(
        placement(belt_blue, X, Y, direction(east)),
        (between(StartX, EndX - 1, X), between(StartY, EndY - 1, Y)),
        Belts
    ).

%% =============================================================================
%% Belt Balancer Solver
%% =============================================================================

%% Solve for an N-to-M belt balancer
%% balancer(InputCount, OutputCount, Layout)
solve_balancer(InputCount, OutputCount, Layout) :-
    % Create input belts
    generate_input_belts(InputCount, 0, InputBelts),

    % Create splitter network
    generate_balancer_splitters(InputCount, OutputCount, 0, Splitters),

    % Create output belts
    OutputStartX is 10,
    generate_output_belts(OutputCount, OutputStartX, OutputBelts),

    % Combine all placements
    append([InputBelts, Splitters, OutputBelts], Layout).

%% Generate input belt lanes (vertical belts going east)
generate_input_belts(0, _, []) :- !.
generate_input_belts(N, Y, Belts) :-
    N > 0,
    Y1 is Y * 2,  % Space belts 2 tiles apart
    generate_belt_line(0, 3, Y1, east, LaneBelts),
    N1 is N - 1,
    Y2 is Y + 1,
    generate_input_belts(N1, Y2, RestBelts),
    append(LaneBelts, RestBelts, Belts).

%% Generate output belt lanes
generate_output_belts(0, _, []) :- !.
generate_output_belts(N, StartX, Belts) :-
    N > 0,
    Y is (N - 1) * 2,
    EndX is StartX + 3,
    generate_belt_line(StartX, EndX, Y, east, LaneBelts),
    N1 is N - 1,
    generate_output_belts(N1, StartX, RestBelts),
    append(LaneBelts, RestBelts, Belts).

%% Generate splitter network for balancing
%% For a simple N-M balancer, create a grid of splitters
generate_balancer_splitters(InputCount, OutputCount, StartX, Splitters) :-
    SplitterX is StartX + 3,

    % For 5-6 balancer, use a simple two-stage design
    InputCount =:= 5,
    OutputCount =:= 6,
    !,
    generate_5_6_balancer(SplitterX, Splitters).

generate_balancer_splitters(_, _, _, []).  % Fallback for unsupported balancers

%% Specific 5-6 balancer design
%% Strategy: Split lane 3 (middle) into two paths to create 6 outputs
generate_5_6_balancer(X, Splitters) :-
    % Stage 1: Split middle input (lane 2, Y=4) to create extra lane
    X1 is X,
    % Splitter at position (X1, 4) splits lane 2
    Splitter1 = placement(splitter_blue, X1, 4, direction(east)),

    % Stage 2: Additional splitters for balancing
    X2 is X1 + 3,
    Splitter2 = placement(splitter_blue, X2, 2, direction(east)),
    Splitter3 = placement(splitter_blue, X2, 6, direction(east)),

    % Stage 3: Final balancing stage
    X3 is X2 + 3,
    Splitter4 = placement(splitter_blue, X3, 0, direction(east)),
    Splitter5 = placement(splitter_blue, X3, 8, direction(east)),

    Splitters = [Splitter1, Splitter2, Splitter3, Splitter4, Splitter5].

%% =============================================================================
%% Main Solver Entry Point
%% =============================================================================

%% solve(Goal, Layout)
%% Goal can be:
%%   - goal(Item, ThroughputPerSec) where throughput is items/sec * 100
%%   - balancer(InputCount, OutputCount) for belt balancers
%%   - belt_square(N) for NxN grid of belts
%%   - belt_perimeter(N) for perimeter of NxN square
%%   - belt_line(Length) for a line of belts
%% Layout = list of placement/4 terms

solve(belt_line(Length), Layout) :-
    % Simple belt line
    generate_belt_line(0, Length, 0, east, Layout).

solve(belt_square(N), Layout) :-
    % Belt grid solver
    solve_belt_square(N, Layout).

solve(belt_perimeter(N), Layout) :-
    % Belt perimeter solver (hard-coded pattern)
    solve_belt_perimeter(N, Layout).

solve(belt_perimeter_constrained(N), Layout) :-
    % Belt perimeter using actual CLP(Z) constraints
    solve_belt_perimeter_constrained(N, Layout).

solve(belt_path(X1, Y1, X2, Y2, MaxLength), Layout) :-
    % Find path from (X1,Y1) to (X2,Y2)
    solve_belt_path(X1, Y1, X2, Y2, MaxLength, Layout).

solve(belt_path_directed(X1, Y1, Dir1, X2, Y2, Dir2, MaxLength), Layout) :-
    % Find path with constrained start and end directions
    solve_belt_path_directed(X1, Y1, Dir1, X2, Y2, Dir2, MaxLength, Layout).

solve(balancer(InputCount, OutputCount), Layout) :-
    % Belt balancer solver
    solve_balancer(InputCount, OutputCount, Layout).

solve(goal(Item, Throughput), Layout) :-
    % Production chain solver
    solve_simple_row(Item, Throughput, Layout).

%% =============================================================================
%% Output Formatting
%% =============================================================================

%% Convert underscores to dashes in entity names for DSL output
atom_underscores_to_dashes(AtomIn, AtomOut) :-
    atom_chars(AtomIn, CharsIn),
    maplist(underscore_to_dash, CharsIn, CharsOut),
    atom_chars(AtomOut, CharsOut).

underscore_to_dash('_', '-') :- !.
underscore_to_dash(C, C).

%% Convert placement to DSL command string and print it
placement_to_dsl(placement(Type, X, Y, recipe(Recipe))) :-
    atom_underscores_to_dashes(Type, DslType),
    atom_underscores_to_dashes(Recipe, DslRecipe),
    write(DslType), write(' '),
    write(X), write(' '),
    write(Y), write(' '),
    write('recipe:'), write(DslRecipe), nl.

placement_to_dsl(placement(Type, X, Y, direction(Dir))) :-
    atom_underscores_to_dashes(Type, DslType),
    direction_short(Dir, Short),
    write(DslType), write(' '),
    write(X), write(' '),
    write(Y), write(' '),
    write(':'), write(Short), nl.

placement_to_dsl(placement(Type, X, Y, none)) :-
    atom_underscores_to_dashes(Type, DslType),
    write(DslType), write(' '),
    write(X), write(' '),
    write(Y), nl.

direction_short(north, n).
direction_short(south, s).
direction_short(east, e).
direction_short(west, w).

%% Print layout as DSL
print_layout([]).
print_layout([P|Rest]) :-
    placement_to_dsl(P),
    print_layout(Rest).

%% =============================================================================
%% Test Queries
%% =============================================================================

%% Test: solve for green circuits at 45 items/sec (1 blue belt)
test_green_circuits :-
    write('Solving for green circuits at 45/sec...'), nl,
    solve(goal(green_circuit, 4500), Layout),
    length(Layout, Count),
    write('Solution found with '), write(Count), write(' placements:'), nl,
    print_layout(Layout).

%% Test: calculate machines needed
test_machine_count :-
    machines_needed(green_circuit, assembler_2, 4500, Count),
    write('Need '), write(Count), write(' assembler-2 for green circuits at 45/sec'), nl.

%% Test: production rate
test_production_rate :-
    items_per_second(green_circuit, assembler_2, Rate),
    RateFloat is Rate / 100,
    write('Assembler-2 produces '), write(RateFloat), write(' green circuits/sec'), nl.
