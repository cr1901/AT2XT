extern crate bit_reverse;

use self::bit_reverse::ParallelReverse;

pub fn reverse_bits(sin : u8) -> u8 {
    sin.swap_bits()
}

pub fn compute_parity(mut sout : u8) -> bool {
    let mut num_ones : u8 = 0;

    for _ in 0..8 {
        num_ones = num_ones + (sout & 0x01);
        sout = sout << 1;
    }

    (num_ones % 2 == 0) // If even, we need an extra one- odd parity.
}
