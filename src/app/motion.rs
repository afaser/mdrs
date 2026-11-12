use std::time::Instant;

use crate::config::Wheel;

#[derive(Debug, Clone, Copy)]
pub(super) struct Motion {
    position: f32,
    target: f32,
    issued: f32,
    last_frame: Instant,
    smoothing: f32,
    settle_distance: f32,
}

impl Motion {
    pub(super) fn start(position: f32, target: f32, now: Instant, wheel: Wheel) -> Self {
        Self {
            position,
            target,
            issued: position,
            last_frame: now,
            smoothing: wheel.smoothing,
            settle_distance: wheel.settle_distance,
        }
    }

    pub(super) fn retarget(mut self, target: f32) -> Self {
        self.target = target;
        self
    }

    pub(super) fn advance(&mut self, now: Instant) -> f32 {
        let elapsed = now.saturating_duration_since(self.last_frame).as_secs_f32();
        let factor = smoothing_factor(elapsed, self.smoothing);
        let remaining = self.target - self.position;

        self.position += remaining * factor;
        self.issued = self.position;
        self.last_frame = now;
        self.snap_if_settled();

        self.position
    }

    pub(super) fn settled(&self) -> bool {
        (self.target - self.position).abs() <= self.settle_distance
    }

    pub(super) fn target(&self) -> f32 {
        self.target
    }

    pub(super) fn position(&self) -> f32 {
        self.position
    }

    pub(super) fn issued(&self) -> f32 {
        self.issued
    }

    fn snap_if_settled(&mut self) {
        if !self.settled() {
            return;
        }

        self.position = self.target;
        self.issued = self.target;
    }
}

fn smoothing_factor(elapsed: f32, smoothing: f32) -> f32 {
    if smoothing <= f32::EPSILON {
        return 1.0;
    }

    1.0 - (-elapsed / smoothing).exp()
}
