use log::{Metadata, Record, LevelFilter, SetLoggerError, set_max_level, set_boxed_logger, Level};

/// Implements a minimal logger that I like to use for my cheats
pub struct MinimalLogger {
    pub level: LevelFilter,
}

impl MinimalLogger {
    pub fn init(level: LevelFilter) -> Result<(), SetLoggerError> {
        set_max_level(level.clone());
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
            Level::Error => "[ERROR]",
            Level::Warn => "[!]",
            Level::Info => "[+]",
            Level::Debug => "[*]",
            Level::Trace => "[?]",
        };

        println!("{} {}", prefix, record.args());
    }

    fn flush(&self) {}
}