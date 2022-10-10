#[cfg(test)]
mod tests {
    use crate::ThreadPool;

    #[test]
    fn test() {
        let mut threadpool = ThreadPool::new();
        dbg!(threadpool.num_threads());
        dbg!(threadpool.num_active_threads());

        threadpool.execute(|| println!("Executing on: {:?}", std::thread::current().name()));
        threadpool.execute(|| println!("Executing on: {:?}", std::thread::current().name()));
        threadpool.execute(|| println!("Executing on: {:?}", std::thread::current().name()));
        threadpool.execute(|| println!("Executing on: {:?}", std::thread::current().name()));
        threadpool.execute(|| println!("Executing on: {:?}", std::thread::current().name()));
        threadpool.execute(|| println!("Executing on: {:?}", std::thread::current().name()));
    }
}