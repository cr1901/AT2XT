MODE := "release"
XFLAGS := "--release"
TARGET := "target/msp430-none-elf/" + MODE + "/at2xt"

# Build AT2XT using software delay loops.
default:
    xargo build {{XFLAGS}} --target=msp430-none-elf
    msp430-elf-objdump -Cd {{TARGET}} > {{TARGET}}.lst
    msp430-elf-readelf -s --wide {{TARGET}} > {{TARGET}}.sym
    msp430-elf-size {{TARGET}}

# Build AT2XT using the timer feature.
timer:
    xargo build {{XFLAGS}} --target=msp430-none-elf --features use-timer
    msp430-elf-objdump -Cd {{TARGET}} > {{TARGET}}.lst
    msp430-elf-readelf -s --wide {{TARGET}} > {{TARGET}}.sym
    msp430-elf-size {{TARGET}}

# Remove AT2XT and dependencies.
clean:
    xargo clean

# Upload firmware to AT2XT board using MSP-EXP430G2.
prog:
    mspdebug rf2500 "prog {{TARGET}}"
