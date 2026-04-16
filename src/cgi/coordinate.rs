use std::ops::{Add, AddAssign, Sub, SubAssign, Neg};

#[derive(Debug, Copy, Clone)]
pub enum Coordinate {
    Absolute(i32),
    Relative(f32),
    Hybrid(i32, f32),
    Adaptative(i32),
}

pub(crate) struct ComputedCoordinate {
    x: i32
}

impl Coordinate {
    pub fn is_null(&self) -> bool {
        match self {
            Coordinate::Absolute(a) => *a == 0,
            Coordinate::Relative(r) => *r == 0.0,
            Coordinate::Hybrid(a, r) => *a == 0 && *r == 0.0,
            Coordinate::Adaptative(_) => false,
        }
    }

    pub(crate) fn compute_adaptative_sizes(coords: &[Self], with_absolute_offset: i32) -> Self {
        // TODO: with_absolute_offset useless
        let mut space_to_occupy = Self::Hybrid(0, 1.0); // full size
        let mut divider = 0;
        for coord in coords {
            if let Coordinate::Adaptative(offset) = coord {
                divider += 1;
                space_to_occupy = space_to_occupy + Self::Absolute(*offset);
            } else {
                space_to_occupy = space_to_occupy - *coord;
            }
        }
        if divider == 0 {
            return Self::Hybrid(0, 0.0);
        }

        if let Coordinate::Hybrid(a, r) = space_to_occupy {
            let relative = r / divider as f32;
            let absolute = a / divider as i32;
            return Self::Hybrid(absolute, relative);
        } else {
            unreachable!()
        }
    }

    pub(crate) fn to_hybrid(&self) -> Self {
        use Coordinate::*;

        match self {
            Absolute(a) => Hybrid(*a, 0.0),
            Relative(r) => Hybrid(0, *r),
            Hybrid(a, r) => Hybrid(*a, *r),
            Adaptative(o) => Hybrid(*o, 1.0),
        }
    }

    pub(crate) fn absolute_part(&self) -> Self {
        use Coordinate::*;

        let a = match self {
            Absolute(a) => *a,
            Relative(_) => 0,
            Hybrid(a, _) => *a,
            Adaptative(o) => *o,
        };

        Absolute(a)
    }

    pub(crate) fn absolute_part_i32(&self) -> i32 {
        let Self::Absolute(x) = self.absolute_part() else {
            return 0;
        };
        x
    }

    pub(crate) fn without_adaptative_offset(&self) -> Self {
        if let Self::Adaptative(_) = self {
            Self::Adaptative(0)
        } else {
            *self
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
            (Adaptative(o1), Adaptative(o2)) => Adaptative(o1 + o2),
            (Adaptative(o), Absolute(a)) | (Absolute(a), Adaptative(o)) => Adaptative(o + a),
            (Adaptative(o), Relative(_)) | (Relative(_), Adaptative(o)) => Adaptative(o),
            (Adaptative(o), Hybrid(a, _)) | (Hybrid(a, _), Adaptative(o)) => Adaptative(o + a),
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
            (Adaptative(o1), Adaptative(o2)) => Adaptative(o1 - o2),

            (Adaptative(o), Absolute(a)) => Adaptative(o - a),
            (Absolute(a), Adaptative(o)) => Adaptative(a - o),
            (Adaptative(o), Relative(_)) => Adaptative(o),
            (Relative(_), Adaptative(o)) => Adaptative(o),
            (Adaptative(o), Hybrid(a, _)) => Adaptative(o - a),
            (Hybrid(a, _), Adaptative(o)) => Adaptative(a - o),
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
            (Adaptative(o1), Adaptative(o2)) => o1 == o2,

            (Absolute(0), Relative(0.0)) | (Relative(0.0), Absolute(0)) => true,
            (Absolute(0), Hybrid(0, 0.0)) | (Hybrid(0, 0.0), Absolute(0)) => true,

            (Absolute(a), Hybrid(a2, 0.0)) | (Hybrid(a2, 0.0), Absolute(a)) => a == a2,
            (Relative(r), Hybrid(0, r2)) | (Hybrid(0, r2), Relative(r)) => r == r2,

            (Adaptative(0), Relative(1.0)) | (Relative(1.0), Adaptative(0)) => true,
            (Adaptative(o), Hybrid(a, 1.0)) => a == o,
            _ => false,
        }
    }
}
impl Eq for Coordinate {}