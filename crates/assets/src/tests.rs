#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::{persistent, Assets};

    #[test]
    fn test() {
        let mut threadpool = world::ThreadPool::default();
        let loader = Assets::new(None);
        persistent!(loader, "test/text.txt");
        let i = Instant::now();
        dbg!("Loading 100 async assets");
        let mut vec = Vec::new();
        for _x in 0..100 {
            let handle = loader
                .async_load::<String>("test/text.txt", &mut threadpool)
                .unwrap();
            vec.push(handle);
        }

        for x in vec {
            let _text = loader.wait(x);
        }

        dbg!(i.elapsed().as_millis());

        dbg!("Loading 100 sync assets");
        let i = Instant::now();
        for _x in 0..100 {
            let _string = loader.load::<String>("test/text.txt").unwrap();
        }
        dbg!(i.elapsed().as_millis());
    }
}
