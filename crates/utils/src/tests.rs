#[cfg(test)]
mod bitset {
    use crate::bitset::BitSet;

    #[test]
    fn bitset() {
        let mut bitset = BitSet::<usize>::new();
        assert!(!bitset.get(0));
        bitset.set(0);
        assert!(bitset.get(0));
        assert_eq!(bitset.chunks().len(), 1);
        bitset.set(64);
        bitset.set(65);
        assert_eq!(bitset.chunks().len(), 2);
        assert!(bitset.get(64));
        assert!(bitset.get(65));

        bitset.remove(0);
        assert!(!bitset.get(0));
        assert!(bitset.get(64));
        assert!(bitset.get(65));
    }

    #[test]
    fn counting_ones() {
        let mut bitset = BitSet::<usize>::new();
        bitset.set(0);
        bitset.set(10);
        assert_eq!(bitset.count_ones(), 2);
        assert_eq!(bitset.find_one_from(0), Some(0));
        assert_eq!(bitset.find_one_from(1), Some(10));
        assert_eq!(bitset.find_one_from(11), None);
        assert_eq!(bitset.find_one_from(10), Some(10));

        bitset.set(4096);
        assert_eq!(bitset.find_one_from(11), Some(4096));
        assert_eq!(bitset.find_one_from(4098), None);
    }

    #[test]
    fn counting_zeros() {
        let mut bitset = BitSet::<usize>::new();
        bitset.set(0);
        bitset.set(10);
        assert_eq!(bitset.count_zeros(), usize::BITS as usize - 2);
        assert_eq!(bitset.find_zero_from(0), Some(1));
        assert_eq!(bitset.find_zero_from(1), Some(1));

        bitset.set(4096);
        assert_eq!(bitset.find_zero_from(10), Some(11));
        assert_eq!(bitset.find_zero_from(11), Some(11));
    }

    #[test]
    fn pattern() {
        let mut bitset = BitSet::<usize>::new();
        bitset.set(0);
        bitset.set(2);
        bitset.set(4);

        assert_eq!(bitset.count_ones(), 3);
        assert_eq!(bitset.find_one_from(0), Some(0));
        assert_eq!(bitset.find_one_from(1), Some(2));
        assert_eq!(bitset.find_one_from(2), Some(2));
        assert_eq!(bitset.find_one_from(3), Some(4));
    }

    #[test]
    fn pattern_all_set() {
        let mut bitset = BitSet::<usize>::new();
        bitset.set(0);
        bitset.set(1);
        bitset.set(2);
        bitset.set(3);
        bitset.set(4);

        assert_eq!(bitset.count_ones(), 5);
        assert_eq!(bitset.find_one_from(0), Some(0));
        assert_eq!(bitset.find_one_from(1), Some(1));
        assert_eq!(bitset.find_one_from(2), Some(2));
        assert_eq!(bitset.find_one_from(3), Some(3));
    }
}

#[cfg(test)]
mod atomic_bitset {
    use crate::bitset::AtomicBitSet;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn bitset() {
        let bitset = AtomicBitSet::<AtomicUsize>::new();
        assert!(!bitset.get(0, Ordering::Relaxed));
        bitset.set(0, Ordering::Relaxed);
        assert!(bitset.get(0, Ordering::Relaxed));
        assert_eq!(bitset.chunks().len(), 1);
        bitset.set(64, Ordering::Relaxed);
        bitset.set(65, Ordering::Relaxed);
        assert_eq!(bitset.chunks().len(), 2);
        assert!(bitset.get(64, Ordering::Relaxed));
        assert!(bitset.get(65, Ordering::Relaxed));

        bitset.remove(0, Ordering::Relaxed);
        assert!(!bitset.get(0, Ordering::Relaxed));
        assert!(bitset.get(64, Ordering::Relaxed));
        assert!(bitset.get(65, Ordering::Relaxed));
    }

