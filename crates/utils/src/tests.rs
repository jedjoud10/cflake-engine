#[cfg(test)]
mod threadpool {
    use std::time::Instant;

    use crate::BitSet;
    use crate::ThreadPool;

    #[test]
    fn data() {
        let mut threadpool = ThreadPool::default();
        let mut vec = (0..64).collect::<Vec<u64>>();

        let test = Instant::now();
        threadpool.for_each(
            vec.as_mut_slice(),
            |value| {
                *value += *value;
            },
            8,
        );
        dbg!(test.elapsed());

        for (i, x) in vec.iter().enumerate() {
            assert_eq!(2 * i as u64, *x)
        }
    }

    #[test]
    fn hop() {
        let mut threadpool = ThreadPool::default();
        let mut vec =
            (0..64).map(|_| 100).collect::<Vec<u64>>();
        let bitset = BitSet::from_pattern(|x| x % 2 == 0, 64);
        dbg!(&bitset);

        threadpool.for_each_filtered(
            vec.as_mut_slice(),
            |value| {
                *value = 0;
            },
            bitset,
            128,
        );

        for (i, x) in vec.iter().enumerate() {
            if i % 2 == 0 {
                assert_eq!(*x, 100);
            } else {
                assert_eq!(*x, 0);
            }
        }
    }

    #[test]
    fn count() {
        let mut threadpool = ThreadPool::default();
        dbg!(threadpool.num_threads());
        let mut vec = (0..=64).collect::<Vec<u128>>();
        let mut vec2 = (0..=64).collect::<Vec<u128>>();
        let _shared = 4;

        threadpool.for_each(
            (vec.as_mut_slice(), vec2.as_mut_slice()),
            |(a, b)| {
                dbg!((a, b));
            },
            2,
        );
    }
}

#[cfg(test)]
mod bitset {
    use crate::BitSet;

    #[test]
    fn bitset() {
        let mut bitset = BitSet::new();
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
        let mut bitset = BitSet::new();
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
        let mut bitset = BitSet::new();
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
        let mut bitset = BitSet::new();
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
        let mut bitset = BitSet::new();
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
    use crate::AtomicBitSet;
    use std::sync::atomic::Ordering;

    #[test]
    fn bitset() {
        let bitset = AtomicBitSet::new();
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
        let bitset = AtomicBitSet::new();
        bitset.set(0, Ordering::Relaxed);
        bitset.set(10, Ordering::Relaxed);
        assert_eq!(bitset.count_ones(Ordering::Relaxed), 2);
        assert_eq!(
            bitset.find_one_from(0, Ordering::Relaxed),
            Some(0)
        );
        assert_eq!(
            bitset.find_one_from(1, Ordering::Relaxed),
            Some(10)
        );
        assert_eq!(bitset.find_one_from(11, Ordering::Relaxed), None);
        assert_eq!(
            bitset.find_one_from(10, Ordering::Relaxed),
            Some(10)
        );

        bitset.set(4096, Ordering::Relaxed);
        assert_eq!(
            bitset.find_one_from(11, Ordering::Relaxed),
            Some(4096)
        );
        assert_eq!(
            bitset.find_one_from(4098, Ordering::Relaxed),
            None
        );
    }

    #[test]
    fn counting_zeros() {
        let bitset = AtomicBitSet::new();
        bitset.set(0, Ordering::Relaxed);
        bitset.set(10, Ordering::Relaxed);
        assert_eq!(
            bitset.count_zeros(Ordering::Relaxed),
            usize::BITS as usize - 2
        );
        assert_eq!(
            bitset.find_zero_from(0, Ordering::Relaxed),
            Some(1)
        );
        assert_eq!(
            bitset.find_zero_from(1, Ordering::Relaxed),
            Some(1)
        );

        bitset.set(4096, Ordering::Relaxed);
        assert_eq!(
            bitset.find_zero_from(10, Ordering::Relaxed),
            Some(11)
        );
        assert_eq!(
            bitset.find_zero_from(11, Ordering::Relaxed),
            Some(11)
        );
    }

    #[test]
    fn pattern() {
        let bitset = AtomicBitSet::new();
        bitset.set(0, Ordering::Relaxed);
        bitset.set(2, Ordering::Relaxed);
        bitset.set(4, Ordering::Relaxed);

        assert_eq!(bitset.count_ones(Ordering::Relaxed), 3);
        assert_eq!(
            bitset.find_one_from(0, Ordering::Relaxed),
            Some(0)
        );
        assert_eq!(
            bitset.find_one_from(1, Ordering::Relaxed),
            Some(2)
        );
        assert_eq!(
            bitset.find_one_from(2, Ordering::Relaxed),
            Some(2)
        );
        assert_eq!(
            bitset.find_one_from(3, Ordering::Relaxed),
            Some(4)
        );
    }
}

#[cfg(test)]
mod hibitset {
    use crate::HiBitSet;

    #[test]
    fn hibitset() {
        let mut hibitset = HiBitSet::new();
        hibitset.set(0);
        assert!(hibitset.get(0));
        assert!(!hibitset.get(1));
        assert!(hibitset.get(0));
        hibitset.remove(0);
        assert!(!hibitset.get(0));

        hibitset.set(4095);
        assert!(hibitset.get(4095));
    }
}

#[cfg(test)]
mod storage {
    use crate::Storage;

    #[test]
    fn init() {
        let mut storage: Storage<u32> = Storage::<u32>::default();
        let zero = storage.insert(0);
        let one = storage.insert(1);
        assert_eq!(0, *storage.get(&zero));
        assert_eq!(1, *storage.get(&one));
    }
}
