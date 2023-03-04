use std::{
    num::{NonZeroU128, NonZeroU32},
    time::{Duration, Instant},
    io::{Write, BufWriter}, fs::File, sync::mpsc,
};

use crate::{FileManager, ThreadPool, Time, FileType};
use world::{post_user, user, System, World};

// Utils resources that is added to the world at the very start
pub struct UtilsSettings {
    pub threadpool_max_threads: Option<usize>,
    pub author_name: String,
    pub app_name: String,
    pub log_receiver: Option<mpsc::Receiver<String>>,
}

// Add the threadpool resource to the world
pub fn threadpool(system: &mut System) {
    // Main initialization event
    system
        .insert_init(|world: &mut World| {
            let num = num_cpus::get();
            world.insert(ThreadPool::with(num))
        })
        .before(user);

    // Update event that check if any of the threads panicked
    system
        .insert_update(|world: &mut World| {
            let pool = world.get::<ThreadPool>().unwrap();
            if let Some(id) = pool.check_any_panicked() {
                panic!("WorkerThread {} panicked", id);
            }
        })
        .after(post_user);
}

// Add the IO path manager
pub fn io(system: &mut System) {
    system
        .insert_init(move |world: &mut World| {
            let settings = world.get::<UtilsSettings>().unwrap();
            let manager = FileManager::new(&settings.author_name, &settings.app_name);
            drop(settings);
            world.insert(manager);
        })
        .before(user);
}

// Add the file logger
pub fn file_logger(system: &mut System) {
    // Get the file name
    let file = chrono::Local::now().format("%Y-%m-%d.log").to_string();

    system
        .insert_init(move |world: &mut World| {
            // Get the utils settings that are added by the app
            let mut settings = world.get_mut::<UtilsSettings>().unwrap();
            let receiver = settings.log_receiver.take().unwrap();

            // Get the file manager to get the log file
            let mut manager = world.get_mut::<FileManager>().unwrap();
            let mut file = manager.write_file(&file, true, FileType::Log).unwrap();

            // Create a secondary thread that will be responsible for logging these events
            std::thread::spawn(move || {
                // This receiver will receive the logged messages from the fern dispatcher
                for line in receiver.iter().filter(|x| !x.is_empty()) {
                    write!(file, "{}", line).unwrap();
                }
            });            
        })
        .before(user)
        .after(io);
}

// Number of ticks that should execute per second
const TICKS_PER_SEC: f32 = 120.0f32;

// Add the Time manager
pub fn time(system: &mut System) {
    // Main initialization event
    system
        .insert_init(|world: &mut World| {
            world.insert(Time {
                delta: Duration::ZERO,
                frame_count: 0,
                startup: Instant::now(),
                frame_start: Instant::now(),
                tick_count: 0,
                last_tick_start: Instant::now(),
                ticks_to_execute: None,
                tick_delta: Duration::from_secs_f32(
                    1.0 / TICKS_PER_SEC,
                ),
            });
        })
        .before(user);

    // Update event that will mutate the time fields
    system
        .insert_update(|world: &mut World| {
            let mut time = world.get_mut::<Time>().unwrap();
            let now = Instant::now();

            // Update frame count and frame start
            let old_frame_start = time.frame_start;
            time.frame_start = now;
            time.frame_count += 1;

            // Calculate delta (using old frame start)
            time.delta = now - old_frame_start;

            // Constants needed for ticks
            const TICKS_DELTA_NS: f32 =
                (1.0 / TICKS_PER_SEC) * 1000000000.0;
            const TICK_DELTA: Duration =
                Duration::from_nanos(TICKS_DELTA_NS as u64);

            // Update the tick count and starts
            let diff = now - time.last_tick_start;
            if diff >= TICK_DELTA {
                // Calculate how many ticks have elapsed since the last tick
                let divided = diff.as_micros() as f32
                    / TICK_DELTA.as_micros() as f32;
                let count = divided.floor() as u32;

                // Add divided tick count to accumulator
                time.last_tick_start = now;
                time.tick_count += count as u128;
                time.ticks_to_execute = NonZeroU32::new(count);
            } else {
                time.ticks_to_execute = None;
            }
        })
        .before(user);

    // Insert the tick event that will increase the tick count as well
    system
        .insert_tick(|world: &mut World| {
            let mut time = world.get_mut::<Time>().unwrap();
            time.tick_count += 1;
        })
        .before(user);
}
