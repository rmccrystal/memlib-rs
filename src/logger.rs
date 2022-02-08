use log::{Metadata, Record};

pub struct Logger<T>(T);

impl<T: crate::System + Send + Sync> Logger<T> {
    pub fn new(system: T) -> Self {
        Self(system)
    }
}

impl<T> log::Log for Logger<T>
    where
        T: crate::System + Send + Sync
{
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let message = format!(
                "{:<5} [{}] {}\n\0",
                record.level().to_string().to_uppercase(),
                record.target(),
                record.args()
            );
            self.0.log(&message);
        }
    }

    fn flush(&self) {}
}
