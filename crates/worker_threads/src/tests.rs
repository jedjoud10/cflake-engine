#[cfg(test)]
pub mod test {
    use std::sync::Arc;

    use crate::{ThreadPool, SharedVec};

    #[test]
    // Just a normal test to see if it crashes or not
    pub fn test() {
        // Test the parralelization
        let pool = ThreadPool::<i32>::new(8, || {});
        let mut numbers = vec![0; 512];
        let data = 10;
        pool.execute(&mut numbers, |x| {
            *x = data;
        });
    }
    #[test]
    // Test the shared vector
    pub fn test_vec() {
        let pool = ThreadPool::<i32>::new(8, || {});
        let mut numbers = (0..65535).collect::<Vec<_>>();
        let shared = SharedVec::<i32>::new( 65535);
        let data = 10;
        pool.execute(&mut numbers, |x| {
            *x += data;
            let y = shared.write().unwrap();
            *y += *x;
        });
        let numbers2 = (10..65545).collect::<Vec<_>>();
        assert_eq!(numbers, numbers2);
    }
    #[test]
    // Test the speed compared to single threaded
    pub fn speed_test() {
        // Test the parralelization
        let pool = ThreadPool::<i32>::new(8, || {});
        let mut numbers1 = vec![0; 4];
        // Some sort of expensive calculation
        fn expensive_calculation() -> i32 {
            let l: i32 = 0;
            l
        }
        let i = std::time::Instant::now();
        pool.execute(&mut numbers1, |b| *b = expensive_calculation());
        println!("Took '{}' micros to execute multithreaded code", i.elapsed().as_micros());

        let mut numbers2 = vec![0; 4];
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
