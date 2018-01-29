# AT-to-XT Keyboard Protocol Converter
[![Build Status](https://travis-ci.org/cr1901/AT2XT.svg?branch=master)](https://travis-ci.org/cr1901/AT2XT)

This repository provides the source, schematics, and Gerber files that converts
the AT-keyboard protocol to the XT keyboard protocol. As XT keyboards are
expensive (seriously, type in "PC XT keyboard" or "PC 5150 keyboard" in Ebay),
this provides a cheaper alternative for someone willing to wait for PCB and
parts. This circuit supports 101-key extended keyboards using the XT protocol,
but older pre-386 systems may not know how to handle extended keys. The
extended keycodes are based on a document from Microsoft that includes XT
keycodes for compatibility.

## Rust Source
As an experiment to test the MSP430 Rust/LLVM backend, the current source has
been rewritten in Rust. All future development will be in Rust. The rewrite
is not _exactly_ semantically equivalent to the C source code; in particular,
in the Rust version, the keyhandling Finite State Machine (FSM) returns
immediately and I/O processing occurs in the main loop. In the C version the
FSM _is_ the main loop, and I/O processing is embedded.

### Prerequisites
This source requires the Rust nightly compiler for the foreseeable future.
To obtain the nightly compiler and relevant dependencies:

1. Visit the rustup [website](www.rustup.rs) and follow the instructions to
first get a stable compiler. _I have only tested the GNU ABI version of Rust
on Windows_, but choose which version makes sense for you.

2. `rustup` should now be on your path. Obtain the nightly compiler with:
`rustup install nightly`. As of before July 16, 2017, MSP430 support is
enabled in Rust nightly. Switch to the nightly compiler by running:
`rustup default nightly`.

3. MSP430 needs a `libcore` installed that doesn't conflict w/ your host. The
`xargo` program allows a developer to maintain multiple `libcores` for
multiple archs simultaneously: `cargo install xargo`.

4. Obtain `msp430-elf-gcc` from TI at the bottom of
[this page](http://www.ti.com/tool/msp430-gcc-opensource), and make sure the
toolchain's bin directory is visible to Rust. As I understand it, the GCC
toolchain is required because Rust is hardcoded to call the compiler driver
to assemble if LLVM is not emitting object files itself; LLVM doesn't emit
objects for MSP430 as of this writing. Furthermore, binutils will
be required for the foreseeable future for the linker.

### Building
The current command to build is:
`xargo build --release --target=msp430-none-elf`. This command has changed
over time, so I provide a Makefile as well: `make` to build, and `make prog`
to program using a Launchpad, `mspdebug`, and Spy-Bi-Wire connections.

### Tags/Comparing Versions
Tags to previous versions are included to compare the overhead of adding
various abstractions and making the source code look more like an idiomatic
hosted Rust program. Some considerations when comparing versions:

* The MSP430 data layout changed between the time I started writing this
firmware (June 12, 2017) and as of this writing (July 16, 2017). Recent
nightly compilers will crash with custom provided layout up until commit
c85088c. The data layout in `msp430.json` before this commit should be:
`e-m:e-p:16:16-i32:16-i64:16-f32:16-f64:16-a:8-n8:16-S16`.

* MSP430 became a supported target within Rust nightly in July 2017, and the
target "triple" changed from `msp430` to `msp430-none-elf`. I switched to the
internal target as of commit c0dc9b9, but the immediate commit prior c85088c
shows how to generate an equivalent binary with the originally-used custom
target.

## Legacy Source
For comparison purposes, I have kept the old C-based source code as well under
the `legacy-src` directory.

Currently, it is up to the user to set up their toolchain to compile the files
for programming an MSP430G2211 or compatible 14-pin DIP MSP430. I recommend the
former, if only because MSP430 is already overkill for this project and G2211
is a low-end model :P. However, I . When the C source was written, TI expected
users to compile with Code Composer Studio (CCS). Today, a Makefile generic to
all OSes and requiring only a command line should work, and will be available
soon. Compile using -O2 or better.

The C source code itself should be easy to port to other microcontrollers,
except for the use of a `__delay_cycles()` intrinsic. I had no choice here, as
using the timer for a software delay can lock the keyboard FSM to a single
state.

## Schematics
Schematics are provided in DIPTrace ASCII format. PCB is provided using Gerber
Files and an N/C Drill File.

It is my intention sometime soon to redo the schematic using KiCAD.