    #[test]
    fn counting_ones() {
        let bitset = AtomicBitSet::<AtomicUsize>::new();
        bitset.set(0, Ordering::Relaxed);
        bitset.set(10, Ordering::Relaxed);
        assert_eq!(bitset.count_ones(Ordering::Relaxed), 2);
        assert_eq!(bitset.find_one_from(0, Ordering::Relaxed), Some(0));
        assert_eq!(bitset.find_one_from(1, Ordering::Relaxed), Some(10));
        assert_eq!(bitset.find_one_from(11, Ordering::Relaxed), None);
        assert_eq!(bitset.find_one_from(10, Ordering::Relaxed), Some(10));

        bitset.set(4096, Ordering::Relaxed);
        assert_eq!(bitset.find_one_from(11, Ordering::Relaxed), Some(4096));
        assert_eq!(bitset.find_one_from(4098, Ordering::Relaxed), None);
    }

    #[test]
    fn counting_zeros() {
        let bitset = AtomicBitSet::<AtomicUsize>::new();
        bitset.set(0, Ordering::Relaxed);
        bitset.set(10, Ordering::Relaxed);
        assert_eq!(
            bitset.count_zeros(Ordering::Relaxed),
            usize::BITS as usize - 2
        );
        assert_eq!(bitset.find_zero_from(0, Ordering::Relaxed), Some(1));
        assert_eq!(bitset.find_zero_from(1, Ordering::Relaxed), Some(1));

        bitset.set(4096, Ordering::Relaxed);
        assert_eq!(bitset.find_zero_from(10, Ordering::Relaxed), Some(11));
        assert_eq!(bitset.find_zero_from(11, Ordering::Relaxed), Some(11));
    }

    #[test]
    fn pattern() {
        let bitset = AtomicBitSet::<AtomicUsize>::new();
        bitset.set(0, Ordering::Relaxed);
        bitset.set(2, Ordering::Relaxed);
        bitset.set(4, Ordering::Relaxed);

        assert_eq!(bitset.count_ones(Ordering::Relaxed), 3);
        assert_eq!(bitset.find_one_from(0, Ordering::Relaxed), Some(0));
        assert_eq!(bitset.find_one_from(1, Ordering::Relaxed), Some(2));
        assert_eq!(bitset.find_one_from(2, Ordering::Relaxed), Some(2));
        assert_eq!(bitset.find_one_from(3, Ordering::Relaxed), Some(4));
    }
}

#[cfg(test)]
mod enable_in_range {
    use crate::bitset::enable_in_range;
    #[test]
    fn bit_range_setter() {
        let r01: usize = enable_in_range(0, 1);
        assert_eq!(r01, 1);
        assert_eq!(r01.count_ones(), 1);

        let r23: usize = enable_in_range(2, 3);
        assert_eq!(r23, 1 << 2);
        assert_eq!(r23.count_ones(), 1);

        let all: usize = enable_in_range(0, usize::BITS as usize);
        assert_eq!(all, usize::MAX);
        assert_eq!(all.count_ones(), usize::BITS as u32);

        let none: usize = enable_in_range(0, 0);
        assert_eq!(none, usize::MIN);
        assert_eq!(none.count_ones(), 0);

        let half: usize = enable_in_range(usize::BITS as usize / 2, usize::BITS as usize);
        assert_eq!(half.count_ones(), usize::BITS as u32 / 2);
        assert_eq!(half.count_zeros(), usize::BITS as u32 / 2);

        let test: usize = enable_in_range(usize::BITS as usize - 1, usize::BITS as usize);
        assert_eq!(test, 1 << (usize::BITS as usize - 1));
    }
}

#[cfg(test)]
mod storage {
    use crate::storage::Storage;

    #[test]
    fn init() {
        let mut storage: Storage<u32> = Storage::<u32>::default();
        let zero = storage.insert(0);
        let one = storage.insert(1);
        assert_eq!(0, *storage.get(&zero));
        assert_eq!(1, *storage.get(&one));
    }
}
