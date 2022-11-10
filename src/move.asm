.data
temp:   66
x:      67
y:      68
z:      69

.text
;; Clear A and temp
        temp
        temp
        temp
        temp
;; Two instr clear x because A is clear
        x
        x
;; Load y into A
        y
;; Load -y into A
        temp
;; Skipped
        temp
;; Store y into x and A
        x
;; Clear A and temp
        temp
        temp
        temp
;; Load z into A
        z
;; Load y - z into x
        x
