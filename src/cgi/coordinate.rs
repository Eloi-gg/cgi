use std::ops::{Add, AddAssign, Sub, SubAssign, Neg};

#[derive(Debug, Copy, Clone)]
pub enum Coordinate {
    Absolute(i32),
    Relative(f32),
    Hybrid(i32, f32),
}

impl Coordinate {
    pub fn is_null(&self) -> bool {
        match self {
            Coordinate::Absolute(a) => *a == 0,
            Coordinate::Relative(r) => *r == 0.0,
            Coordinate::Hybrid(a, r) => *a == 0 && *r == 0.0,
        }
    }

    pub(crate) fn to_hybrid(&self) -> Self {
        use Coordinate::*;

        match self {
            Absolute(a) => Hybrid(*a, 0.0),
            Relative(r) => Hybrid(0, *r),
            Hybrid(a, r) => Hybrid(*a, *r),
        }
    }

    pub(crate) fn absolute_part(&self) -> Self {
        use Coordinate::*;

        let a = match self {
            Absolute(a) => *a,
            Relative(_) => 0,
            Hybrid(a, _) => *a,
        };

        Absolute(a)
    }

    pub(crate) fn absolute_part_i32(&self) -> i32 {
        let Self::Absolute(x) = self.absolute_part() else {
            return 0;
        };
        x
    }

    pub(crate) fn compute_at(&self, size: i32) -> i32 {
        use Coordinate::*;

        match self {
            Absolute(a) => (*a).max(0), // clamp to 0
            Relative(r) => (size as f32 * *r) as i32,
            Hybrid(a, r) => (*a + (size as f32 * *r) as i32).max(0), // clamp to 0
        }
    }

    fn relative_part(&self) -> Self {
        use Coordinate::*;

        match self {
            Relative(r) => Relative(*r),
            Hybrid(_, r) => Relative(*r),
            _ => Absolute(0),
        }
    }
}

impl Add for Coordinate {
    type Output = Coordinate;

    fn add(self, rhs: Self) -> Self::Output {
        use Coordinate::*;

        if self.is_null() {
            return rhs;
        }
        if rhs.is_null() {
            return self;
        }
        match (self, rhs) {
            (Absolute(a1), Absolute(a2)) => Absolute(a1 + a2),
            (Relative(r1), Relative(r2)) => Relative(r1 + r2),
            (Hybrid(a1, r1), Hybrid(a2, r2)) => Hybrid(a1 + a2, r1 + r2),
            (Absolute(a), Relative(r)) | (Relative(r), Absolute(a)) => Hybrid(a, r),
            (Absolute(a), Hybrid(b1, b2)) | (Hybrid(b1, b2), Absolute(a)) => Hybrid(a + b1, b2),
            (Relative(a), Hybrid(b1, b2)) | (Hybrid(b1, b2), Relative(a)) => Hybrid(b1, a + b2),
        }
    }
}

impl AddAssign for Coordinate {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Sub for Coordinate {
    type Output = Coordinate;

    fn sub(self, rhs: Self) -> Self::Output {
        use Coordinate::*;

        if rhs.is_null() {
            return self;
        }
        match (self, rhs) {
            (Absolute(a1), Absolute(a2)) => Absolute(a1 - a2),
            (Relative(r1), Relative(r2)) => Relative(r1 - r2),
            (Hybrid(a1, r1), Hybrid(a2, r2)) => Hybrid(a1 - a2, r1 - r2),
            (Absolute(a), Relative(r)) => Hybrid(a, -r),
            (Relative(r), Absolute(a)) => Hybrid(-a, r),
            (Absolute(a), Hybrid(a2, r2)) => Hybrid(a - a2, -r2),
            (Hybrid(a1, r1), Absolute(a2)) => Hybrid(a1 - a2, r1),
            (Relative(r1), Hybrid(a2, r2)) => Hybrid(-a2, r1 - r2),
            (Hybrid(a1, r1), Relative(r2)) => Hybrid(a1, r1 - r2),
        }
    }
}

impl SubAssign for Coordinate {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl Neg for Coordinate {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::Absolute(0) - self
    }
}

impl std::ops::Div<f32> for Coordinate {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        use Coordinate::*;

        match self {
            Absolute(a) => Absolute((a as f32 / rhs) as i32),
            Relative(r) => Relative(r / rhs),
            Hybrid(a, r) => Hybrid((a as f32 / rhs) as i32, r / rhs),
        }
    }
}

impl std::ops::Mul<f32> for Coordinate {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        use Coordinate::*;

        match self {
            Absolute(a) => Absolute((a as f32 * rhs) as i32),
            Relative(r) => Relative(r * rhs),
            Hybrid(a, r) => Hybrid((a as f32 * rhs) as i32, r * rhs),
        }
    }
}

impl PartialEq for Coordinate {
    fn eq(&self, other: &Self) -> bool {
        use Coordinate::*;
        // Convert to Hybrid for comparison
        let self_hybrid = self.to_hybrid();
        let other_hybrid = other.to_hybrid();
        match (self_hybrid, other_hybrid) {
            (Absolute(a1), Absolute(a2)) => a1 == a2,
            (Relative(r1), Relative(r2)) => r1 == r2,
            (Hybrid(a1, r1), Hybrid(a2, r2)) => a1 == a2 && r1 == r2,

            (Absolute(0), Relative(0.0)) | (Relative(0.0), Absolute(0)) => true,
            (Absolute(0), Hybrid(0, 0.0)) | (Hybrid(0, 0.0), Absolute(0)) => true,

            (Absolute(a), Hybrid(a2, 0.0)) | (Hybrid(a2, 0.0), Absolute(a)) => a == a2,
            (Relative(r), Hybrid(0, r2)) | (Hybrid(0, r2), Relative(r)) => r == r2,

            _ => false,
        }
    }
}
impl Eq for Coordinate {}

impl From<i32> for Coordinate {
    fn from(value: i32) -> Self {
        Coordinate::Absolute(value)
    }
}

impl From<f32> for Coordinate {
    fn from(value: f32) -> Self {
        Coordinate::Relative(value)
    }
}

impl From<(i32, f32)> for Coordinate {
    fn from(value: (i32, f32)) -> Self {
        Coordinate::Hybrid(value.0, value.1)
    }
}
