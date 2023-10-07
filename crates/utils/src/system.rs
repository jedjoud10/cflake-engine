use std::{
    sync::mpsc,
};

/// Utils resources that is added to the world at the very start
pub struct UtilsSettings {
    pub author_name: String,
    pub app_name: String,
    pub tick_rate: u32,
    pub tick_rate_max: u32,
    pub log_receiver: Option<mpsc::Receiver<String>>,
}

/*
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

// Add the Time manager
pub fn time(system: &mut System) {
    // Main initialization event
    system
        .insert_init(|world: &mut World| {
            let settings = world.get::<UtilsSettings>().unwrap();

            let time = Time {
                delta: Duration::ZERO,
                frame_count: 0,
                startup: Instant::now(),
                frame_start: Instant::now(),
                tick_count: 0,
                last_tick_start: Instant::now(),
                ticks_to_execute: None,
                tick_delta: Duration::from_secs_f32(1.0 / settings.tick_rate as f32),
                local_tick_count: 0,
                tick_interpolation: 0.0,
                accumulator: 0.0,
                tick_rate: settings.tick_rate,
                tick_rate_max: settings.tick_rate_max,
            };

            drop(settings);
            world.insert(time);
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

            // https://gafferongames.com/post/fix_your_timestep/
            time.accumulator += time.delta.as_secs_f32();
            time.tick_interpolation = time.accumulator / time.tick_delta.as_secs_f32();

            while time.accumulator > time.tick_delta.as_secs_f32() {
                time.local_tick_count = 0;

                // Add one to ticks to execute
                if let Some(count) = time.ticks_to_execute.as_mut() {
                    *count = count.checked_add(1).unwrap();
                } else {
                    time.ticks_to_execute = Some(NonZeroU32::new(1).unwrap());
                }

                // Decrease delta and reset interpolations
                time.accumulator -= time.tick_delta.as_secs_f32();
                time.tick_interpolation = 1.0;
            }

            // LIMIT TICKS WHEN WE HAVE SPIRAL OF DEATH
            let tick_rate_max = time.tick_rate_max;
            if let Some(count) = time.ticks_to_execute.as_mut() {
                const MAX_TICKS_DURING_SLOWDOWN: u32 = 1;

                if count.get() > tick_rate_max {
                    log::warn!("Too many ticks to execute! Spiral of death effect is occuring");
                    *count = NonZeroU32::new(MAX_TICKS_DURING_SLOWDOWN).unwrap();
                }
            }
        })
        .before(user);

    // Insert the tick event that will increase the tick count as well
    system
        .insert_tick(|world: &mut World| {
            let mut time = world.get_mut::<Time>().unwrap();
            time.tick_count += 1;
            time.local_tick_count += 1;
            time.ticks_to_execute = None;
        })
        .before(user);
}

// Add the event cleaner system
pub fn per_frame_event_clean(system: &mut System) {
    system
        .insert_update(|world: &mut World| {
            if let Some(cleaner) = crate::PER_FRAME_EVENTS_CACHE_CLEANER.get() {
                let locked = cleaner.lock();

                for (_, callback) in locked.iter() {
                    callback(world)
                }
            }
        })
        .after(post_user);
}
*/
