pub mod aircraft;
pub mod battery;
pub mod imu;
pub mod osd;
pub mod output;
pub mod parser;
pub mod receiver;
pub mod serial;
pub mod setter;
pub mod yaml;

use core::fmt::Write;
use core::str::Split;

use crate::datastructures::decimal::IntegerDecimal;
use crate::datastructures::measurement::Axes;
use crate::hal::io::Read;

pub use aircraft::Aircraft;
pub use battery::Battery;
pub use imu::IMU;
pub use osd::{Offset, Standard, OSD};
pub use output::{Output, Outputs, Protocol};
pub use receiver::Receiver;
pub use serial::{Config as SerialConfig, Serials};
use setter::{SetError, Setter};
use yaml::{FromYAML, ToYAML, YamlParser};

impl FromYAML for Axes {
    fn from_yaml<'a>(parser: &mut YamlParser<'a>) -> Self {
        let mut axis = Self::default();
        while let Some((key, value)) = parser.next_key_value() {
            let value = value.parse().unwrap_or(0);
            match key {
                "x" => axis.x = value,
                "y" => axis.y = value,
                "z" => axis.z = value,
                _ => continue,
            }
        }
        axis
    }
}

impl ToYAML for Axes {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> core::fmt::Result {
        self.write_indent(indent, w)?;
        writeln!(w, "x: {}", self.x)?;
        self.write_indent(indent, w)?;
        writeln!(w, "y: {}", self.y)?;
        self.write_indent(indent, w)?;
        writeln!(w, "z: {}", self.z)
    }
}

#[derive(Clone)]
pub struct Speedometer {
    pub kp: IntegerDecimal,
}

impl Default for Speedometer {
    fn default() -> Self {
        Self { kp: IntegerDecimal::new(25, 2) }
    }
}

impl FromYAML for Speedometer {
    fn from_yaml<'a>(parser: &mut YamlParser<'a>) -> Self {
        let mut kp = IntegerDecimal::default();
        while let Some((key, value)) = parser.next_key_value() {
            match key {
                "kp" => kp = IntegerDecimal::from(value),
                _ => continue,
            }
        }
        Self { kp }
    }
}

impl ToYAML for Speedometer {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> core::fmt::Result {
        self.write_indent(indent, w)?;
        writeln!(w, "kp: {}", self.kp)
    }
}

#[derive(Default, Clone)]
pub struct Config {
    pub battery: Battery,
    pub aircraft: Aircraft,
    pub imu: IMU,
    pub osd: OSD,
    pub receiver: Receiver,
    pub serials: Serials,
    pub speedometer: Speedometer,
    pub outputs: Outputs,
}

impl Setter for Config {
    fn set(&mut self, path: &mut Split<char>, value: Option<&str>) -> Result<(), SetError> {
        match path.next() {
            Some("receiver") => self.receiver.set(path, value),
            Some("outputs") => self.outputs.set(path, value),
            Some("osd") => self.osd.set(path, value),
            _ => Err(SetError::MalformedPath),
        }
    }
}

impl FromYAML for Config {
    fn from_yaml<'a>(parser: &mut YamlParser<'a>) -> Self {
        let mut config = Self::default();
        while let Some(key) = parser.next_entry() {
            match key {
                "aircraft" => config.aircraft = Aircraft::from_yaml(parser),
                "battery" => config.battery = Battery::from_yaml(parser),
                "imu" => config.imu = IMU::from_yaml(parser),
                "osd" => config.osd = OSD::from_yaml(parser),
                "receiver" => config.receiver = Receiver::from_yaml(parser),
                "serials" => config.serials = Serials::from_yaml(parser),
                "speedometer" => config.speedometer = Speedometer::from_yaml(parser),
                "outputs" => config.outputs = Outputs::from_yaml(parser),
                _ => parser.skip(),
            };
        }
        config
    }
}

impl ToYAML for Config {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> core::fmt::Result {
        self.write_indent(indent, w)?;
        writeln!(w, "aircraft:")?;
        self.aircraft.write_to(indent + 1, w)?;

        self.write_indent(indent, w)?;
        writeln!(w, "battery:")?;
        self.battery.write_to(indent + 1, w)?;

        self.write_indent(indent, w)?;
        writeln!(w, "imu:")?;
        self.imu.write_to(indent + 1, w)?;

        self.write_indent(indent, w)?;
        writeln!(w, "osd:")?;
        self.osd.write_to(indent + 1, w)?;

        self.write_indent(indent, w)?;
        writeln!(w, "receiver:")?;
        self.receiver.write_to(indent + 1, w)?;

        if self.serials.len() > 0 {
            self.write_indent(indent, w)?;
            writeln!(w, "serials:")?;
            self.serials.write_to(indent + 1, w)?;
        } else {
            self.write_indent(indent, w)?;
            writeln!(w, "serials: []")?;
        }

        self.write_indent(indent, w)?;
        writeln!(w, "speedometer:")?;
        self.speedometer.write_to(indent + 1, w)?;

        if self.outputs.len() > 0 {
            self.write_indent(indent, w)?;
            writeln!(w, "outputs:")?;
            self.outputs.write_to(indent + 1, w)?;
        } else {
            self.write_indent(indent, w)?;
            writeln!(w, "outputs: []")?;
        }
        Ok(())
    }
}

impl core::fmt::Display for Config {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.write_to(0, f)
    }
}

static mut CONFIG: Option<Config> = None;

#[inline]
pub fn get() -> &'static Config {
    unsafe { CONFIG.as_ref().unwrap() }
}

pub fn load<E>(reader: &mut dyn Read<Error = E>) -> &'static Config {
    let mut buffer = [0u8; 2048];
    let size = reader.read(&mut buffer).ok().unwrap_or(0);
    let config = if size > 0 {
        Config::from_yaml(&mut YamlParser::from(&buffer[..size]))
    } else {
        Config::default()
    };
    unsafe { CONFIG = Some(config) }
    get()
}

pub fn replace(config: Config) {
    unsafe { CONFIG = Some(config) }
}

mod test {
    #[cfg(test)]
    extern crate std;

    #[test]
    #[serial]
    fn test_init_config() -> std::io::Result<()> {
        use std::fs::File;
        use std::io::Read;
        use std::string::{String, ToString};

        use super::yaml::{FromYAML, ToYAML, YamlParser};
        use super::Config;

        let mut file = File::open("sample.yml")?;
        let mut yaml_string = String::new();
        file.read_to_string(&mut yaml_string)?;
        let config = Config::from_yaml(&mut YamlParser::from(yaml_string.as_ref() as &str));

        let mut buf = String::new();
        config.write_to(0, &mut buf).ok();
        assert_eq!(yaml_string.trim(), buf.to_string().trim());
        Ok(())
    }

    #[test]
    fn test_set_config() -> std::io::Result<()> {
        use std::fs::File;
        use std::io::Read;
        use std::string::String;

        use super::output::{Output, Servo, ServoType};
        use super::setter::Setter;
        use super::yaml::{FromYAML, YamlParser};
        use super::Config;

        let mut file = File::open("sample.yml")?;
        let mut yaml_string = String::new();
        file.read_to_string(&mut yaml_string)?;
        let mut config = Config::from_yaml(&mut YamlParser::from(yaml_string.as_ref() as &str));

        config.set(&mut "outputs.PWM5.min-angle".split('.'), Some("-80")).unwrap();
        let expected = Output::Servo(Servo::new(ServoType::AileronLeft, -80, 90, false));
        assert_eq!(config.outputs.get("PWM5").unwrap(), expected);
        Ok(())
    }
}
