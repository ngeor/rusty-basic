CLS
PRINT "Only background color"
FOR I = 0 TO 15
    COLOR , I
    PRINT "background ", I
NEXT
' reset to black
COLOR , 0
PRINT "Only foreground color"
FOR I = 0 TO 15
    COLOR I
    PRINT "foreground ", I
NEXT
' reset to light gray
COLOR 7
PRINT "Both foreground and background"
FOR F = 0 TO 15
    FOR B = 0 TO 15
        COLOR F, B
        PRINT "Foreground", F, "background", B
    NEXT
NEXT
' reset
COLOR 7, 0
