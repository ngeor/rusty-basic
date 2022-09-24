CLS
LOCATE , , 0 ' cursor off
FOR N = 1 TO 12
    COLOR 15, N
    FOR R = 1 TO 10
        IF N MOD 2 = 0 THEN
            ROW = 11 - R
        ELSE
            ROW = R
        END IF
        COL = R + (N - 1) * 10
        LOCATE ROW, COL
        PRINT "*";
    NEXT
NEXT
COLOR 7, 0
LOCATE 11, , 1 ' cursor on
