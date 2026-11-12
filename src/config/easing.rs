#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    CubicBezier { x1: f32, y1: f32, x2: f32, y2: f32 },
}

impl Easing {
    pub fn from_parts(x1: f32, y1: f32, x2: f32, y2: f32) -> Option<Self> {
        if !(0.0..=1.0).contains(&x1) || !(0.0..=1.0).contains(&x2) {
            return None;
        }

        Some(Self::CubicBezier { x1, y1, x2, y2 })
    }

    pub fn parse(value: &str) -> Option<Self> {
        if let Some(curve) = parse_curve(value) {
            return Some(curve);
        }

        match value {
            "linear" => Some(Self::Linear),
            "ease-in" => Some(Self::EaseIn),
            "ease-out" => Some(Self::EaseOut),
            "ease-in-out" => Some(Self::EaseInOut),
            _ => None,
        }
    }

    pub fn apply(self, value: f32) -> f32 {
        let t = value.clamp(0.0, 1.0);

        match self {
            Self::Linear => t,
            Self::EaseIn => t * t * t,
            Self::EaseOut => {
                let inverse = 1.0 - t;
                1.0 - inverse * inverse * inverse
            }
            Self::EaseInOut => {
                if t < 0.5 {
                    return 4.0 * t * t * t;
                }

                let inverse = -2.0 * t + 2.0;
                1.0 - inverse * inverse * inverse / 2.0
            }
            Self::CubicBezier { x1, y1, x2, y2 } => {
                let parameter = solve_parameter(t, x1, x2);
                bezier(parameter, y1, y2)
            }
        }
    }
}

fn parse_curve(value: &str) -> Option<Easing> {
    let numbers: Vec<f32> = value
        .split(|character: char| character.is_whitespace() || character == ',')
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse().ok())
        .collect();

    if numbers.len() != 4 {
        return None;
    }

    Easing::from_parts(numbers[0], numbers[1], numbers[2], numbers[3])
}

fn bezier(parameter: f32, first: f32, second: f32) -> f32 {
    let inverse = 1.0 - parameter;
    3.0 * inverse * inverse * parameter * first
        + 3.0 * inverse * parameter * parameter * second
        + parameter * parameter * parameter
}

fn derivative(parameter: f32, first: f32, second: f32) -> f32 {
    let inverse = 1.0 - parameter;
    3.0 * inverse * inverse * first
        + 6.0 * inverse * parameter * (second - first)
        + 3.0 * parameter * parameter * (1.0 - second)
}

fn solve_parameter(x: f32, x1: f32, x2: f32) -> f32 {
    let mut parameter = x;

    for _ in 0..8 {
        let difference = bezier(parameter, x1, x2) - x;
        let slope = derivative(parameter, x1, x2);

        if slope.abs() <= f32::EPSILON {
            break;
        }

        parameter = (parameter - difference / slope).clamp(0.0, 1.0);
    }

    parameter
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn easings_map_endpoints() {
        for easing in [
            Easing::Linear,
            Easing::EaseIn,
            Easing::EaseOut,
            Easing::EaseInOut,
            Easing::CubicBezier {
                x1: 0.25,
                y1: 0.1,
                x2: 0.25,
                y2: 1.0,
            },
        ] {
            assert_eq!(easing.apply(0.0), 0.0);
            assert_eq!(easing.apply(1.0), 1.0);
        }
    }

    #[test]
    fn easing_names_are_parsed() {
        assert_eq!(Easing::parse("linear"), Some(Easing::Linear));
        assert_eq!(Easing::parse("ease-out"), Some(Easing::EaseOut));
        assert_eq!(Easing::parse("bounce"), None);
    }

    #[test]
    fn cubic_curve_is_parsed_from_four_numbers() {
        assert_eq!(
            Easing::parse("0.25 0.1 0.25 1"),
            Some(Easing::CubicBezier {
                x1: 0.25,
                y1: 0.1,
                x2: 0.25,
                y2: 1.0,
            })
        );
        assert_eq!(
            Easing::parse("0.42, 0, 0.58, 1"),
            Some(Easing::CubicBezier {
                x1: 0.42,
                y1: 0.0,
                x2: 0.58,
                y2: 1.0,
            })
        );
    }

    #[test]
    fn cubic_curve_is_monotonic_for_x() {
        let easing = Easing::CubicBezier {
            x1: 0.25,
            y1: 0.1,
            x2: 0.25,
            y2: 1.0,
        };
        let mut previous = 0.0;

        for step in 0..=20 {
            let value = easing.apply(step as f32 / 20.0);
            assert!(value >= previous - f32::EPSILON);
            previous = value;
        }
    }
}
