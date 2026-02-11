; 5-6 Belt Balancer - Fully Connected
; Strategy: Split middle lane (Y=4) to create 6th lane, then balance

; === Input Lanes (5 lanes at Y=0,2,4,6,8) ===
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

; === Stage 1: Connect inputs to first splitter ===
; Lane 0 (Y=0) - straight through
belt-blue 3 0 :east

; Lane 1 (Y=2) - straight through  
belt-blue 3 2 :east

; Lane 2 (Y=4) - goes to splitter
splitter-blue 3 4 :east

; Lane 3 (Y=6) - straight through
belt-blue 3 6 :east

; Lane 4 (Y=8) - straight through
belt-blue 3 8 :east

; === Stage 2: Route from first stage to second splitters ===
; Top output from splitter at (3,4) goes to Y=4
belt-blue 5 4 :east

; Bottom output from splitter at (3,4) creates new lane at Y=10
belt-blue 5 5 :north
belt-blue 5 6 :north
belt-blue 5 7 :north
belt-blue 5 8 :north
belt-blue 5 9 :north
belt-blue 5 10 :east

; Other lanes continue
belt-blue 5 0 :east
belt-blue 5 2 :east
belt-blue 5 6 :east
belt-blue 5 8 :east

; Second stage splitters
splitter-blue 6 2 :east
splitter-blue 6 6 :east

; === Stage 3: Route to third stage ===
belt-blue 8 0 :east
belt-blue 8 2 :east
belt-blue 8 4 :east
belt-blue 8 6 :east
belt-blue 8 8 :east
belt-blue 8 10 :east

; Third stage splitters
splitter-blue 9 0 :east
splitter-blue 9 8 :east

; === Output connections ===
belt-blue 11 0 :east
belt-blue 11 2 :east
belt-blue 11 4 :east
belt-blue 11 6 :east
belt-blue 11 8 :east
belt-blue 11 10 :east

; === Output Lanes (6 lanes) ===
belt-blue 12 0 :east
belt-blue 13 0 :east

belt-blue 12 2 :east
belt-blue 13 2 :east

belt-blue 12 4 :east
belt-blue 13 4 :east

belt-blue 12 6 :east
belt-blue 13 6 :east

belt-blue 12 8 :east
belt-blue 13 8 :east

belt-blue 12 10 :east
belt-blue 13 10 :east
