use std::f64::consts::PI;
use std::cmp::Ordering;

#[derive(Copy, Clone)]
pub struct Angle {
    value: f64,
    sin: f64,
    cos: f64,
}

impl Angle {
    pub fn value(&self) -> f64 { self.value }
    pub fn sin(&self) -> f64 { self.sin }
    pub fn cos(&self) -> f64 { self.cos }
    
    pub fn new(value: f64, sin: f64, cos: f64) -> Self {
        Angle {
            value: value,
            sin: sin,
            cos: cos,
        }
    }

    pub fn wrapped(&self) -> Angle {
        Angle::new(Angle::wrap(self.value), self.sin, self.cos)
    }

    pub fn is_between(&self, start: f64, end: f64) -> bool {
        if start < end {
            start <= self.value && self.value <= end
        } else {
            start < self.value || self.value < end
        }
    }

    pub fn is_in_range(&self, start: f64, end: f64) -> Ordering {
        if self.is_between(start, end) {
            Ordering::Equal
        } else if Angle::wrap(self.value - end).abs() < Angle::wrap(self.value - start).abs() {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }

    pub fn wrap(value: f64) -> f64 {
        if value > PI {
            value - 2.0 * PI
        } else if value < -PI {
            value + 2.0 * PI
        } else {
            value
        }
    }
}

impl From<f64> for Angle {
    fn from(value: f64) -> Self {
        Angle {
            value: value,
            sin: value.sin(),
            cos: value.cos(),
        }
    }
}

impl PartialEq for Angle {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialOrd for Angle {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}