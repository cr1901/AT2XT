# AT-to-XT Keyboard Protocol Converter
[![Build Status](https://travis-ci.org/cr1901/AT2XT.svg?branch=master)](https://travis-ci.org/cr1901/AT2XT)
[![Latest version](https://img.shields.io/github/tag/cr1901/AT2XT.svg)](https://GitHub.com/cr1901/AT2XT/tags/)
[![GitHub license](https://img.shields.io/github/license/cr1901/AT2XT.svg)](https://github.com/cr1901/AT2XT/blob/master/LICENSE.md)
[![Last commit](https://img.shields.io/github/last-commit/cr1901/AT2XT.svg)](https://GitHub.com/cr1901/AT2XT/commit/)
[![](https://tokei.rs/b1/github/cr1901/AT2XT?category=code)](https://github.com/cr1901/AT2XT)
[![Awesome Badges](https://img.shields.io/badge/badges-awesome-green.svg)](https://github.com/Naereen/badges)
[![Contact Me](https://img.shields.io/twitter/follow/cr1901.svg?label=Contact%20Me&&style=social)](https://twitter.com/cr1901)

This repository provides the source, schematics, and Gerber files that converts
the AT-keyboard protocol to the XT keyboard protocol. As XT keyboards are
expensive (seriously, type in "PC XT keyboard" or "PC 5150 keyboard" in Ebay),
this provides a cheaper alternative for someone willing to wait for PCB and
parts. This circuit supports 101-key extended keyboards using the XT protocol,
but older pre-386 systems may not know how to handle extended keys. The
extended keycodes are based on a [document](https://download.microsoft.com/download/1/6/1/161ba512-40e2-4cc9-843a-923143f3456c/scancode.doc)
from Microsoft that includes XT keycodes for compatibility.

## Rust Source
As an experiment to test the MSP430 Rust/LLVM backend, the current source has
been rewritten in Rust. All future development will be in Rust. The rewrite
is not _exactly_ semantically equivalent to the C source code; in particular,
in the Rust version, the keyhandling Finite State Machine (FSM) returns
immediately and I/O processing occurs in the main loop. In the C version the
FSM _is_ the main loop, and I/O processing is embedded.

### Prerequisites
This source requires the Rust nightly compiler for the foreseeable future due
to the use of `abi_msp430_interrupt` [feature](https://doc.rust-lang.org/unstable-book/language-features/abi-msp430-interrupt.html).
To obtain the nightly compiler and relevant dependencies:

1. Visit the rustup [website](www.rustup.rs) and follow the instructions to
first get a stable compiler. _I have only tested the GNU ABI version of Rust
on Windows_, but choose which version makes sense for you.

2. `rustup` should now be on your path. Obtain the nightly compiler with:
`rustup install nightly`. As of before July 16, 2017, MSP430 support is
enabled in Rust nightly. Switch to the nightly compiler by running:
`rustup default nightly`. Alternatively, if you only want to use the nightly
compiler for this project, you can run `rustup override set --path /path/to/AT2XT/root nightly`.
This tells `rustup` to use the nightly compiler by default when compiling the
AT2XT crate.

3. Obtain `msp430-elf-gcc` from TI at the bottom of
[this page](http://www.ti.com/tool/msp430-gcc-opensource), and make sure the
toolchain's bin directory is visible to Rust. Until LLVM gets linker support
for msp430, binutils is required for the linker.

### Building
MSP430 needs a `libcore` installed that doesn't conflict w/ your host. The
_unstable_ `cargo` feature `-Zbuild-std=core` allows a developer to maintain
multiple `libcores` for multiple archs simultaneously.

The current command to build is:
`cargo build --release -Zbuild-std=core --target=msp430-none-elf`. This command
has changed over time, so I provide a [Justfile](https://github.com/casey/just)
as well. Run `just --list` for a list of avilable recipes. The build can be
further customized by setting the following variables on the `just` command
line (e.g. `just MODE=release`):

* `MODE`: `release` or `debug`. Defaults to `release`, which _must_ be paired
  with the `--release` option to `cargo`.
* `CFLAGS`: Flags to pass to `cargo`. Defaults to `--release -Zbuild-std=core --target=msp430-none-elf`;
  the `-Zbuild-std=core` and `--target=msp430-none-elf` flags are
  unconditionally required, but `--release` should be unset if doing a `debug`
  build.

### Dependencies Caveats
#### Compiler/Dependency Mismatches
As Rust the language evolves, certain features in `nightly` may be enabled
which break old commits that once compiled. For example, a
[ThinLTO bug](https://github.com/japaric/xargo/issues/158) in `rustc` ensured
compilation for targets using an external assembler- including MSP430-
was broken in `nightly` from August until January!

I can give approximate ranges for which `nightlies` work with which range of
commits, but because the functionality of the `nightly` I make no guarantees
that previous commits will compile; using the correct compiler
[may not solve](https://github.com/cr1901/msp430-rtfm/commit/f6163b7acaeb135e08af1491daded54057e0d59f)
all dependency version mismatches in libraries whose public APIs are in flux

_That said_, it was my intent when porting the code to Rust that tagged
commits should be able to serve as an example of how to write bare-metal Rust
applications using a variety of different code structures and varying number of
external dependencies (see CHANGELOG.md). Previous versions should still be
able to compile/function with a small to moderate amount of work
(see "data layout" in Tags/Comparing Versions for an example).

#### RTFM
AT2XT at present does not use [RTFM framework](http://www.rtfm-lang.org).
This will likely be a version `4.x` milestone. `3.x` is dedicated to testing
the `take`-based `Peripherals` [API](https://blog.japaric.io/brave-new-io/).

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
users to compile with Code Composer Studio (CCS). Today, I provide a generic
Makefile instead. To compile, invoke `make`; there is only a single target,
`at2xt.elf`. This requires the `msp430-elf-gcc` toolchain from the
[Prerequisites](#prerequisites) section.

The C source code itself should be easy to port to other microcontrollers,
except for the use of a `__delay_cycles()` intrinsic. I had no choice here, as
using the timer for a software delay can lock the keyboard FSM to a single
state.

### XTATKEY.ASM
One of the original XT to AT keyboard converters was written by [Chuck Guzis](http://www.vcfed.org/forum/member.php?3458-Chuck(G))
in 2009. By my own admission, PIC is a better fit for this project due to 5V
compatibility and fewer parts required. However, I wrote my version in 2013
because of my familiarity with msp430, easy [5V interfacing](http://www.ti.com/lit/an/slaa148a/slaa148a.pdf)
and easy access to parts and an msp430 programmer. In contrast, PIC programmers
at the time were expensive ([less true](https://www.microchip.com/developmenttools/ProductDetails/PartNO/PG164100)
today), and I didn't feel like buying or making one.

However, since I've used Chuck(G)'s version as inspiration when I got stuck,
I have provided the [source](http://www.vcfed.org/forum/showthread.php?15907-AT-to-XT-Keyboard-Converter&p=106297#post106297)
and [schematics](http://www.vcfed.org/forum/showthread.php?15907-AT-to-XT-Keyboard-Converter&p=106341#post106341)
to his version- _with permission_- under the `legacy-src/XTATKEY` directory.
See linked forum posts for details.

## Schematics
Schematics are provided in DIPTrace ASCII format. PCB is provided using Gerber
Files and an N/C Drill File.

It is my intention sometime soon to redo the schematic using KiCAD.
