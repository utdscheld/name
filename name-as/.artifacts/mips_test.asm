main:   
   add $t0,$t2,$t3
   sub $t4, $t5, $t6
   sll $s0, $s0, 5
   srl $s5, $s7, 10
   xor $t7, $t8, $t9
   lui $t0, 50
   ori $t0, $t1, 50
   ori $t0, $t0, 0x50
   ori $t0, $t0, 050
   ori $t0, $t0, 0b1010
   lb  $t0, 0x50($t1)
   beq $s0, $s0, test
test:
   j   test
   jal test
