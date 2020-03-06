CFLAGS=-Os -I. -flto -ffunction-sections -fdata-sections
LDFLAGS=-mmcu=msp430g2211 -Tmsp430g2211.ld -Wl,-Map=at2xt.map -Wl,--gc-sections


at2xt.elf: AT2XT_FSM.o AT2XTmain.o
	msp430-elf-gcc -o at2xt.elf $(LDFLAGS) AT2XT_FSM.o AT2XTmain.o
	msp430-elf-objdump -Cd $@ > at2xt.lst
	msp430-elf-readelf -s $@ > at2xt.sym
	msp430-elf-size at2xt.elf

.c.o:
	msp430-elf-gcc -c -o $@ $(CFLAGS) $<

clean:
	rm -rf *.o at2xt.elf at2xt.lst at2xt.sym at2xt.map
