#[cfg(test)]
pub mod test {
    use crate::ThreadPool;

    #[test]
    // Just a normal test to see if it crashes or not
    pub fn test() {
        // Test the parralelization
        let pool = ThreadPool::<(), i32>::new(16);
        let mut numbers = vec![0; 512];
        pool.execute(&mut numbers, &(), |a, b| {}, 64);
    }
    #[test]
    // Test the speed compared to single threaded
    pub fn speed_test() {
        // Test the parralelization
        let pool = ThreadPool::<(), i32>::new(70);
        let mut numbers1 = vec![0; 4096];
        // Some sort of expensive calculation
        fn expensive_calculation() -> i32 {
            let mut i = 0;
            for x in 0..512 { i += x; };
            i - 512 + 2
        }
        let i = std::time::Instant::now();
        pool.execute(&mut numbers1, &(), |a, b| { *b = expensive_calculation() }, 64);
        println!("Took '{}' micros to execute multithreaded code", i.elapsed().as_micros());

        let mut numbers2 = vec![0; 4096];
        let i = std::time::Instant::now();        
        for b in numbers2.iter_mut() {
            *b = expensive_calculation()
        }
        println!("Took '{}' micros to execute singlethreaded code", i.elapsed().as_micros());

        // We need to make sure that both are the same
        assert_eq!(numbers1, numbers2);
    }
}