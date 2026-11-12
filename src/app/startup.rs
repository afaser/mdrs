use std::time::Instant;

pub(super) struct Startup {
    started: Instant,
}

impl Startup {
    pub(super) fn new(started: Instant) -> Option<Self> {
        if !measurement_requested() {
            return None;
        }

        Some(Self { started })
    }

    pub(super) fn mark(&self, label: &str) {
        eprintln!("{label}: {:.2} ms", self.elapsed_ms());
    }

    pub(super) fn finish(self) -> f32 {
        let elapsed = self.elapsed_ms();
        eprintln!("first-frame: {elapsed:.2} ms");
        elapsed
    }

    fn elapsed_ms(&self) -> f32 {
        self.started.elapsed().as_secs_f32() * 1000.0
    }
}

fn measurement_requested() -> bool {
    std::env::var_os("MDRS_STARTUP").is_some_and(|value| value.to_str() == Some("1"))
}
