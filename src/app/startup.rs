use std::time::Instant;

pub struct Startup {
    started: Instant,
    enabled: bool,
}

impl Startup {
    pub fn new(started: Instant) -> Self {
        Self {
            started,
            enabled: std::env::var_os("MDRS_STARTUP")
                .is_some_and(|value| value.to_str() == Some("1")),
        }
    }

    pub fn mark(&self, label: &str) {
        if !self.enabled {
            return;
        }

        eprintln!("{label}: {:.2} ms", self.elapsed_ms());
    }

    pub fn finish(self) {
        if !self.enabled {
            return;
        }

        eprintln!("first-frame: {:.2} ms", self.elapsed_ms());
        std::process::exit(0);
    }

    fn elapsed_ms(&self) -> f64 {
        self.started.elapsed().as_secs_f64() * 1000.0
    }
}
