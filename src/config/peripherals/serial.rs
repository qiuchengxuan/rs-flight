use core::fmt::Write;
use core::str::{FromStr, Split};

use heapless::consts::U8;
use heapless::LinearMap;

use crate::config::setter::{Error, Setter, Value};
use crate::config::yaml::ToYAML;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Identifier {
    UART(u8),
    USART(u8),
}

impl FromStr for Identifier {
    type Err = ();

    fn from_str(name: &str) -> Result<Self, ()> {
        if name.starts_with("USART") {
            return Ok(Identifier::USART(name[5..].parse().map_err(|_| ())?));
        } else if name.starts_with("UART") {
            return Ok(Identifier::UART(name[4..].parse().map_err(|_| ())?));
        }
        Err(())
    }
}

impl Into<bool> for Identifier {
    fn into(self) -> bool {
        match self {
            Self::UART(index) => index > 0,
            Self::USART(index) => index > 0,
        }
    }
}

impl core::fmt::Display for Identifier {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::USART(index) => write!(f, "USART{}", index),
            Self::UART(index) => write!(f, "UART{}", index),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GNSSProtocol {
    UBX,
    NMEA,
}

impl FromStr for GNSSProtocol {
    type Err = ();

    fn from_str(string: &str) -> Result<Self, ()> {
        match string {
            "UBX" => Ok(Self::UBX),
            "NMEA" => Ok(Self::NMEA),
            _ => Err(()),
        }
    }
}

impl core::fmt::Display for GNSSProtocol {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::UBX => write!(f, "UBX"),
            Self::NMEA => write!(f, "NMEA"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GNSSConfig {
    pub baudrate: u32,
    pub protocol: GNSSProtocol,
}

impl Default for GNSSConfig {
    fn default() -> Self {
        Self { baudrate: 9600, protocol: GNSSProtocol::NMEA }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SbusConfig {
    pub fast: bool,
    pub rx_inverted: bool,
    pub half_duplex: bool,
}

impl Default for SbusConfig {
    fn default() -> Self {
        Self { fast: false, rx_inverted: true, half_duplex: false }
    }
}

impl SbusConfig {
    pub fn baudrate(&self) -> u32 {
        if !self.fast {
            100_000
        } else {
            200_000
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Config {
    GNSS(GNSSConfig),
    SBUS(SbusConfig),
}

impl Setter for Config {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        let key = path.next().ok_or(Error::MalformedPath)?;
        if key == "type" {
            *self = match value.0 {
                Some("GNSS") => Self::GNSS(GNSSConfig::default()),
                Some("SBUS") => Self::SBUS(SbusConfig::default()),
                Some(_) => return Err(Error::UnexpectedValue),
                _ => return Err(Error::ExpectValue),
            };
            return Ok(());
        }
        match self {
            Self::GNSS(ref mut gnss) => match key {
                "baudrate" => gnss.baudrate = value.parse()?.unwrap_or(9600),
                "protocol" => gnss.protocol = value.parse()?.unwrap_or(GNSSProtocol::NMEA),
                _ => return Err(Error::MalformedPath),
            },
            Self::SBUS(ref mut sbus) => match key {
                "fast" => sbus.fast = value.parse()?.unwrap_or(false),
                "rx-inverted" => sbus.rx_inverted = value.parse()?.unwrap_or(true),
                "half-duplex" => sbus.half_duplex = value.parse()?.unwrap_or(false),
                _ => return Err(Error::MalformedPath),
            },
        }
        Ok(())
    }
}

impl ToYAML for Config {
    fn write_to(&self, indent: usize, w: &mut impl Write) -> core::fmt::Result {
        self.write_indent(indent, w)?;
        match self {
            Self::GNSS(gnss) => {
                writeln!(w, "type: GNSS")?;
                self.write_indent(indent, w)?;
                writeln!(w, "baudrate: {}", gnss.baudrate)?;
                self.write_indent(indent, w)?;
                writeln!(w, "protocol: {}", gnss.protocol)
            }
            Self::SBUS(sbus) => {
                writeln!(w, "type: SBUS")?;
                self.write_indent(indent, w)?;
                writeln!(w, "fast: {}", sbus.fast)?;
                self.write_indent(indent, w)?;
                writeln!(w, "rx-inverted: {}", sbus.rx_inverted)?;
                self.write_indent(indent, w)?;
                writeln!(w, "half-duplex: {}", sbus.half_duplex)
            }
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Serials(LinearMap<Identifier, Config, U8>);

impl Serials {
    pub fn get(&self, name: &str) -> Option<&Config> {
        Identifier::from_str(name).ok().map(|id| self.0.get(&id)).flatten()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Setter for Serials {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        let id_sring = path.next().ok_or(Error::MalformedPath)?;
        let id = Identifier::from_str(id_sring).map_err(|_| Error::MalformedPath)?;
        if self.0.contains_key(&id) {
            return self.0[&id].set(path, value);
        }
        let mut config = Config::GNSS(GNSSConfig::default());
        config.set(path, value)?;
        self.0.insert(id, config).ok();
        Ok(())
    }
}

impl ToYAML for Serials {
    fn write_to(&self, indent: usize, w: &mut impl Write) -> core::fmt::Result {
        for (&id, config) in self.0.iter() {
            if id.into() {
                self.write_indent(indent, w)?;
                writeln!(w, "{}:", id)?;
                config.write_to(indent + 1, w)?;
            }
        }
        Ok(())
    }
}
