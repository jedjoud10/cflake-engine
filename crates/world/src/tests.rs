#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::{ThreadPool};

    fn task(integer: &u32) {
        //std::thread::sleep(std::time::Duration::from_millis(100));
        //println!("Executing on: {:?}", std::thread::current().name())
        //dbg!(integer);
    }

    #[test]
    fn test() {
        let mut threadpool = ThreadPool::new();
        dbg!(threadpool.num_threads());
        //dbg!(threadpool.num_active_threads());
    
        let vec = (0..10).into_iter().collect::<Vec<u32>>();
        let i = Instant::now();
        threadpool.for_each(&vec.as_slice(), task, 32);
        dbg!(i.elapsed().as_micros());

        let mut vector = vec![0u32; 64];
        let mut slice = vector.as_mut_slice();
        /*
        let item = slice.fetch(0).unwrap();
        let item = slice.fetch(0).unwrap();
        */
    }
}