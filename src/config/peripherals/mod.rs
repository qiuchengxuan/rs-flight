use core::fmt::Write;
use core::str::Split;

use super::setter::{Error, Setter, Value};
use super::yaml::ToYAML;

pub mod pwm;
pub mod serial;

use pwm::PWMs;
use serial::Serials;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Peripherals {
    pub serials: Serials,
    pub pwms: PWMs,
}

impl Peripherals {
    pub fn any(&self) -> bool {
        self.serials.len() > 0 || self.pwms.len() > 0
    }
}

impl Setter for Peripherals {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        match path.next().ok_or(Error::MalformedPath)? {
            "serial" => self.serials.set(path, value),
            "pwm" => self.pwms.set(path, value),
            _ => Err(Error::MalformedPath),
        }
    }
}

impl ToYAML for Peripherals {
    fn write_to(&self, indent: usize, w: &mut impl Write) -> core::fmt::Result {
        if self.serials.len() == 0 && self.pwms.len() == 0 {
            return Ok(());
        }
        if self.serials.len() > 0 {
            self.write_indent(indent, w)?;
            writeln!(w, "serial:")?;
            self.serials.write_to(indent + 1, w)?;
        }
        if self.pwms.len() > 0 {
            self.write_indent(indent, w)?;
            writeln!(w, "pwm:")?;
            self.pwms.write_to(indent + 1, w)?;
        }
        Ok(())
    }
}
