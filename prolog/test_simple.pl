%% Simple test file for Factorio constraint solver
%% Run with: scryer-prolog test_simple.pl

:- use_module(library(clpz)).
:- use_module(library(format)).

%% =============================================================================
%% Minimal Entity Database
%% =============================================================================

entity(assembler_2, 3, 3).
entity(inserter_fast, 1, 1).
entity(belt_blue, 1, 1).

%% =============================================================================
%% Core Spatial Constraints
%% =============================================================================

%% No overlap using disjunctive constraint
no_overlap(X1, Y1, W1, H1, X2, Y2, W2, H2) :-
    X1 + W1 #=< X2 #\/ X2 + W2 #=< X1 #\/
    Y1 + H1 #=< Y2 #\/ Y2 + H2 #=< Y1.

%% Check two placements don't overlap
check_no_overlap(p(T1, X1, Y1), p(T2, X2, Y2)) :-
    entity(T1, W1, H1),
    entity(T2, W2, H2),
    no_overlap(X1, Y1, W1, H1, X2, Y2, W2, H2).

%% =============================================================================
%% Simple 2-Assembler Layout Test
%% =============================================================================

%% Place two assemblers that don't overlap
test_two_assemblers(X1, Y1, X2, Y2) :-
    % Bound the search space
    X1 in 0..20, Y1 in 0..20,
    X2 in 0..20, Y2 in 0..20,

    % They must not overlap
    check_no_overlap(p(assembler_2, X1, Y1), p(assembler_2, X2, Y2)),

    % Find solution
    labeling([ff], [X1, Y1, X2, Y2]),

    format("Assembler 1 at (~d, ~d)~n", [X1, Y1]),
    format("Assembler 2 at (~d, ~d)~n", [X2, Y2]).

%% =============================================================================
%% Belt Line Test
%% =============================================================================

%% Generate a line of belts with constrained positions
belt_line(StartX, Y, Length, Belts) :-
    length(Belts, Length),
    belt_line_positions(Belts, StartX, Y).

belt_line_positions([], _, _).
belt_line_positions([p(belt_blue, X, Y)|Rest], X, Y) :-
    X1 #= X + 1,
    belt_line_positions(Rest, X1, Y).

test_belt_line :-
    belt_line(0, 5, 10, Belts),
    format("Belt line: ~w~n", [Belts]).

%% =============================================================================
%% Inserter Placement Test
%% =============================================================================

%% Inserter must be adjacent to assembler
inserter_adjacent_to_assembler(InsX, InsY, AsmX, AsmY) :-
    % Assembler is 3x3
    % Inserter can be on any of the 4 sides

    % Left side
    (InsX #= AsmX - 1, InsY in AsmY..AsmY+2) ;
    % Right side
    (InsX #= AsmX + 3, InsY in AsmY..AsmY+2) ;
    % Top side
    (InsY #= AsmY - 1, InsX in AsmX..AsmX+2) ;
    % Bottom side
    (InsY #= AsmY + 3, InsX in AsmX..AsmX+2).

test_inserter_adjacent :-
    AsmX = 5, AsmY = 5,
    InsX in 0..20, InsY in 0..20,
    inserter_adjacent_to_assembler(InsX, InsY, AsmX, AsmY),
    labeling([ff], [InsX, InsY]),
    format("Inserter at (~d, ~d) is adjacent to assembler at (~d, ~d)~n",
           [InsX, InsY, AsmX, AsmY]).

%% =============================================================================
%% Full Simple Layout
%% =============================================================================

%% Layout: assembler with input inserter, input belt, output inserter, output belt
simple_layout(Layout) :-
    % Assembler at center
    AsmX = 5, AsmY = 5,

    % Input inserter on west side, facing east (puts into assembler)
    InInsX in 0..20, InInsY in 0..20,
    InInsX #= AsmX - 1,
    InInsY #= AsmY + 1,  % middle of assembler

    % Input belt west of input inserter
    InBeltX #= InInsX - 1,
    InBeltY #= InInsY,

    % Output inserter on east side, facing east (takes from assembler)
    OutInsX #= AsmX + 3,
    OutInsY #= AsmY + 1,

    % Output belt east of output inserter
    OutBeltX #= OutInsX + 1,
    OutBeltY #= OutInsY,

    % Label all variables
    labeling([ff], [InInsX, InInsY, InBeltX, InBeltY, OutInsX, OutInsY, OutBeltX, OutBeltY]),

    Layout = [
        p(assembler_2, AsmX, AsmY),
        p(inserter_fast, InInsX, InInsY),
        p(belt_blue, InBeltX, InBeltY),
        p(inserter_fast, OutInsX, OutInsY),
        p(belt_blue, OutBeltX, OutBeltY)
    ].

test_simple_layout :-
    simple_layout(Layout),
    format("Simple layout:~n", []),
    print_placements(Layout).

print_placements([]).
print_placements([p(Type, X, Y)|Rest]) :-
    format("  ~w at (~d, ~d)~n", [Type, X, Y]),
    print_placements(Rest).

%% =============================================================================
%% Run All Tests
%% =============================================================================

run_tests :-
    format("=== Test 1: Two non-overlapping assemblers ===~n", []),
    (test_two_assemblers(_, _, _, _) -> true ; format("FAILED~n", [])),
    nl,

    format("=== Test 2: Belt line ===~n", []),
    (test_belt_line -> true ; format("FAILED~n", [])),
    nl,

    format("=== Test 3: Inserter adjacent to assembler ===~n", []),
    (test_inserter_adjacent -> true ; format("FAILED~n", [])),
    nl,

    format("=== Test 4: Simple layout ===~n", []),
    (test_simple_layout -> true ; format("FAILED~n", [])),
    nl,

    format("All tests completed.~n", []).

%% Entry point
:- initialization(run_tests).
