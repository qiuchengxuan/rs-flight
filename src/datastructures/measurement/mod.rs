use core::str::FromStr;

use integer_sqrt::IntegerSquareRoot;
use nalgebra::Vector3;

pub mod battery;
pub mod displacement;
pub mod distance;
pub mod euler;
pub mod unit;

use crate::datastructures::decimal::IntegerDecimal;

use distance::Distance;
use unit::CentiMeter;

pub type Velocity<T, U> = distance::Distance<T, U>;
pub type VelocityVector<T, U> = displacement::DistanceVector<T, U>;

pub type Temperature = i16;
pub type Altitude = Distance<i32, CentiMeter>;

pub type Heading = IntegerDecimal;
pub type Course = IntegerDecimal;

#[derive(Copy, Clone)]
pub enum HeadingOrCourse {
    Heading(Heading),
    Course(Course),
}

impl HeadingOrCourse {
    pub fn or_course(self) -> Heading {
        match self {
            Self::Heading(h) => h,
            Self::Course(c) => c,
        }
    }
}

impl Default for HeadingOrCourse {
    fn default() -> Self {
        Self::Course(IntegerDecimal::default())
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Rotation {
    NoRotation,
    Degree90,
    Degree180,
    Degree270,
}

impl Default for Rotation {
    fn default() -> Self {
        Self::NoRotation
    }
}

impl FromStr for Rotation {
    type Err = ();

    fn from_str(name: &str) -> Result<Rotation, ()> {
        match name {
            "0" => Ok(Rotation::NoRotation),
            "90" => Ok(Rotation::Degree90),
            "180" => Ok(Rotation::Degree180),
            "270" => Ok(Rotation::Degree270),
            _ => Err(()),
        }
    }
}

impl core::fmt::Display for Rotation {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let s = match self {
            Self::NoRotation => "0",
            Self::Degree90 => "90",
            Self::Degree180 => "180",
            Self::Degree270 => "270",
        };
        write!(f, "{}", s)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Default, Value)]
pub struct Axes {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Axes {
    pub const MAX: Axes = Axes { x: i32::MAX, y: i32::MAX, z: i32::MAX };
    pub const MIN: Axes = Axes { x: i32::MIN, y: i32::MIN, z: i32::MIN };

    pub fn rotate(self, rotation: Rotation) -> Self {
        let (x, y, z) = (self.x, self.y, self.z);
        let (x, y, z) = match rotation {
            Rotation::NoRotation => (x, y, z),
            Rotation::Degree90 => (y, x, z),
            Rotation::Degree180 => (-x, -y, z),
            Rotation::Degree270 => (-y, x, z),
        };
        Self { x, y, z }
    }
}

impl core::ops::Add for Axes {
    type Output = Axes;

    fn add(self, other: Axes) -> Self {
        Self { x: (self.x + other.x), y: (self.y + other.y), z: (self.z + other.z) }
    }
}

impl core::ops::Sub<&Axes> for Axes {
    type Output = Axes;

    fn sub(self, other: &Axes) -> Self {
        Self { x: (self.x - other.x), y: (self.y - other.y), z: (self.z - other.z) }
    }
}

impl core::ops::Div<i32> for Axes {
    type Output = Axes;

    fn div(self, div: i32) -> Self {
        Self { x: self.x / div, y: self.y / div, z: self.z / div }
    }
}

impl core::ops::Mul<&Axes> for Axes {
    type Output = Axes;

    fn mul(self, other: &Axes) -> Self {
        Self { x: self.x * other.x, y: self.y * other.y, z: self.z * other.z }
    }
}

impl PartialOrd for Axes {
    fn partial_cmp(self: &Self, other: &Self) -> Option<core::cmp::Ordering> {
        if self.x > other.x || self.y > other.y || self.z > other.z {
            Some(core::cmp::Ordering::Greater)
        } else {
            Some(core::cmp::Ordering::Less)
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Gain {
    pub x: u16,
    pub y: u16,
    pub z: u16,
}

impl Default for Gain {
    fn default() -> Self {
        Self { x: u16::MAX / 2, y: u16::MAX / 2, z: u16::MAX / 2 }
    }
}

impl Into<Axes> for Gain {
    fn into(self) -> Axes {
        Axes { x: self.x as i32, y: self.y as i32, z: self.z as i32 }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Value)]
pub struct Measurement {
    pub axes: Axes,
    pub sensitive: i32,
}

impl Measurement {
    pub fn zero(self, axes: &Axes) -> Self {
        Self { axes: self.axes - axes, sensitive: self.sensitive }
    }

    pub fn gain(self, gain: &Axes) -> Self {
        return Self { axes: self.axes * gain / self.sensitive, sensitive: self.sensitive };
    }

    pub fn rotate(self, rotation: Rotation) -> Self {
        Self { axes: self.axes.rotate(rotation), sensitive: self.sensitive }
    }
}

impl PartialOrd for Measurement {
    fn partial_cmp(self: &Self, other: &Self) -> Option<core::cmp::Ordering> {
        self.axes.partial_cmp(&other.axes)
    }
}

impl Into<Vector3<f32>> for Measurement {
    fn into(self) -> Vector3<f32> {
        Vector3::new(
            self.axes.x as f32 / self.sensitive as f32,
            self.axes.y as f32 / self.sensitive as f32,
            self.axes.z as f32 / self.sensitive as f32,
        )
    }
}

impl Default for Measurement {
    fn default() -> Self {
        Self { axes: Default::default(), sensitive: i32::MAX }
    }
}

pub const GRAVITY: f32 = 9.80665;

#[derive(Debug, Default, Copy, Clone)]
pub struct Acceleration(pub Measurement);

impl sval::Value for Acceleration {
    fn stream(&self, stream: &mut sval::value::Stream) -> sval::value::Result {
        self.0.stream(stream)
    }
}

impl Acceleration {
    pub fn g_force(&self) -> u8 {
        let axes = self.0.axes;
        let (x, y, z) = (axes.x, axes.y, axes.z);
        let square_sum = x * x + y * y + z * z;
        if square_sum > 0 {
            let g_force = square_sum.integer_sqrt();
            (g_force * 10 / self.0.sensitive) as u8
        } else {
            0
        }
    }
}

pub type Gyro = Measurement;

#[derive(Copy, Clone, Default)]
pub struct Pressure(pub u32); // unit of Pa

impl Into<Altitude> for Pressure {
    fn into(self) -> Altitude {
        Distance::new(((1013_25 - self.0 as isize) * 82 / 10) as i32, CentiMeter)
    }
}

pub type Magnetism = Measurement;

mod test {
    #[test]
    fn test_distance_unit_convert() {
        use super::{Altitude, Distance};
        use crate::datastructures::measurement::unit::{CentiMeter, Feet, Meter, NauticalMile};

        let altitude = Altitude::new(1000, CentiMeter);
        assert_eq!(altitude.to_unit(Meter), Distance::new(10, Meter));
        assert_eq!(altitude.to_unit(Feet), Distance::new(33, Feet));

        let distance = Distance::new(1, NauticalMile);
        assert_eq!(distance.to_unit(Meter), Distance::new(1852, Meter));
    }

    #[test]
    fn test_speed_unit_convert() {
        use super::Velocity;
        use crate::datastructures::measurement::unit::{FTpM, KMpH, Knot, Meter};

        let knot = Velocity::new(186, KMpH);
        assert_eq!(knot.to_unit(Knot), Velocity::new(100, Knot));

        let meter = Velocity::new(1800, FTpM);
        assert_eq!(meter.to_unit(Meter), Velocity::new(9, Meter));
    }
}
