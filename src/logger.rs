use log::{Metadata, Record, LevelFilter, SetLoggerError, set_max_level, set_boxed_logger, Level};
use colored::Colorize;

/// Implements a minimal logger that I like to use for my cheats
pub struct MinimalLogger {
    pub level: LevelFilter,
}

impl MinimalLogger {
    pub fn init(level: LevelFilter) -> Result<(), SetLoggerError> {
        set_max_level(level);
        set_boxed_logger(Box::new(Self { level }))
    }
}

impl log::Log for MinimalLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(&record.metadata()) {
            return;
        }

        let prefix = match record.level() {
            Level::Error => "[ERROR]".red(),
            Level::Warn => "[!]".yellow(),
            Level::Info => "[+]".green(),
            Level::Debug => "[*]".blue(),
            Level::Trace => "[?]".magenta(),
        };

        println!("{} {}", prefix, record.args());
    }

    fn flush(&self) {}
}