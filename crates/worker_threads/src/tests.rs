#[cfg(test)]
pub mod test {
    use crate::ThreadPool;

    #[test]
    // Just a normal test to see if it crashes or not
    pub fn test() {
        // Test the parralelization
        let pool = ThreadPool::<(), i32>::new(8, || {});
        let mut numbers = vec![0; 512];
        pool.execute(&mut numbers, &(), |_, _| {});
    }
    #[test]
    // Test the speed compared to single threaded
    pub fn speed_test() {
        // Test the parralelization
        let pool = ThreadPool::<(), i32>::new(8, || {});
        let mut numbers1 = vec![0; 4096];
        // Some sort of expensive calculation
        fn expensive_calculation() -> i32 {
            let mut l = 0;
            for x in 0..512 { l += x; };
            l
        }
        let i = std::time::Instant::now();
        pool.execute(&mut numbers1, &(), |a, b| { *b = expensive_calculation() });
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
    /*
    #[test]
    // Benchmarking the best min chunk size
    pub fn benchmark() {
        let pool = ThreadPool::<(), i32>::new(8, || {});
        // Some sort of expensive calculation
        fn expensive_calculation() -> i32 {
            let mut l = 0;
            //for x in 0..32 { l += x; };
            l
        }
        let mut numbers2 = vec![0; 32768];
        let i = std::time::Instant::now();        
        for b in numbers2.iter_mut() {
            *b = expensive_calculation()
        }
        println!("Took '{}' micros to execute singlethreaded code", i.elapsed().as_micros());

        for x in 0..14 {
            let min_chunk_size = 2_usize.pow(x);
            let t: u128 = (0..64).into_iter().map(|_| {
                let mut numbers1 = vec![0; 32768];
                let i = std::time::Instant::now();
                pool.execute(&mut numbers1, &(), |a, b| { *b = expensive_calculation() });
                assert_eq!(numbers1, numbers2);
                i.elapsed().as_micros()
            }).sum();
            println!("Took '{}' micros to execute code. Min chunk size: {}", t / 32, min_chunk_size);
        }
    }
    */
}