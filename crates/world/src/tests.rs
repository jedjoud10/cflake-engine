#[cfg(test)]
mod tests {
    use crate::ThreadPool;
    #[test]
    fn test() {
        let mut threadpool = ThreadPool::new();
        dbg!(threadpool.num_threads());
        let mut vec = (0..=64).into_iter().collect::<Vec<u128>>();
        let mut vec2 = (0..=64).into_iter().collect::<Vec<u128>>();
        let shared = 4;

        threadpool.for_each(
            (vec.as_mut_slice(), vec2.as_mut_slice()),
            |(a, b)| {
                dbg!((a, b));
            },
            2,
        );
    }
}
