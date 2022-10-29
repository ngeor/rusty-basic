DECLARE FUNCTION Fib% (N%)
PRINT "Enter the number of fibonacci to calculate"
INPUT N%
FOR I% = 0 TO N%
    PRINT "Fibonacci of ", I%, " is ", Fib%(I%)
NEXT

FUNCTION Fib% (N%)
    IF N% <= 1 THEN
        Fib% = N%
    ELSE
        Fib% = Fib%(N% - 1) + Fib%(N% - 2)
    END IF
END FUNCTION
