use num_traits::PrimInt;
use std::mem::size_of;

/*
// Update a value in a specific bitmask, though return the unwritten value first
fn toggle_bit(
    bitmask: &mut usize,
    index: usize,
    value: bool,
) -> bool {
    let copy = (*bitmask >> index) & 1 == 1;

    if value {
        *bitmask |= 1 << index;
    } else {
        *bitmask &= !(1 << index);
    }

    copy
}

// Enable all the bits between "start" and "end" in the binary representation of a usize
// Start is inclusive, end is exclusive
pub(crate) fn enable_in_range(start: usize, end: usize) -> usize {
    assert!(end >= start);

    if end == BITS {
        !((1usize << (start)) - 1usize)
    } else if start == BITS {
        0
    } else {
        ((1usize << (start)) - 1usize) ^ ((1usize << end) - 1usize)
    }
}

// Check if a bit at a specific index is set
fn is_bit_enabled(bitset: usize, index: usize) -> bool {
    bitset >> index & 1 == 1
}

 */

// Update a value in a specific bitmask, though return the unwritten value first
pub fn toggle_bit<T: PrimInt>(
    bitmask: &mut T,
    index: usize,
    value: bool,
) -> bool {
    let copy = ((*bitmask >> index) & T::one()) == T::one();

    if value {
        *bitmask = *bitmask | (T::one() << index);
    } else {
        *bitmask = *bitmask & (!(T::one() << index));
    }

    copy
}

// Enable all the bits between "start" and "end" in the binary representation of a T
// Start is inclusive, end is exclusive
pub fn enable_in_range<T: PrimInt>(start: usize, end: usize) -> T {
    assert!(end >= start);
    let bits = size_of::<T>() * 8;

    if end == bits {
        !((T::one() << (start)) - T::one())
    } else if start == bits {
        T::zero()
    } else {
        ((T::one() << (start)) - T::one())
            ^ ((T::one() << end) - T::one())
    }
}

// Check if a bit at a specific index is set
pub fn is_bit_enabled<T: PrimInt>(bitset: T, index: usize) -> bool {
    (bitset >> index & T::one()) == T::one()
}
