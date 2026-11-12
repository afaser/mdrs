use log::{Level, Log, Metadata, Record};

const LEVEL: Level = Level::Info;

struct Stderr;

impl Log for Stderr {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= LEVEL
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            eprintln!("[{}] {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: Stderr = Stderr;

pub(super) fn init() {
    if std::env::var_os("MDRS_LOG").is_none() {
        return;
    }

    if log::set_logger(&LOGGER).is_ok() {
        log::set_max_level(LEVEL.to_level_filter());
    }
}
