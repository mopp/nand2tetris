// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/04/Fill.asm

// Runs an infinite loop that listens to the keyboard input.
// When a key is pressed (any key), the program blackens the screen,
// i.e. writes "black" in every pixel;
// the screen should remain fully black as long as the key is pressed.
// When no key is pressed, the program clears the screen, i.e. writes
// "white" in every pixel;
// the screen should remain fully clear as long as no key is pressed.
(LOOP)
    @KBD
    D = M
    @WHITE
    D;JEQ

(BLACK)
    // Use black color for filling the screen.
    D = 0
    D = !D
    @color
    M = D
    @FILL_BEGIN
    0;JMP

(WHITE)
    // Use white color for filling the screen.
    @color
    M = 0

(FILL_BEGIN)
    // 512 * 256 = (16 * 32) * 256 = 16 * (32 * 256) = 16 * 8192
    // i = 8192
    // while (1) {
    //   SCREEN[i] = 0xffff;
    //   i = i - 1;
    //   if (i == 0) {
    //     break;
    //   }
    // }
    @SCREEN
    D = A
    @8191
    D = D + A
    @R0
    M = D
(FILL_LOOP)
    // screen[i] = 0xFFFF
    @color
    D = M
    @R0
    A = M
    M = D

    @R0
    M = M - 1
    D = M
    @FILL_LOOP
    D;JNE

    @LOOP
    0;JMP
