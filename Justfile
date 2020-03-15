MODE := "release"
XFLAGS := "--release"
TARGET := "target/msp430-none-elf/" + MODE + "/at2xt"
CLIPPY_LINTS := "-W clippy::if_not_else -W clippy::match_same_arms -W clippy::as_conversions"

# Build AT2XT using the timer feature.
timer:
    xargo build {{XFLAGS}} --target=msp430-none-elf
    msp430-elf-objdump -Cd {{TARGET}} > {{TARGET}}.lst
    msp430-elf-readelf -s --wide {{TARGET}} > {{TARGET}}.sym
    msp430-elf-size {{TARGET}}

# Build AT2XT using the timer feature and extra artifacts.
timer-extra:
    xargo rustc {{XFLAGS}} --target=msp430-none-elf -- --emit=obj={{TARGET}}.o
    msp430-elf-objdump -Cd {{TARGET}} > {{TARGET}}.lst
    msp430-elf-readelf -s --wide {{TARGET}} > {{TARGET}}.sym
    msp430-elf-objdump -Cd {{TARGET}}.o > {{TARGET}}.o.lst
    msp430-elf-readelf -r --wide {{TARGET}}.o > {{TARGET}}.reloc
    msp430-elf-size {{TARGET}}

# Run clippy on AT2XT.
clippy:
  xargo clippy --target=msp430-none-elf -- {{CLIPPY_LINTS}}

# Run clippy on AT2XT- pedantic mode (many lints won't apply).
clippy-pedantic:
  xargo clippy --target=msp430-none-elf -- -W clippy::pedantic -W clippy::as_conversions

# Fix warnings in AT2XT.
fix:
  xargo fix --target=msp430-none-elf

# Fix warnings and attempt to apply clippy suggestions (nightly only).
fix-clippy:
  xargo fix -Z unstable-options --target=msp430-none-elf --clippy

fmt:
  xargo fmt

# Remove AT2XT and dependencies.
clean:
    xargo clean

# Upload firmware to AT2XT board using MSP-EXP430G2.
prog:
    mspdebug rf2500 "prog {{TARGET}}"
