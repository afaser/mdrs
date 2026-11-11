use std::time::Instant;

use iced::mouse::ScrollDelta;
use iced::widget::scrollable::{AbsoluteOffset, Viewport};

use crate::app::view::TEXT_LINE_HEIGHT;

const LINES_PER_NOTCH: f32 = 8.0;
const PIXELS_PER_NOTCH: f32 = 15.0;
const SMOOTHING_SECONDS: f32 = 0.03;
const SETTLE_DISTANCE: f32 = 0.5;
const EXTERNAL_DISTANCE: f32 = 0.5;

#[derive(Debug)]
pub struct Scroll {
    offset: AbsoluteOffset,
    limit: f32,
    motion: Option<Motion>,
}

#[derive(Debug, Clone, Copy)]
struct Motion {
    position: f32,
    target: f32,
    issued: f32,
    last_frame: Instant,
}

impl Scroll {
    pub fn new() -> Self {
        Self {
            offset: AbsoluteOffset::default(),
            limit: 0.0,
            motion: None,
        }
    }

    pub fn offset(&self) -> AbsoluteOffset {
        self.offset
    }

    pub fn moving(&self) -> bool {
        self.motion.is_some()
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn stop(&mut self) {
        if let Some(motion) = self.motion.take() {
            self.offset.y = motion.issued;
        }
    }

    pub fn remember(&mut self, viewport: Viewport) {
        self.remember_offset(viewport.absolute_offset(), vertical_limit(viewport));
    }

    pub fn wheel(&mut self, delta: ScrollDelta) {
        let distance = wheel_distance(delta);

        if distance == 0.0 {
            return;
        }

        let motion = match self.motion {
            Some(motion) => Motion {
                target: self.clamp(motion.target + distance),
                ..motion
            },
            None => Motion {
                position: self.offset.y,
                target: self.clamp(self.offset.y + distance),
                issued: self.offset.y,
                last_frame: Instant::now(),
            },
        };

        if (motion.target - motion.position).abs() <= SETTLE_DISTANCE {
            self.offset.y = motion.target;
            self.motion = None;
            return;
        }

        self.motion = Some(motion);
    }

    pub fn frame(&mut self, now: Instant) -> Option<AbsoluteOffset> {
        let motion = self.motion?;
        let elapsed = now
            .saturating_duration_since(motion.last_frame)
            .as_secs_f32();
        let factor = 1.0 - (-elapsed / SMOOTHING_SECONDS).exp();
        let remaining = motion.target - motion.position;
        let next = motion.position + remaining * factor;
        let reached = (motion.target - next).abs() <= SETTLE_DISTANCE;
        let position = if reached { motion.target } else { next };

        self.offset.y = position;

        if reached {
            self.motion = None;
        } else {
            self.motion = Some(Motion {
                position,
                issued: position,
                last_frame: now,
                ..motion
            });
        }

        Some(AbsoluteOffset {
            x: self.offset.x,
            y: position,
        })
    }

    fn remember_offset(&mut self, offset: AbsoluteOffset, limit: f32) {
        self.limit = limit;

        if let Some(motion) = self.motion
            && (offset.y - motion.issued).abs() > EXTERNAL_DISTANCE
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

fn wheel_distance(delta: ScrollDelta) -> f32 {
    let notches = match delta {
        ScrollDelta::Lines { y, .. } => y,
        ScrollDelta::Pixels { y, .. } => y / PIXELS_PER_NOTCH,
    };

    -notches * LINES_PER_NOTCH * TEXT_LINE_HEIGHT
}

fn vertical_limit(viewport: Viewport) -> f32 {
    let bounds = viewport.bounds();
    let content = viewport.content_bounds();

    (content.height - bounds.height).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn ready(limit: f32) -> Scroll {
        let mut scroll = Scroll::new();
        scroll.remember_offset(AbsoluteOffset { x: 0.0, y: 0.0 }, limit);
        scroll
    }

    #[test]
    fn discrete_notch_moves_eight_lines() {
        let mut scroll = ready(1000.0);
        let start = Instant::now();

        scroll.wheel(ScrollDelta::Lines { x: 0.0, y: -1.0 });
        let offset = scroll
            .frame(start + Duration::from_millis(250))
            .expect("frame");

        assert_eq!(offset.y, 192.0);
        assert!(!scroll.moving());
    }

    #[test]
    fn pixel_notch_matches_discrete_step() {
        let mut scroll = ready(1000.0);
        let start = Instant::now();

        scroll.wheel(ScrollDelta::Pixels { x: 0.0, y: -15.0 });
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

        scroll.wheel(ScrollDelta::Lines { x: 0.0, y: 1.0 });
        let offset = scroll
            .frame(start + Duration::from_millis(250))
            .expect("frame");

        assert_eq!(offset.y, 308.0);
    }

    #[test]
    fn wheel_at_bounds_creates_no_motion() {
        let mut scroll = ready(0.0);
        scroll.wheel(ScrollDelta::Lines { x: 0.0, y: -1.0 });
        assert!(!scroll.moving());
        assert_eq!(scroll.offset().y, 0.0);

        scroll.remember_offset(AbsoluteOffset { x: 0.0, y: 192.0 }, 192.0);
        scroll.wheel(ScrollDelta::Lines { x: 0.0, y: -1.0 });
        assert!(!scroll.moving());
        assert_eq!(scroll.offset().y, 192.0);
    }

    #[test]
    fn repeated_notches_accumulate_target() {
        let mut scroll = ready(1000.0);
        let start = Instant::now();

        scroll.wheel(ScrollDelta::Lines { x: 0.0, y: -1.0 });
        scroll.wheel(ScrollDelta::Lines { x: 0.0, y: -1.0 });
        let offset = scroll
            .frame(start + Duration::from_millis(250))
            .expect("frame");

        assert_eq!(offset.y, 384.0);
    }

    #[test]
    fn animation_settles_within_expected_time() {
        let mut scroll = ready(1000.0);
        let start = Instant::now();
        scroll.wheel(ScrollDelta::Lines { x: 0.0, y: -1.0 });

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
    fn external_viewport_cancels_motion() {
        let mut scroll = ready(1000.0);
        scroll.wheel(ScrollDelta::Lines { x: 0.0, y: -1.0 });
        scroll.frame(Instant::now() + Duration::from_millis(16));
        assert!(scroll.moving());

        scroll.remember_offset(AbsoluteOffset { x: 0.0, y: 500.0 }, 1000.0);

        assert!(!scroll.moving());
        assert_eq!(scroll.offset().y, 500.0);
    }

    #[test]
    fn matching_viewport_keeps_motion() {
        let mut scroll = ready(1000.0);
        scroll.wheel(ScrollDelta::Lines { x: 0.0, y: -1.0 });
        let issued = scroll
            .frame(Instant::now() + Duration::from_millis(32))
            .expect("frame");

        scroll.remember_offset(issued, 1000.0);

        assert!(scroll.moving());
    }

    #[test]
    fn stop_keeps_issued_position() {
        let mut scroll = ready(1000.0);
        scroll.wheel(ScrollDelta::Lines { x: 0.0, y: -1.0 });
        let issued = scroll
            .frame(Instant::now() + Duration::from_millis(32))
            .expect("frame");

        scroll.stop();

        assert!(!scroll.moving());
        assert_eq!(scroll.offset(), issued);
    }
}
