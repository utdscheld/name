mips-linux-gnu-as .artifacts/mips_test.asm -o a.o
mips-linux-gnu-objcopy -O binary --only-section=.text a.o output.o
rm a.o
