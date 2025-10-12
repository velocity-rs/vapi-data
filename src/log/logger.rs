use fern::Dispatch;
use fern::colors::{Color, ColoredLevelConfig};
use log::info;
use std::collections::HashMap;
use std::str::FromStr;

fn initialize(log_levels: HashMap<String, String>) -> Dispatch {
    let colors_line = ColoredLevelConfig::new()
        .error(Color::BrightRed)
        .warn(Color::BrightCyan)
        // we actually don't need to specify the color for debug and info, they are white by default
        .info(Color::BrightBlue)
        .debug(Color::BrightYellow)
        // depending on the terminals color scheme, this is the same as the background color
        .trace(Color::Magenta);

    let mut dispatch: Dispatch = Dispatch::new().format(move |out, message, record| {
        out.finish(format_args!(
            "{color_line}[{time}][{level}][{final_file_path} ><{line_num}] -> {message}",
            color_line = format_args!(
                "\x1B[{}m",
                colors_line.get_color(&record.level()).to_fg_str()
            ),
            time = chrono::Local::now().format("%d-%m-%y %H:%M:%S%.3f %Z"),
            level = format!("{:width$}", record.level(), width = 5),
            final_file_path = {
                let mut file_path = String::from(record.file().unwrap());
                let path_len = file_path.len();
                let truncated_file_path = match path_len > 28 {
                    true => [String::from(".."), file_path.split_off(path_len - 26)].concat(),
                    false => file_path,
                };

                format!("{:width$}", truncated_file_path, width = 28)
            },
            line_num = format!("{:width$}", record.line().unwrap(), width = 4),
            message = message
        ));
    });

    dispatch = dispatch.level(log::LevelFilter::Error);

    for (key, value) in log_levels {
        match log::LevelFilter::from_str(&value) {
            Ok(level) => {
                dispatch = dispatch.level_for(key, level);
            }
            Err(e) => {
                println!(
                    "Error parsing level for {}. Given value {} is invalid. Error {:?}",
                    key, value, e
                );
            }
        }
    }

    dispatch.chain(std::io::stdout())
}

pub fn setup(log_levels: HashMap<String, String>) {
    match initialize(log_levels).apply() {
        Ok(()) => log::info!("Logger setup done"),
        Err(e) => log::error!(
            "Error setting up logger. Logger will function with default configs {}",
            e
        ),
    }
}

pub fn set_log_level(module: String, level: String) {
    //FIXME this is not working
    info!("Setting log level for {} to {}", module, level);
    match log::LevelFilter::from_str(&level) {
        Ok(level_filter) => {
            let config = fern::Dispatch::new().level_for(module, level_filter);
            match fern::Dispatch::chain(config, std::io::stdout()).apply() {
                Ok(()) => log::info!("Logger setup done"),
                Err(e) => log::error!("Error setting up logger: {}", e),
            }
        }
        Err(e) => {
            println!(
                "Error parsing level for {}. Given value {} is invalid. Error {:?}",
                module, level, e
            );
        }
    }
}
