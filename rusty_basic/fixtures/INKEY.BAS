' clear buffer
PRINT "Clearing input buffer..."
WHILE INKEY$ <> ""
WEND
PRINT "Press any key..."
k$ = INKEY$
WHILE k$ = ""
    k$ = INKEY$
WEND
PRINT "You pressed:", k$
