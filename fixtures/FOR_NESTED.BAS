PRINT "Before the outer loop"
FOR I = 1 TO 10
    PRINT "Before the inner loop", I
    FOR J = 1 TO 10
        PRINT "Inner loop", I, J
    NEXT
    PRINT "After the inner loop", I
NEXT
PRINT "After the outer loop"
