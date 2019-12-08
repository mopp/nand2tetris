// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/04/Mult.asm

// Multiplies R0 and R1 and stores the result in R2.
// (R0, R1, R2 refer to RAM[0], RAM[1], and RAM[2], respectively.)

    @R2
    M = 0
(LOOP)
    // Jump to END if the R1 is zero.
    @R1
    D = M
    @END
    D;JEQ

    // Add R0 to R2.
    @R0
    D = M
    @R2
    M = M + D

    // Decrement R1
    @R1
    M = M - 1

    @LOOP
    0;JMP

(END)
    @END
    0;JMP
