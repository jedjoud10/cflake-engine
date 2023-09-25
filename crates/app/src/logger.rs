use std::{sync::mpsc, str::FromStr};
use fern::colors::ColoredLevelConfig;
use log::LevelFilter;

fn file_logger(sender: mpsc::Sender<String>) -> fern::Dispatch {
    fern::Dispatch::new()
        .format(move |out, _, record| {
            out.finish(format_args!(
                "[{thread_name}][{date}][{target}][{level}] {message}",
                date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                target = record.target(),
                level = record.level(),
                message = record.args(),
                thread_name = std::thread::current().name().unwrap_or("none")
            ));
        })
        .chain(sender)
}

fn console_logger(
    colors_level: ColoredLevelConfig,
    colors_line: ColoredLevelConfig,
) -> fern::Dispatch {
    fern::Dispatch::new().format(move |out, message, record| {
        out.finish(format_args!(
            "{color_line}[{thread_name}][{date}][{target}][{level}{color_line}] {message}\x1B[0m",
            color_line = format_args!(
                "\x1B[{}m",
                colors_line.get_color(&record.level()).to_fg_str()
            ),
            date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            target = record.target(),
            level = colors_level.color(record.level()),
            message = message,
            thread_name = std::thread::current().name().unwrap_or("none"),
        ));
    }).chain(std::io::stdout())
}

pub(crate) fn init_logger(mut logging_level: LevelFilter, sender: mpsc::Sender<String>) {
    use fern::colors::*;

    let overwrite = std::env::var("CFLAKE_LOGGING_TRACE")
        .as_deref()
        .ok()
        .map(|x| <LevelFilter as FromStr>::from_str(x).ok())
        .flatten();

    if let Some(new) = overwrite {
        logging_level = new;
    }

    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::White)
        .debug(Color::White)
        .trace(Color::BrightBlack);

    let colors_level = colors_line
        .info(Color::Green)
        .debug(Color::Blue)
        .warn(Color::Yellow)
        .error(Color::Red);

    fern::Dispatch::new()
        .level(logging_level)
        .chain(console_logger(colors_level, colors_line))
        .chain(file_logger(sender))
        .apply()
        .unwrap();

    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        log::error!("{:?}", panic_info.to_string());
        hook(panic_info);
    }));
}