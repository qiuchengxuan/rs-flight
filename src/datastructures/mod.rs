use core::str::FromStr;

pub mod coordinate;
pub mod data_source;
#[macro_use]
pub mod decimal;
pub mod input;
pub mod measurement;
pub mod waypoint;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Ratio(pub u8, pub u8);

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct GNSSFixed(pub bool);

impl Into<u16> for GNSSFixed {
    fn into(self) -> u16 {
        self.0 as u16
    }
}

impl From<u16> for GNSSFixed {
    fn from(value: u16) -> Self {
        Self(value > 0)
    }
}

impl Into<bool> for GNSSFixed {
    fn into(self) -> bool {
        self.0
    }
}

impl FromStr for Ratio {
    type Err = ();

    fn from_str(string: &str) -> Result<Self, ()> {
        let mut splitted = string.split(':');
        let ratio_0 = splitted.next().ok_or(())?.parse().map_err(|_| ())?;
        let ratio_1 = splitted.next().ok_or(())?.parse().map_err(|_| ())?;
        Ok(Self(ratio_0, ratio_1))
    }
}

impl Default for Ratio {
    fn default() -> Self {
        Self(1, 1)
    }
}

impl core::fmt::Display for Ratio {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}
