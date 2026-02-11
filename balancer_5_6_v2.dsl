; 5-6 Belt Balancer v2
; Lanes are at Y = 0, 2, 4, 6, 8 (5 inputs)
; Output will be at Y = 0, 2, 4, 6, 8, 10 (6 outputs)

; === Input Section (X=0-2) ===
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

; === Lane routing before splitters (X=3) ===
belt-blue 3 0 :east
belt-blue 3 2 :east
belt-blue 3 4 :east
belt-blue 3 6 :east
belt-blue 3 8 :east

; === First splitter stage (X=4-5) ===
; Split lane at Y=4 to create lanes at Y=4 and Y=5
splitter-blue 4 4 :east

; === Route the split output down to Y=10 (X=6-7) ===
; Upper output continues at Y=4
belt-blue 6 4 :east

; Lower output at Y=5 needs to route down to Y=10
belt-blue 6 5 :north
belt-blue 6 6 :north
belt-blue 6 7 :north
belt-blue 6 8 :north
belt-blue 6 9 :north
belt-blue 6 10 :east

; Other lanes pass through
belt-blue 6 0 :east
belt-blue 6 2 :east
underground-belt-blue 6 6 :east
underground-belt-blue 8 6 :east
belt-blue 6 8 :east

; === Middle section (X=7-8) ===
belt-blue 7 0 :east
belt-blue 7 2 :east
belt-blue 7 4 :east
belt-blue 7 8 :east
belt-blue 7 10 :east

belt-blue 8 0 :east
belt-blue 8 2 :east
belt-blue 8 4 :east
belt-blue 8 8 :east
belt-blue 8 10 :east

; === Balance with splitters (X=9-10) ===
splitter-blue 9 0 :east
splitter-blue 9 8 :east

; === Output routing (X=11-13) ===
belt-blue 11 0 :east
belt-blue 11 1 :east
belt-blue 11 2 :east
belt-blue 11 3 :east
belt-blue 11 4 :east
belt-blue 11 5 :east
belt-blue 11 6 :east
belt-blue 11 7 :east
belt-blue 11 8 :east
belt-blue 11 9 :east
belt-blue 11 10 :east

; === Output Lanes (X=12-14) ===
belt-blue 12 0 :east
belt-blue 13 0 :east
belt-blue 14 0 :east

belt-blue 12 2 :east
belt-blue 13 2 :east
belt-blue 14 2 :east

belt-blue 12 4 :east
belt-blue 13 4 :east
belt-blue 14 4 :east

belt-blue 12 6 :east
belt-blue 13 6 :east
belt-blue 14 6 :east

belt-blue 12 8 :east
belt-blue 13 8 :east
belt-blue 14 8 :east

belt-blue 12 10 :east
belt-blue 13 10 :east
belt-blue 14 10 :east
