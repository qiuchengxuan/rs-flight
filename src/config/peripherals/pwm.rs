use core::cmp;
use core::fmt::Write;
use core::str::{FromStr, Split};

use heapless::consts::U8;
use heapless::LinearMap;

use crate::config::setter::{Error, Setter, Value};
use crate::config::yaml::ToYAML;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Identifier(pub u8);

impl FromStr for Identifier {
    type Err = ();

    fn from_str(name: &str) -> Result<Identifier, ()> {
        if name.starts_with("PWM") {
            return Ok(Identifier(name[3..].parse::<u8>().map_err(|_| ())? - 1));
        }
        Err(())
    }
}

impl core::fmt::Display for Identifier {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "PWM{}", self.0 + 1)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Protocol {
    PWM,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Motor {
    pub protocol: Protocol,
    pub index: u8,
    pub rate: u16,
}

impl Motor {
    pub fn new(protocol: Protocol, index: u8, rate: u16) -> Self {
        Self { protocol, index, rate }
    }
}

impl Default for Motor {
    fn default() -> Self {
        Self { protocol: Protocol::PWM, index: 0, rate: 400 }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ServoType {
    Aileron,
    Elevator,
    Rudder,
    ElevonLeft,
    ElevonRight,
}

impl Into<&str> for ServoType {
    fn into(self) -> &'static str {
        match self {
            Self::Aileron => "aileron",
            Self::Elevator => "elevator",
            Self::Rudder => "rudder",
            Self::ElevonLeft => "elevon-left",
            Self::ElevonRight => "elevon-right",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Servo {
    pub servo_type: ServoType,
    pub min_angle: i8,
    pub max_angle: i8,
    pub reversed: bool,
}

impl Servo {
    pub fn new(servo_type: ServoType, min_angle: i8, max_angle: i8, reversed: bool) -> Self {
        Self { servo_type, min_angle, max_angle, reversed }
    }

    pub fn of(servo_type: ServoType) -> Self {
        Self { servo_type, min_angle: -90, max_angle: 90, reversed: false }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PWM {
    Motor(Motor),
    Servo(Servo),
}

impl PWM {
    pub fn rate(self) -> u16 {
        match self {
            Self::Motor(motor) => motor.rate,
            _ => 50,
        }
    }
}

impl Setter for PWM {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        let key = path.next().ok_or(Error::MalformedPath)?;
        if key == "type" {
            let output_type = value.0.ok_or(Error::ExpectValue)?;
            *self = match output_type {
                "motor" => Self::Motor(Motor::default()),
                "aileron" => Self::Servo(Servo::of(ServoType::Aileron)),
                "elevator" => Self::Servo(Servo::of(ServoType::Elevator)),
                "rudder" => Self::Servo(Servo::of(ServoType::Rudder)),
                "elevon-left" => Self::Servo(Servo::of(ServoType::ElevonLeft)),
                "elevon-right" => Self::Servo(Servo::of(ServoType::ElevonRight)),
                _ => return Err(Error::UnexpectedValue),
            };
            return Ok(());
        }
        match self {
            Self::Motor(ref mut motor) => match key {
                "index" => motor.index = value.parse()?.unwrap_or(0),
                "protocol" => match value.0 {
                    Some("PWM") => motor.protocol = Protocol::PWM,
                    Some(_) => return Err(Error::UnexpectedValue),
                    _ => motor.protocol = Protocol::PWM,
                },
                "rate" => motor.rate = value.parse()?.unwrap_or(400),
                _ => return Err(Error::MalformedPath),
            },
            Self::Servo(ref mut servo) => match key {
                "min-angle" => {
                    let min = value.parse()?.unwrap_or(-90);
                    servo.min_angle = cmp::min(cmp::max(min, -90), 0)
                }
                "max-angle" => {
                    let max = value.parse()?.unwrap_or(90);
                    servo.max_angle = cmp::max(cmp::min(max, 90), 0)
                }
                "reversed" => servo.reversed = value.parse()?.unwrap_or_default(),
                _ => return Err(Error::MalformedPath),
            },
        }
        Ok(())
    }
}

impl ToYAML for PWM {
    fn write_to(&self, indent: usize, w: &mut impl Write) -> core::fmt::Result {
        match self {
            Self::Motor(motor) => {
                self.write_indent(indent, w)?;
                writeln!(w, "type: motor")?;
                self.write_indent(indent, w)?;
                writeln!(w, "index: {}", motor.index)?;
                match motor.protocol {
                    Protocol::PWM => {
                        self.write_indent(indent, w)?;
                        writeln!(w, "protocol: PWM")?;
                    }
                }
                self.write_indent(indent, w)?;
                writeln!(w, "rate: {}", motor.rate)?;
            }
            Self::Servo(servo) => {
                self.write_indent(indent, w)?;
                let servo_type: &str = servo.servo_type.into();
                writeln!(w, "type: {}", servo_type)?;
                if servo.min_angle != -90 {
                    self.write_indent(indent, w)?;
                    writeln!(w, "min-angle: {}", servo.min_angle)?;
                }
                if servo.max_angle != 90 {
                    self.write_indent(indent, w)?;
                    writeln!(w, "max-angle: {}", servo.max_angle)?;
                }
                if servo.reversed {
                    self.write_indent(indent, w)?;
                    writeln!(w, "reversed: true")?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PWMs(pub LinearMap<Identifier, PWM, U8>);

impl PWMs {
    pub fn get(&self, name: &str) -> Option<&PWM> {
        Identifier::from_str(name).ok().map(|id| self.0.get(&id)).flatten()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl ToYAML for PWMs {
    fn write_to(&self, indent: usize, w: &mut impl Write) -> core::fmt::Result {
        for (id, config) in self.0.iter() {
            self.write_indent(indent, w)?;
            writeln!(w, "{}:", id)?;
            config.write_to(indent + 1, w)?;
        }
        Ok(())
    }
}

impl Setter for PWMs {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        let id_string = path.next().ok_or(Error::MalformedPath)?;
        if !id_string.starts_with("PWM") {
            return Err(Error::MalformedPath);
        }
        let id = id_string.parse().map_err(|_| Error::MalformedPath)?;
        if self.0.contains_key(&id) {
            return self.0[&id].set(path, value);
        }
        let mut config = PWM::Motor(Motor::default());
        config.set(path, value)?;
        self.0.insert(id, config).ok();
        Ok(())
    }
}
