#[cfg(test)]
mod tests {
    use std::{time::Instant, hint::spin_loop};

    use crate::{ThreadPool};

    fn task(integer: &u128) {
        dbg!(integer);
        //
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    #[test]
    fn test() {
        let mut threadpool = ThreadPool::new();
        dbg!(threadpool.num_threads());
        //dbg!(threadpool.num_active_threads());
    
        let mut vec = (0..=64).into_iter().collect::<Vec<u128>>();
        let i = Instant::now();
        threadpool.for_each(vec.as_slice(), task, 32);
        dbg!(i.elapsed().as_micros());
        dbg!(vec);
        /*
        let item = slice.fetch(0).unwrap();
        let item = slice.fetch(0).unwrap();
        */
    }
}