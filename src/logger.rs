use std::fs;
use chrono::{Timelike, Datelike};

pub fn setup_logger() -> Result<(), fern::InitError> {

    fs::create_dir_all("logs/").unwrap();

    let log_file = format!("logs/start-{}-{}-{}.log", 
        chrono::offset::Local::now().year(), chrono::offset::Local::now().month(), chrono::offset::Local::now().day());
    
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}:{}:{} {} {}] {}",
                chrono::offset::Local::now().hour(),
                chrono::offset::Local::now().minute(),
                chrono::offset::Local::now().second(),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Off)
        .level_for("memer_rs", log::LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(fern::log_file(log_file)?)
        .apply()?;
    Ok(())
}