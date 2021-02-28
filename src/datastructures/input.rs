use core::str::FromStr;

pub type RSSI = u16;

#[derive(Copy, Clone, Eq, Debug, PartialEq)]
pub enum InputType {
    Throttle = 0,
    Roll,
    Pitch,
    Yaw,
}

impl FromStr for InputType {
    type Err = ();
    fn from_str(string: &str) -> Result<Self, ()> {
        match string {
            "throttle" => Ok(Self::Throttle),
            "roll" => Ok(Self::Roll),
            "pitch" => Ok(Self::Pitch),
            "yaw" => Ok(Self::Yaw),
            _ => Err(()),
        }
    }
}

impl Into<&str> for InputType {
    fn into(self) -> &'static str {
        match self {
            Self::Throttle => "throttle",
            Self::Roll => "roll",
            Self::Pitch => "pitch",
            Self::Yaw => "yaw",
        }
    }
}

impl core::fmt::Display for InputType {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let s: &str = (*self).into();
        write!(f, "{}", s)
    }
}

pub type Throttle = i16;
pub type Roll = i16;
pub type Pitch = i16;
pub type Yaw = i16;
pub enum Flaps {
    Auto,
    Half,
    Full,
}
pub enum LandingGear {
    Up,
    Down,
}

#[derive(Copy, Clone, Debug, Value)]
pub struct ControlInput {
    pub throttle: Throttle,
    pub roll: Roll,
    pub pitch: Pitch,
    pub yaw: Yaw,
}

impl Default for ControlInput {
    fn default() -> Self {
        Self { throttle: i16::MIN, roll: 0, pitch: 0, yaw: 0 }
    }
}

pub struct FixedWingInput {
    pub flaps: Flaps,
    pub landing_gear: LandingGear,
}
