# AT-to-XT Keyboard Protocol Converter
[![Build Status](https://github.com/cr1901/AT2XT/actions/workflows/ci.yml/badge.svg)](https://github.com/cr1901/AT2XT/actions)
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

## Schematics And PCB
Kicad Schematics are provided the [`board`](board) directory. Gerber Files and
an N/C Drill File are provided in the [`gerber`](gerber) directory. A BOM is
provided in [`at2xtv200_BOM.csv`](at2xtv200_BOM.csv).

You can get boards made using my shared [OSHPark Project](https://oshpark.com/shared_projects/AXSJvSh9),
and parts using my shared (_to the extent I could figure out how to share it_)
[Mouser Project](https://www.mouser.com/ProjectManager/ProjectDetail.aspx?AccessID=fe2f22f0a4).

## Building The Firmware (Rust Source)
As an experiment to test the MSP430 Rust/LLVM backend, the current source has
been rewritten in Rust. All future development will be in Rust. The rewrite
is not _exactly_ semantically equivalent to the C source code; in particular,
in the Rust version, the keyhandling Finite State Machine (FSM) returns
immediately and I/O processing occurs in the main loop. In the C version the
FSM _is_ the main loop, and I/O processing is embedded.

### Minimum Supported Rust Version
In theory, the Minimum Supported Rust Version is "the most recently nightly
available from `rustup` that doesn't break CI". When CI breaks, I notice within
a few days (and I've been meaning to set up a GHA to tell me when it breaks).

In practice, I run `rustup update` every 6 weeks during the Thursday release,
and have not run into problems using a 6 week old `nightly` compiler for AT2XT.

### Prerequisites
This source requires the Rust nightly compiler for the foreseeable future due
to the use of `abi_msp430_interrupt` [feature](https://doc.rust-lang.org/unstable-book/language-features/abi-msp430-interrupt.html).

1. Make sure `git` (for AT2XT), `curl` (for `rustup`), `gcc` (for [proc macros](https://doc.rust-lang.org/reference/procedural-macros.html)),
   and optionally `wget` (for downloading `msp430-elf-gcc`) and `just` (for
   convenience) are installed.

2. Visit the rustup [website](https://www.rustup.rs) and follow the
   instructions to first get `rustc` plus `cargo`.

3. Obtain `msp430-elf-gcc` from TI at the bottom of [this page](http://www.ti.com/tool/msp430-gcc-opensource),
   and make sure the toolchain's bin directory is visible to Rust. Until LLVM
   gets linker support for msp430, binutils is required for the linker.

### Build Command
_As of this writing (1-31-2023), AT2XT can be built using the standard
`cargo build --release`._

#### `.cargo/config` and `rust-toolchain.toml`
Thanks to the [`.cargo/config`](.cargo/config) and [`rust-toolchain.toml`](rust-toolchain.toml)
files, the nightly compiler and compiler source will be downloaded, and
`cargo` will automatically target the built-in `msp430-none-elf` target.

Additionally, MSP430 needs a `libcore` installed that doesn't conflict w/ your host,
and an MSP430 `libcore` is not currently provided as part of the toolchain. The
_unstable_ `cargo` feature `-Zbuild-std=core` allows a developer to build
a `libcore` as part of building your application; `.cargo/config` also takes
care of this step. 

For completeness' sake, the full command to build AT2XT (using `rustup` as a
toolchain manager) is:

```
cargo +nightly build --release -Zbuild-std=core --target=msp430-none-elf
```

#### Justfile
Historically, the build command has changed over time, so I provided a
[Justfile](https://github.com/casey/just) to build AT2XT as well. The Justfile
has mostly been superceded by the above files, and at this point consists of
personal recipes I use for development and CI.

For those interested, run `just --list` for a list of avilable recipes. The
build can be further customized by setting the following variables on the
`just` command line (e.g. `just MODE=release`):

* `MODE`: `release` or `debug`. Defaults to `release`, which _must_ be paired
  with the `--release` option to `cargo`.
* `CFLAGS`: Flags to pass to `cargo`. Defaults to `--release -Zbuild-std=core --target=msp430-none-elf`;
  the `-Zbuild-std=core` and `--target=msp430-none-elf` flags are
  unconditionally required, but `--release` should be unset if doing a `debug`
  build.

## Historical Context And Legacy Source
### Building Older Versions Of The Rust Firmware
When this firmware was first rewritten in Rust in 2017, `nightly` features,
nightly (literally!) code additions, and dependencies were subject to breaking
changes much more frequently than today (1-31-2023). While I was still getting
comfortable with Rust, I did not set up CI to figure out which `nightlies` or
dependency changes broke the build. 

In retrospect, this was a mistake; sometimes the build broke for several
months, such as with this [ThinLTO bug](https://github.com/japaric/xargo/issues/158).
Additionally, I tracked branches rather than refs/tags in [git dependencies](https://github.com/cr1901/msp430-rtfm/commit/f6163b7acaeb135e08af1491daded54057e0d59f), so sometimes even having a working
`nightly` would mean the build would break. In 2023, it's difficult for me to
give ranges of working `nightlies` for old versions of AT2XT.

_That said_, it was my intent when porting the code to Rust that tagged commits
should be able to serve as an example of how to write
bare-metal Rust applications using a variety of different code structures and
varying number of external dependencies (see CHANGELOG.md). It should be
possible with moderate effort to port old tags of AT2XT to modern `nightlies`,
though I have no plans to do this. Additionally, with a proper `nightly` compiler installed, previous
versions should still be able to compile/function with a small to moderate
amount of work (see [Data Layout](#data-layout)). Maybe I'll try a [bisect](https://github.com/rust-lang/cargo-bisect-rustc)
in the future for fun and populate a table of working nightlies for tags :D.

#### RTIC/RTFM
Long ago, AT2XT was implemented using the a proof-of-concept version of the [RTIC framework](https://rtic.rs/1/book/en/)
for MSP430 (back when it was known as the RTFM framework). This support
disappeared in `v3.0.0` of the firmware while removing [API unsoundness](https://blog.japaric.io/brave-new-io/)
elsewhere. I looked into porting doing a more proper port of RTIC in late 2019,
but never followed through. At present (1-31-2023), it is not in my plans to
reintroduce RTIC to AT2XT.

#### Data Layout
The MSP430 data layout changed between the time I started writing this
firmware (June 12, 2017) and as of this writing (July 16, 2017). Recent
nightly compilers will crash with custom provided layout up until commit
c85088c. The data layout in `msp430.json` before this commit should be:
`e-m:e-p:16:16-i32:16-i64:16-f32:16-f64:16-a:8-n8:16-S16`.

MSP430 became a supported target within Rust nightly in July 2017, and the
target "triple" changed from `msp430` to `msp430-none-elf`. I switched to the
internal target as of commit c0dc9b9, but the immediate commit prior c85088c
shows how to generate an equivalent binary with the originally-used custom
target.

### Original AT2XT C Source
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
