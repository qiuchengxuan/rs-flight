use core::fmt::Write;
use core::str::Split;

use crate::config::setter::{Error, Setter, Value};
use crate::config::yaml::ToYAML;
use crate::datastructures::measurement::Rotation;

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Board {
    pub rotation: Rotation,
}

impl Setter for Board {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        match path.next().ok_or(Error::MalformedPath)? {
            "rotation" => self.rotation = value.parse()?.unwrap_or(Rotation::NoRotation),
            _ => return Err(Error::MalformedPath),
        }
        Ok(())
    }
}

impl ToYAML for Board {
    fn write_to(&self, indent: usize, w: &mut impl Write) -> core::fmt::Result {
        self.write_indent(indent, w)?;
        writeln!(w, "rotation: {}", self.rotation)
    }
}
