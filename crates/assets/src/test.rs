#[cfg(test)]
mod test {
    use std::time::Instant;

    use crate::{Assets, persistent};

    #[test]
    fn test() {
        let loader = Assets::new(None);
        persistent!(loader, "test/text.txt");
        let i = Instant::now();
        dbg!("Loading 10 async assets");
        let mut vec = Vec::new();
        for x in 0..10 {
            let handle = loader.threaded_load::<String>("test/text.txt");
            vec.push(handle);
        }

        for x in vec {
            loader.wait(x);
        }

        dbg!(i.elapsed().as_millis());

        dbg!("Loading 10 sync assets");
        let i = Instant::now();
        for x in 0..10 {
            let string = loader.load::<String>("test/text.txt").unwrap();
        }
        dbg!(i.elapsed().as_millis());
    }
}