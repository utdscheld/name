# Hello World
.text

.include "SysCalls.asm"

.eqv FIFTY 50
.macro mtest (%a, %b, %c)
   add %a, %b, %c
   sub %c, %b, %a
   lb %a, FIFTY(%c)
.end_macro

main: # Hello Test
   add $t0,$t2,$t3
   sub $t4, $t5, $t6
   sll $s0, $s0, 5
   srl $s5, $s7, 10
   xor $t7, $t8, $t9
   mtest ($t0, $t1, $t2)
   lui $t0, FIFTY
   ori $t0, $t1, FIFTY
   ori $t0, $t0, 0x50
   ori $t0, $t0, 050
   ori $t0, $t0, 0b1010
   lb  $t0, 0x50($t1)
   lb  $t0, 50($t1)
   lb  $t0, ($t1)
   beq $s0, $s0, test
   or $t0, $s5, $fp
   or $s3, $s5, $sp
   slt $t9, $zero, $zero
   jr $ra
   slti $t6, $zero, 0x789
   addi $ra, $ra, 8
   la $a0, main
test:
   j   main
   jal test
