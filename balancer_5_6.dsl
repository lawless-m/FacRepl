; 5-6 Belt Balancer
; 5 input lanes at Y=0,2,4,6,8
; 6 output lanes at Y=0,2,4,6,8,10

; Input belts (5 lanes)
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

; Stage 1: Split middle lane to create 6th lane
splitter-blue 3 4 :east out-priority:left
belt-blue 5 4 :east
belt-blue 5 5 :north
belt-blue 5 6 :north
belt-blue 5 7 :north
belt-blue 5 8 :north
belt-blue 5 9 :north
belt-blue 5 10 :east

; Continue other lanes
belt-blue 5 0 :east
belt-blue 5 2 :east
belt-blue 5 6 :east
belt-blue 5 8 :east

; Output belts (6 lanes)
belt-blue 6 0 :east
belt-blue 7 0 :east

belt-blue 6 2 :east
belt-blue 7 2 :east

belt-blue 6 4 :east
belt-blue 7 4 :east

belt-blue 6 6 :east
belt-blue 7 6 :east

belt-blue 6 8 :east
belt-blue 7 8 :east

belt-blue 6 10 :east
belt-blue 7 10 :east
