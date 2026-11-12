use std::time::Instant;

use iced::mouse::ScrollDelta;
use iced::widget::scrollable::{AbsoluteOffset, Viewport};

use crate::app::motion::Motion;
use crate::config::Wheel;

#[derive(Debug)]
pub struct Scroll {
    offset: AbsoluteOffset,
    limit: f32,
    motion: Option<Motion>,
    wheel: Wheel,
}

impl Default for Scroll {
    fn default() -> Self {
        Self {
            offset: AbsoluteOffset::default(),
            limit: 0.0,
            motion: None,
            wheel: Wheel::default(),
        }
    }
}

impl Scroll {
    pub fn new(wheel: Wheel) -> Self {
        Self {
            wheel,
            ..Self::default()
        }
    }

    pub fn offset(&self) -> AbsoluteOffset {
        self.offset
    }

    pub fn moving(&self) -> bool {
        self.motion.is_some()
    }

    pub fn reset(&mut self) {
        self.offset = AbsoluteOffset::default();
        self.limit = 0.0;
        self.motion = None;
    }

    pub fn stop(&mut self) {
        if let Some(motion) = self.motion.take() {
            self.offset.y = motion.issued();
        }
    }

    pub fn remember(&mut self, viewport: Viewport) {
        self.remember_offset(viewport.absolute_offset(), vertical_limit(viewport));
    }

    pub fn wheel(&mut self, delta: ScrollDelta, now: Instant) {
        let distance = wheel_distance(delta, self.wheel);

        if is_zero(distance) {
            return;
        }

        let target = match self.motion {
            Some(motion) => self.clamp(motion.target() + distance),
            None => self.clamp(self.offset.y + distance),
        };
        let motion = match self.motion.take() {
            Some(motion) => motion.retarget(target),
            None => Motion::start(self.offset.y, target, now, self.wheel),
        };

        if motion.settled() {
            self.offset.y = motion.position();
            self.motion = None;
            return;
        }

        self.motion = Some(motion);
    }

    pub fn frame(&mut self, now: Instant) -> Option<AbsoluteOffset> {
        let mut motion = self.motion.take()?;

        self.offset.y = motion.advance(now);

        if motion.settled() {
            return Some(self.offset);
        }

        self.motion = Some(motion);
        Some(self.offset)
    }

    fn remember_offset(&mut self, offset: AbsoluteOffset, limit: f32) {
        self.limit = limit;

        if let Some(motion) = self.motion
            && (offset.y - motion.issued()).abs() > self.wheel.external_distance
        {
            self.motion = None;
        }

        self.offset = AbsoluteOffset {
            x: offset.x,
            y: offset.y,
        };
    }

    fn clamp(&self, value: f32) -> f32 {
        value.clamp(0.0, self.limit)
    }
}

fn wheel_distance(delta: ScrollDelta, wheel: Wheel) -> f32 {
    let notches = match delta {
        ScrollDelta::Lines { y, .. } => y,
        ScrollDelta::Pixels { y, .. } => y / wheel.pixels_per_notch,
    };

    -notches * wheel.lines_per_notch * wheel.line_height
}

fn vertical_limit(viewport: Viewport) -> f32 {
    let bounds = viewport.bounds();
    let content = viewport.content_bounds();

    (content.height - bounds.height).max(0.0)
}

