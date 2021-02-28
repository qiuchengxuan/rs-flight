use core::fmt::Write;
use core::str::Split;

use heapless::consts::U18;
use heapless::LinearMap;

use crate::datastructures::input::InputType;

use super::setter::{Error, Setter, Value};
use super::yaml::ToYAML;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Input {
    pub channel: u8,
    pub scale: u8,
}

impl Setter for Input {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        match path.next().ok_or(Error::MalformedPath)? {
            "channel" => {
                self.channel = value.parse()?.ok_or(Error::ExpectValue)?;
                self.channel = self.channel.wrapping_sub(1)
            }
            "scale" => self.scale = value.parse()?.unwrap_or(100),
            _ => return Err(Error::MalformedPath),
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Inputs(pub LinearMap<InputType, Input, U18>);

impl Setter for Inputs {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        let type_sring = path.next().ok_or(Error::MalformedPath)?;
        let input_type = type_sring.parse().map_err(|_| Error::MalformedPath)?;
        if self.0.contains_key(&input_type) {
            return self.0[&input_type].set(path, value);
        }
        let mut config = Input { channel: u8::MAX, scale: 100 };
        config.set(path, value)?;
        self.0.insert(input_type, config).ok();
        Ok(())
    }
}

impl ToYAML for Inputs {
    fn write_to(&self, indent: usize, w: &mut impl Write) -> core::fmt::Result {
        for (input_type, config) in self.0.iter() {
            self.write_indent(indent, w)?;
            writeln!(w, "{}:", input_type)?;
            self.write_indent(indent, w)?;
            writeln!(w, "  channel: {}", config.channel + 1)?;
            if config.scale != 100 {
                self.write_indent(indent, w)?;
                writeln!(w, "  scale: {}", config.scale)?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Receiver {
    pub inputs: Inputs,
}

impl ToYAML for Receiver {
    fn write_to(&self, indent: usize, w: &mut impl Write) -> core::fmt::Result {
        self.write_indent(indent, w)?;
        writeln!(w, "inputs:")?;
        self.inputs.write_to(indent + 1, w)
    }
}

impl Setter for Receiver {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        let key = path.next().ok_or(Error::MalformedPath)?;
        if key != "inputs" {
            return Err(Error::MalformedPath);
        }
        self.inputs.set(path, value)
    }
}
