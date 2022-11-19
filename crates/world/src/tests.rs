#[cfg(test)]
mod threadpool {
    use math::BitSet;

    use crate::ThreadPool;

    #[test]
    fn data() {
        let mut threadpool = ThreadPool::default();
        let mut vec = (0..64).into_iter().collect::<Vec<u64>>();

        threadpool.for_each(
            vec.as_mut_slice(),
            |value| {
                *value += *value;
            },
            8,
        );

        for (i, x) in vec.iter().enumerate() {
            assert_eq!(2 * i as u64, *x)
        }
    }

    #[test]
    fn hop() {
        let mut threadpool = ThreadPool::default();
        let mut vec =
            (0..64).into_iter().map(|_| 100).collect::<Vec<u64>>();
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
        let mut vec = (0..=64).into_iter().collect::<Vec<u128>>();
        let mut vec2 = (0..=64).into_iter().collect::<Vec<u128>>();
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