fn is_zero(value: f32) -> bool {
    value.abs() <= f32::EPSILON
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn ready(limit: f32) -> Scroll {
        let mut scroll = Scroll::new(Wheel::default());
        scroll.remember_offset(AbsoluteOffset { x: 0.0, y: 0.0 }, limit);
        scroll
    }

    #[test]
    fn discrete_notch_moves_eight_lines() {
        let mut scroll = ready(1000.0);
        let start = Instant::now();

        scroll.wheel(ScrollDelta::Lines { x: 0.0, y: -1.0 }, start);
        let offset = scroll
            .frame(start + Duration::from_millis(250))
            .expect("frame");

        assert_eq!(offset.y, 192.0);
        assert!(!scroll.moving());
    }

    #[test]
    fn configured_wheel_changes_step() {
        let wheel = Wheel {
            lines_per_notch: 4.0,
            line_height: 10.0,
            ..Wheel::default()
        };
        let mut scroll = Scroll::new(wheel);
        scroll.remember_offset(AbsoluteOffset { x: 0.0, y: 0.0 }, 1000.0);
        let start = Instant::now();

        scroll.wheel(ScrollDelta::Lines { x: 0.0, y: -1.0 }, start);
        let offset = scroll
            .frame(start + Duration::from_millis(250))
            .expect("frame");

        assert_eq!(offset.y, 40.0);
    }

    #[test]
    fn zero_smoothing_moves_to_target() {
        let wheel = Wheel {
            smoothing: 0.0,
            ..Wheel::default()
        };
        let mut scroll = Scroll::new(wheel);
        scroll.remember_offset(AbsoluteOffset { x: 0.0, y: 0.0 }, 1000.0);
        let start = Instant::now();

        scroll.wheel(ScrollDelta::Lines { x: 0.0, y: -1.0 }, start);
        let offset = scroll
            .frame(start + Duration::from_millis(16))
            .expect("frame");

        assert_eq!(offset.y, 192.0);
        assert!(!scroll.moving());
    }

    #[test]
    fn pixel_notch_matches_discrete_step() {
        let mut scroll = ready(1000.0);
        let start = Instant::now();

        scroll.wheel(ScrollDelta::Pixels { x: 0.0, y: -15.0 }, start);
        let offset = scroll
            .frame(start + Duration::from_millis(250))
            .expect("frame");

        assert_eq!(offset.y, 192.0);
    }

    #[test]
    fn wheel_up_moves_toward_start() {
        let mut scroll = ready(1000.0);
        scroll.remember_offset(AbsoluteOffset { x: 0.0, y: 500.0 }, 1000.0);
        let start = Instant::now();

        scroll.wheel(ScrollDelta::Lines { x: 0.0, y: 1.0 }, start);
        let offset = scroll
            .frame(start + Duration::from_millis(250))
            .expect("frame");

        assert_eq!(offset.y, 308.0);
    }

    #[test]
    fn wheel_at_bounds_creates_no_motion() {
        let mut scroll = ready(0.0);
        scroll.wheel(ScrollDelta::Lines { x: 0.0, y: -1.0 }, Instant::now());
        assert!(!scroll.moving());
        assert_eq!(scroll.offset().y, 0.0);

        scroll.remember_offset(AbsoluteOffset { x: 0.0, y: 192.0 }, 192.0);
        scroll.wheel(ScrollDelta::Lines { x: 0.0, y: -1.0 }, Instant::now());
        assert!(!scroll.moving());
        assert_eq!(scroll.offset().y, 192.0);
    }

    #[test]
    fn repeated_notches_accumulate_target() {
        let mut scroll = ready(1000.0);
        let start = Instant::now();

        scroll.wheel(ScrollDelta::Lines { x: 0.0, y: -1.0 }, start);
        scroll.wheel(ScrollDelta::Lines { x: 0.0, y: -1.0 }, start);
        let offset = scroll
            .frame(start + Duration::from_millis(250))
            .expect("frame");

        assert_eq!(offset.y, 384.0);
    }

    #[test]
    fn animation_settles_within_expected_time() {
        let mut scroll = ready(1000.0);
        let start = Instant::now();
        scroll.wheel(ScrollDelta::Lines { x: 0.0, y: -1.0 }, start);

        let mut now = start;
        let mut elapsed = Duration::ZERO;
        let mut previous = scroll.offset().y;

        while scroll.moving() {
            now += Duration::from_millis(16);
            elapsed += Duration::from_millis(16);
            let offset = scroll.frame(now).expect("frame");
            assert!(offset.y >= previous);
            previous = offset.y;
        }

        assert_eq!(previous, 192.0);
        assert!(elapsed >= Duration::from_millis(100));
        assert!(elapsed <= Duration::from_millis(200));
    }

    #[test]
    fn external_viewport_cancels_motion_and_matching_keeps_it() {
        let mut scroll = ready(1000.0);
        scroll.wheel(ScrollDelta::Lines { x: 0.0, y: -1.0 }, Instant::now());
        scroll.frame(Instant::now() + Duration::from_millis(16));
        assert!(scroll.moving());

        scroll.remember_offset(AbsoluteOffset { x: 0.0, y: 500.0 }, 1000.0);

        assert!(!scroll.moving());
        assert_eq!(scroll.offset().y, 500.0);

        let mut scroll = ready(1000.0);
        scroll.wheel(ScrollDelta::Lines { x: 0.0, y: -1.0 }, Instant::now());
        let issued = scroll
            .frame(Instant::now() + Duration::from_millis(32))
            .expect("frame");

        scroll.remember_offset(issued, 1000.0);

        assert!(scroll.moving());
    }

    #[test]
    fn stop_keeps_issued_position() {
        let mut scroll = ready(1000.0);
        scroll.wheel(ScrollDelta::Lines { x: 0.0, y: -1.0 }, Instant::now());
        let issued = scroll
            .frame(Instant::now() + Duration::from_millis(32))
            .expect("frame");

        scroll.stop();

        assert!(!scroll.moving());
        assert_eq!(scroll.offset(), issued);
    }
}
