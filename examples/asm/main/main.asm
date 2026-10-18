; Prints a greeting through the CP/M BDOS.

BDOS    EQU     0005H           ; the BDOS entry point
PRINT   EQU     9               ; BDOS function: print a string up to $

        ORG     0100H           ; CP/M loads a .COM file here

        MVI     C,PRINT
        LXI     D,MSG
        CALL    BDOS
        RET                     ; back to the CCP

MSG:    DB      'HELLO FROM ASM',13,10,'$'

        END
