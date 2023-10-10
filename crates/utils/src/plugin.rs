use std::{sync::mpsc, num::NonZeroU32, time::{Instant, Duration}, fs::OpenOptions, io::{BufWriter, Write, LineWriter}};
use world::{prelude::{Init, Update}, world::World, system::{Registries, pre_user}};
use crate::time::Time;

/// Utils resources that is added to the world at the very start
pub struct UtilsSettings {
    /// Name of the author (for the log file)
    pub author_name: String,

    /// Name of the app (for the log file)
    pub app_name: String,

    /// Tick rate per second
    pub tick_rate: u32,

    /// Max number of ticks allowed in one frame
    pub tick_rate_max: u32,

    /// Log receiver for the log file
    /// This is always set to [Some], and then it is taken internally by the [init_file_logger] function
    pub log_receiver: Option<mpsc::Receiver<String>>,
}

/// Init system for adding the file logger resource
pub fn init_file_logger(world: &mut World, _: &Init) {
    // Get the utils settings that are added by the app
    let mut settings = world.get_mut::<UtilsSettings>().unwrap();
    let receiver = settings.log_receiver.take().unwrap();
    let file_name = chrono::Local::now().format("%Y-%m-%d.log").to_string();

    // Create the log directory if not already created
    let mut log_dir_path = dirs::cache_dir().expect("Cache dir path could not be found");
    log_dir_path.extend([&settings.author_name, &settings.app_name, "log"]);
    
    if !log_dir_path.exists() {
        std::fs::DirBuilder::new().recursive(true).create(&log_dir_path).unwrap();
    }
    
    // Create a log file to write to
    let mut log_file = log_dir_path.clone();
    log_file.push(file_name);
    let file = OpenOptions::new()
        .truncate(true)
        .create(true)
        .write(true)
        .open(log_file)
        .expect("Could not create/open log file");
    let mut file = LineWriter::new(file);

    // Create a secondary thread that will be responsible for logging these events
    std::thread::spawn(move || {
        for line in receiver.iter().filter(|x| !x.is_empty()) {
            write!(file, "{}", line).unwrap();
        }
    });
}

/// Init system for adding the time resource
pub fn init_time(world: &mut World, _: &Init) {
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
}


/// Update system for update the time resource
pub fn update_time(world: &mut World, _: &Update) {
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

    /*
    // LIMIT TICKS WHEN WE HAVE SPIRAL OF DEATH
    let tick_rate_max = time.tick_rate_max;
    if let Some(count) = time.ticks_to_execute.as_mut() {
        const MAX_TICKS_DURING_SLOWDOWN: u32 = 1;

        if count.get() > tick_rate_max {
            log::warn!("Too many ticks to execute! Spiral of death effect is occuring");
            *count = NonZeroU32::new(MAX_TICKS_DURING_SLOWDOWN).unwrap();
        }
    }
    */
}

/// Main utils plugin that willo add all of these systems
pub fn plugin(registries: &mut Registries) {
    registries.init.insert(init_time).before(pre_user);
    registries.init.insert(init_file_logger).before(pre_user);
    registries.update.insert(update_time).before(pre_user);
}

/*
let mut time = world.get_mut::<Time>().unwrap();
time.tick_count += 1;
time.local_tick_count += 1;
time.ticks_to_execute = None;
*/