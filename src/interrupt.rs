#![feature(asm)]

#[inline(always)]
// Unsafe because it can be used to reenable interrupts anywhere, including regions where it
// is assumed interrupts cannot occur.
pub unsafe fn enable() {
    asm!(r#"nop
            bis #8, sr
            nop"# : : : "sr" : "volatile");
}

#[inline(always)]
pub fn disable() {
    unsafe {
        asm!(r#"bic #8, sr
                nop"# : : : "sr" : "volatile");
    }
}

// First try. Nowhere near as effective as the second version.
// Keeping it to try it out later to see if I understand llvm's inline asm.
/* #[inline(never)]
pub fn is_enabled() -> bool {
    let r: u16;
    unsafe {
        asm!(r#"
            mov sr, r4
            and #8, r4
            tst r4
            jnz enabled
disabled:
            mov #0, $0
            jmp done
enabled:
            mov #1, $0
done:
            "# : "=r"(r) : : "r4", "cc" : "volatile");
    }

    if r == 0 {
        false
    } else {
        true
    }
} */

#[inline(always)]
pub fn is_enabled() -> bool {
    let r: u16;
    unsafe {
        asm!("mov sr, $0" : "=r"(r) : : :);
    }

    if (r & 0x08) == 0 {
        false
    } else {
        true
    }
}

pub struct CriticalSectionToken {
    _0: (),
}


// This is effectively a "rewrite" of:
// https://github.com/japaric/cortex-m/blob/master/src/interrupt.rs#L87
pub fn critical_section<F, R>(f: F) -> R where F: FnOnce(&CriticalSectionToken) -> R {
    let was_enabled : bool = is_enabled();

    disable();

    let r = f(&CriticalSectionToken { _0: () });

    if was_enabled {
        unsafe { enable() }
    }

    r
}
