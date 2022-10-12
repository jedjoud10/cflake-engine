#[cfg(test)]
mod tests {
    use std::{time::Instant, hint::spin_loop};

    use crate::{ThreadPool};

    fn task(integer: &mut u128) {
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    #[test]
    fn test() {
        let mut threadpool = ThreadPool::new();
        dbg!(threadpool.num_threads());
        //dbg!(threadpool.num_active_threads());
    
        let mut vec = (0..=8).into_iter().collect::<Vec<u128>>();
        let i = Instant::now();
        threadpool.for_each_mut(vec.as_mut_slice(), task, 1);
        dbg!(i.elapsed().as_micros());
        dbg!(vec);
        /*
        let item = slice.fetch(0).unwrap();
        let item = slice.fetch(0).unwrap();
        */
    }
}