include include.mk

ifeq ($(TOOLCHAIN), gcc)
CC=$(GCC_PATH)msp430-elf-gcc
CFLAGS=-mmcu=msp430g2211 -Os -I. -ffunction-sections -fdata-sections
LDFLAGS=-mmcu=msp430g2211 -Tmsp430g2211.ld -Wl,-Map=at2xt.map -Wl,--gc-sections
else ifeq ($(TOOLCHAIN), clang)
CC=$(CLANG_PATH)clang --target=msp430 --gcc-toolchain=$(GCC_SYSROOT)
CFLAGS=-Os -I. -ffunction-sections -fdata-sections
LDFLAGS=-mmcu=msp430g2211 -Tmsp430g2211.ld -Wl,-Map=at2xt.map
endif

at2xt.elf: AT2XT_FSM.o AT2XTmain.o
	$(CC) -o at2xt.elf $(LDFLAGS) AT2XT_FSM.o AT2XTmain.o
	msp430-elf-objdump -Cd $@ > at2xt.lst
	msp430-elf-readelf -a $@ > at2xt.sym
	msp430-elf-size at2xt.elf

.c.o:
	$(CC) -c -o $@ $(CFLAGS) $<

cp-build: at2xt.elf
	mkdir -p $(TOOLCHAIN)-build
	cp *.o at2xt.elf at2xt.lst at2xt.sym at2xt.map $(TOOLCHAIN)-build

clean:
	rm -rf *.o at2xt.elf at2xt.lst at2xt.sym at2xt.map
