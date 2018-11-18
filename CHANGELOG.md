# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/).
This project does _not_ strictly adhere to semantic versioning. Major version
changes are reserved for code restructuring changes. Minor version changes
are reserved for collections of changes that justify incrementing the version
number. The patch level was only used once and then abandoned,
as optimizations, bugs, and code cleanup are pooled into minor releases.

# AT2XT Firmware
## [Unreleased]

## [2.2.0]
The following is a "cleanup" release. Nightly Rust did not work properly on
msp430 from approx
[July 7, 2018](https://travis-ci.org/cr1901/AT2XT/builds/401135317) to
[Oct 31, 2018](https://travis-ci.org/cr1901/AT2XT/builds/448735034).

In this  time frame, `rustc` learned to shave 40 or so bytes off the final
binary in `-Os`! Contrast to
[July 6, 2018](https://travis-ci.org/cr1901/AT2XT/builds/400722102) build.

Additional changes, which are found in the
[commit history](https://github.com/cr1901/AT2XT/compare/v2.1.0...v2.2.0),
include keeping AT2XT up to date with the enabled/disabled features of the
nightly compiler _at the time_. Some of these changes, such as the
`proc_macro_gen` feature gate, appeared _and_ disappeared between [2.1.0]
and [2.2.0].

### Added
- Travis CI [support](https://travis-ci.org/cr1901/AT2XT).
- Various README.md improvements.
- Do not depend on [msp430-rt]'s `panic_implementation`; it is set for
removal (and `panic_implementation` does not exist in recent nightlies).
Instead, since `panic_handler` is stable, we provide our own implementation.
- Support version `6.4.0` of `msp430-elf-gcc`.

### Fixed
- Fix multiple warnings, particularly of the following types:
  - Unnecessary mutability
  - Unnecessary parentheses

### Changed
- Use [msp430-atomic] crate available on
[crates.io](https://crates.io/crates/msp430-atomic).
- Output `msp430-elf-readelf` symbols in wide format for better debugging.
- [msp430-rtfm] syntax/semantics changed in a manner that significantly
increased code size. Although the
bug has since been
[fixed](https://github.com/japaric/cortex-m-rtfm/issues/41), there has not
been much progress in checking that the fix works on msp430. Instead for now,
we pin AT2XT to a [known working]
(https://github.com/cr1901/msp430-rtfm/tree/at2xt-pin) version of
[msp430-rtfm].
- Remove unsafety in `keymap` module, now that `rustc` will correctly remove
bounds check and/or not bring in panic strings (I haven't checked).
- LLVM misoptimizes calls to `Pin::bitmask()` in some cases; they aren't
treated as a function returning a compile-time constant. Provide a
workaround using `const`.

### Removed
- All uses of `Option::unwrap()`; as of before Jan 28, 2018, they add strings
to the binary which AT2XT can't display.

## [2.1.0]
### Added
- Firmware can now be built using `TIMERA` for delay loops instead of a
software delay.
- Changelog added based on tag messages (for commits after/including [1.0.0]).

### Fixed
- Timer functionality was pulled in from a private branch between this release
and the previous, but was non-functional; now fixed. Fixes include:
  - Increase LED toggle wait so firmware doesn't send byte while keyboard is
  still sending response code.
  - Fix busy-wait loop condition for timer.
  - Ensure timer interrupt is enabled.
  - Fix busy-wait tick count for non-timer firmware.

### Changed
- The [msp430g2211] crate was published,
so we use the [crates.io](https://crates.io/crates/msp430g2211) version now.
- Change [bit_reverse] algorithm to `BitwiseReverse` to save approx 18-20 bytes.
- Use the newly-created [msp430_atomic] repository for wait-free `AtomicBool`.
Removes a number of critical sections, thus saving approx 30 bytes.
- Between [2.0.0] and [2.1.0], an [r0](https://github.com/japaric/r0)
[optimization](https://github.com/japaric/r0/blob/master/CHANGELOG.md#v022---2017-07-21)
was added to the published crate, which is then pulled in by [msp430-rt],
[msp430], and [msp430g2211]. This optimization saves between 20 to 40 bytes
(I've seen up to 44 on commit 8c311c6).

### Removed
- Clean up unused dependencies in Cargo.toml file.
- Removed redundant register writes to clear interrupt flag (done by
hardware).

## [2.0.0]
### Added
- Additional README.md changes, installation instruction improvements.
- Code has been restructured to use the [RTFM framework][msp430-rtfm].

### Changed
- RTFM subsumes all responsibilities of [msp430-rt], and provides a macro which
allows hardware to be accessed in interrupts without a `CriticalSection` token.
- Keyboard driver no longer requires critical sections, but now requires an
`rtfm` resource. This resource can either be acquired safely (`rtfm::atomic`,
or within an ISR) or opting into unsafety.

## [1.3.0]
### Added
- Finally updated the long-out-of-date usage/installation instructions.
- Use the [msp430g2211] crate to provide a microcontroller API.

### Changed
- Instead of using symbols representing register addresses provided by the
linker, use [msp430g2211]'s API functions instead. API also prevents leakage
of `unsafe` I/O writes.
- Use [msp430g2211]'s API to provide the interrupt table, which wraps around
[msp430-rt]'s method of populating the table.

### Removed
- Linker script providing I/O addresses no longer necessary, and thus removed.

## [1.2.0]
### Added
- Use the [msp430-rt] crate to provide startup code and interrupt handler API.

### Changed
- Rust nightly provides `msp430-none-elf` target by this point, so use that.
The last commit before the target swap generated an equivalent binary (at
the time) for reproducibility.
- Interrupt handlers are provided using [msp430-rt] API instead of using
`link_section` attribute.
- `memcpy` and `memset` are now provided by [compiler-builtins], a `libgcc`/
`libcompiler_rt` helper crate.

### Removed
- [msp430-rt] provides startup code, including memory initialization via [r0].
No need to call [r0] directly.
  - `panic_fmt` code removed for same reason.
- TI linker script proper no longer necessary, as [msp430-rt] provides
a linker script with everything except the memory map (`memory.x`)
- `libc` no longer linked against directly, thus removed as link library.
[compiler-builtins] still will emit `memcpy`/`memset`, however.

## [1.1.0]
Most space savings were checked a few weeks after [1.0.0]'s release with a more
recent nightly compiler. Some things have changed since then:
- In general, `-O3` no longer compiles.
- Commit b8d4779 now saves space.
I am currently uncertain what changed. This is a footnote to remind me to check
at some point using nightly 07-15-2017 and (approx) 06-12-2017.

### Added
- Use the [r0] crate to provide `.bss` and `.data` initialization. At the time,
this increased binary size by a fair amount.
  - A number of extra linker-script-provided symbols (`__bssstart`,
    `__bssend`) now required.

### Fixed
- `DEVICE_ACK` busy-wait wasn't properly translated to `Cell` encapsulation,
fixed by release.
- [r0] was brought in because the addition of `Mutex` and `Cell` required
global data, and custom init code was insufficient to initialize either of
these. 30 bytes (`-O3`) added for `.data` init, 28 bytes (`-O3`) for .bss init.

### Changed
- The `FSM` in `keyfsm` no longer returns a `Result<State, State>`, but rather
a `State`. _This was a significant savings of 70 bytes_.
- Wrap the globals `HOST_MODE`, `DEVICE_ACK`, `KEY_OUT`, `KEY_IN` in `Cell`s
and use interior mutability to safely mutate globals. Guarantted to be safe
thanks to `free` sections, and size cost is minimal, nearly zero-cost.
  - Due to this, the keyboard driver functions no longer require
  `CriticalSection`s. It's assumed user will grab a token before using these
  functions, unsure safety in some other way, or opt into unsafety
  (increases function usability).
- `IN_BUFFER`, on the other hand, requires a `RefCell` b/c `Array`doesn't
implement `Clone`. Due to  runtime checks (which _I_ know apriori can't fail,
but the compiler doesn't), cost of safety is nonzero.
- Some savings (36 bytes- mutex booleans commit) came from combining
`free` sections. _I don't remember this savings originally._
- The combination of using [r0] startup code, making `IN_BUFFER` safe, means
the code no longer fits into msp430g2211 in `-O3`, so enable `-Os` without
issue.
- `delay` is now considered safe after talking it over in `#rust-embedded`.

### Removed
- Manual initialization of global `.data`/`.bss` is no longer used; use
[r0] instead, at the cost of space. _This enables fair comparisons to
C code, which will normally bring in startup code even for small micros._
- The `util.rs` source file was removed per
[recommendation](https://twitter.com/withoutboats/status/877761222073925633).
- `keymap` module was incorporated as private module into the FSM source file.

## [1.0.1]
### Fixed
- Up to this point, little effort was made to fix warnings. Code was modified
to remove warnings, mainly of the following types (and their fixes):
  - Unused vars (Remove, or `let _ = my_unused_var;`)
  - Unused imports (Remove)
  - Unused features (Remove)
  - Dead code (`#[allow(dead_code)]`)
  - Snake case for structs (e.g. `state` => `State`)
  - Redundant and unnecessary `unsafe` blocks (Remove)
  - Additional changes:
    - `allow(private_no_mangle_fns)]` for `panic_fmt` dance
    - Function name fixes (no capital letters)

### Changed
- Now that the firmware is known to work, I start importing already-existing
crates to automate work I did manually.
  - [bare_metal](https://github.com/japaric/bare-metal) crate provides a
  generic `CriticalSection`, so replace `CriticalSectionToken` =>
  `CriticalSection`.
  - [msp430] crate provides interrupt helpers, as well as `critical_section`,
  so replace `critical_section` => `free`.
  - Neither of the above changes add to code size- truly zero-cost.

### Removed
- Thanks to above changes/crates, `interrupt.rs` no longer provides any
functionality, and thus removed.
- Commented out code cleaned up (mostly no-longer relevant code).

## [1.0.0]
### Added
- Pause/Break handling logic implemented, which must be handled specially.
- First fully functional firmware. This initial release depends on very few
external packages and provides an example of how to write a Rust firmware
taking care of everything manually (aside from a few small helper crates).

## [0.9.5]
### Added
- LED toggling logic added, which requires a special state in the FSM to
send a "light LED command" to the keyboard upon receipt of a break code, before
sending the equivalent break code to the XT host (XT keyboard lights were
not standardized like this).

## [0.9.0]
### Added
- Handle `E0` keycodes in the FSM, which are special keycodes whose first byte
are passed to the XT unchanged. These include _right_ CTRL, SHIFT, ALT, and
arrow keys, for example. Pause must be handled specially, and is not
implemented yet.

## [0.8.0]
### Added
- Actually clear `KeycodeBuffer` when FSM requests it.

### Fixed
- All possible `panic` locations up to this point had been optimized out by
LLVM due to optimizations involving infinite loops. Fix by introducing an `asm`
barrier.
- `panic` implementation exposed at least two more bugs. It was a happy
  accident that `panic` omission generated "working" code:
  - `.bss` initialization needed to be done for initially-zero variables due
  to lack of runtime support.
  - FSM was missing proper state transition back to `state::NotInKey` after
  break code was sent to XT.

## [0.7.0]
### Added
- Implement most of the FSM keycode conversions. Specifically, any key
consisting of a single make code and 2-byte break code on the AT side is now
translated properly to the XT host.
  - This gets quite a bit of functionality, and is likely usable on its own (if
  unpleasant).

### Fixed
- The FSM logic generated multiplication. Since this MSP430 variant does _not_
  have a hardware multiplier, we must use the gcc-provided software
  multiplication routines.

### Changed
- My patch to [bit_reverse] made it to a published version, so use upstream.

## [0.5.0]
This release should have been split into smaller releases. All bugs _and fixes_
introduced _between_ 0.1.0 and [0.5.0] are documented here.

### Added
- Keyboard driver file added (`driver.rs`), which is a truly zero-cost
abstraction for reading and writing the relevant microcontroller pins.
  - Up until after version [1.0.1], `critical_section` was hand-rolled
  and not provided by a crate. _This was mainly an experiment to demonstrate
  how to write Rust firmware without (many) external dependencies._
- `critical_section` implementation added to safely access I/O and mark
`unsafe` I/O writes as truly safe (no data races possible if interrupts
disabled).
- Keyboard buffer abstraction (`keybuffer.rs`) file added for convenience
and abstraction. The file relies on Rust's inlining for buffer operations
to be truly zero-cost.
- Use [volatile_register](https://crates.io/crates/volatile-register) because at the time I didn't understand
[interior mutability](https://doc.rust-lang.org/core/cell/), and Rust's
volatile semantics.
  - Interior mutability allows writing I/O pins without a mutable reference.
  Volatile operations prevent reordering or optimizing I/O R/W away. Volatile
  operations are not at the type level as they are in C.
- Introduce the [bit_reverse] crate to take advantage of code reuse when
  shifting in keyboard data.
- Sending and receiving to keyboard routines, as well as sending to XT host
  are functional.

### Fixed
- Keyboard buffer implementation had a number of bugs
- [bit_reverse] repository did not have support for `usize=16` Rust targets
when I started
- msp430 interrupt enable/disable did not handle pipeline bugs (requires
`nop`s to work properly)

### Removed
- Keyboard driver originally embedded `critical_section`s. This was removed in
favor of passing in `CriticalSectionToken`s.


# AT2XT PCB
The AT2XT PCB follows a different versioning scheme. _Unless otherwise noted,
any version of the AT2XT firmware will function on any version of the board_.
New versions of the PCB do not constitute a version increase in the repository.
Links to versions are the commit where the new version was introduced (as
opposed to a comparison).

# [PCB-1.01]- Circa 2013-09
### Changed
- Second revision of PCBs, using less-hassle through-hole components.
These have the voltage regulator pins swapped on the board, and
should not be manufactured. A new design will follow shortly.

# 0.90- Circa 2013-07
### Added
- Original batch of PCBs using surface mount components. Design is lost to time.


[bit_reverse]: https://github.com/EugeneGonzalez/bit_reverse
[r0]: https://github.com/japaric/r0
[msp430]: https://github.com/rust-embedded/msp430
[msp430-rt]: https://github.com/rust-embedded/msp430-rt
[msp430g2211]: https://github.com/cr1901/msp430g2211
[msp430-rtfm]: https://github.com/japaric/msp430-rtfm
[msp430_atomic]: https://github.com/pftbest/msp430-atomic
[compiler-builtins]: https://github.com/rust-lang-nursery/compiler-builtins

[Unreleased]: https://github.com/cr1901/AT2XT/compare/v2.2.0...HEAD
[2.2.0]: https://github.com/cr1901/AT2XT/compare/v2.1.0...v2.2.0
[2.1.0]: https://github.com/cr1901/AT2XT/compare/v2.0.0...v2.1.0
[2.0.0]: https://github.com/cr1901/AT2XT/compare/v1.3.0...v2.0.0
[1.3.0]: https://github.com/cr1901/AT2XT/compare/v1.2.0...v1.3.0
[1.2.0]: https://github.com/cr1901/AT2XT/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/cr1901/AT2XT/compare/v1.0.1...v1.1.0
[1.0.1]: https://github.com/cr1901/AT2XT/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/cr1901/AT2XT/compare/v0.9.5...v1.0.0
[0.9.5]: https://github.com/cr1901/AT2XT/compare/v0.9.0...v0.9.5
[0.9.0]: https://github.com/cr1901/AT2XT/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/cr1901/AT2XT/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/cr1901/AT2XT/compare/v0.5.0...v0.7.0
[0.5.0]: https://github.com/cr1901/AT2XT/compare/v0.1.0...v0.5.0

[PCB-1.01]: https://github.com/cr1901/AT2XT/commit/7fb0578e1a45b8f2f998aadfc368d10b6378ccda
