#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::ThreadPool;

    fn task() {
        std::thread::sleep(std::time::Duration::new(8, 0));
        println!("Executing on: {:?}", std::thread::current().name())
    }

    #[test]
    fn test() {
        let mut threadpool = ThreadPool::new();
        dbg!(threadpool.num_threads());
        //dbg!(threadpool.num_active_threads());
    
        let vec = (0..16).into_iter().collect::<Vec<u32>>();
        let i = Instant::now();
        dbg!(i.elapsed().as_millis());
    }
}