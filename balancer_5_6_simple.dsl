; Simple 5-6 Belt Splitter
; Just creates 6 lanes from 5, minimal balancing

; === 5 Input Lanes ===
belt-blue 0 0 :east
belt-blue 1 0 :east
belt-blue 2 0 :east

belt-blue 0 2 :east
belt-blue 1 2 :east
belt-blue 2 2 :east

belt-blue 0 4 :east
belt-blue 1 4 :east
belt-blue 2 4 :east

belt-blue 0 6 :east
belt-blue 1 6 :east
belt-blue 2 6 :east

belt-blue 0 8 :east
belt-blue 1 8 :east
belt-blue 2 8 :east

; === Connect to splitter ===
belt-blue 3 0 :east
belt-blue 3 2 :east
belt-blue 3 4 :east
belt-blue 3 6 :east
belt-blue 3 8 :east

; === Split middle lane (Y=4) ===
; Splitter takes input at Y=4, outputs at Y=4 and Y=5
splitter-blue 4 4 :east out-priority:right

; === Route Y=5 output down to Y=10 ===
belt-blue 6 5 :south
underground-belt-blue 6 6 :south
underground-belt-blue 6 8 :south
belt-blue 6 9 :south
belt-blue 6 10 :south

; === Pass through other lanes ===
belt-blue 6 0 :east
belt-blue 6 2 :east
belt-blue 6 4 :east
belt-blue 6 6 :east
belt-blue 6 8 :east

; === 6 Output Lanes ===
belt-blue 7 0 :east
belt-blue 8 0 :east

belt-blue 7 2 :east
belt-blue 8 2 :east

belt-blue 7 4 :east
belt-blue 8 4 :east

belt-blue 7 6 :east
belt-blue 8 6 :east

belt-blue 7 8 :east
belt-blue 8 8 :east

belt-blue 7 10 :east
belt-blue 8 10 :east
