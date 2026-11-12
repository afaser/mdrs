#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Generation(u64);

impl Generation {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn advance(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }
}
