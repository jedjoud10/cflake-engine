#[cfg(test)]
mod tests {
    use std::{hint::spin_loop, time::Instant};

    use crate::ThreadPool;

    fn task(integer: &mut u128) {}

    #[test]
    fn test() {
        let mut threadpool = ThreadPool::new();
        dbg!(threadpool.num_threads());
        //dbg!(threadpool.num_active_threads());

        /*
        let i = Instant::now();
        threadpool.for_each(vec.as_mut_slice(), task, 128);
        dbg!(i.elapsed().as_micros());
        dbg!(vec);
        */
        let mut vec = (0..=64).into_iter().collect::<Vec<u128>>();
        let shared = 4;

        threadpool.scope(|scope| {
            scope.execute(|| {
                dbg!(shared);
            });
            scope.execute(|| {
                dbg!(shared);
            });
        });

        threadpool.for_each(
            vec.as_mut_slice(),
            |mutable: &mut u128| {
                dbg!(shared);
            },
            128,
        );

        threadpool.for_each(
            vec.as_mut_slice(),
            |mutable: &mut u128| {
                dbg!(shared);
            },
            128,
        );

        /*
        let item = slice.fetch(0).unwrap();
        let item = slice.fetch(0).unwrap();
        */
    }
}
