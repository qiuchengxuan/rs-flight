use core::fmt::Write;
use core::str::Split;

use super::setter::{SetError, Setter};

use super::yaml::ToYAML;

#[derive(Copy, Clone)]
pub enum Configuration {
    Airplane,
}

impl From<&str> for Configuration {
    fn from(string: &str) -> Self {
        match string {
            "airplane" => Self::Airplane,
            _ => Self::Airplane,
        }
    }
}

impl Into<&str> for Configuration {
    fn into(self) -> &'static str {
        match self {
            Self::Airplane => "airplane",
        }
    }
}

#[derive(Copy, Clone)]
pub struct Aircraft {
    pub configuration: Configuration,
}

impl Default for Aircraft {
    fn default() -> Self {
        Self { configuration: Configuration::Airplane }
    }
}

impl Setter for Aircraft {
    fn set(&mut self, path: &mut Split<char>, value: Option<&str>) -> Result<(), SetError> {
        match path.next() {
            Some("configuration") => {
                self.configuration = value.ok_or(SetError::ExpectValue)?.into()
            }
            _ => return Err(SetError::MalformedPath),
        }
        Ok(())
    }
}

impl ToYAML for Aircraft {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> core::fmt::Result {
        self.write_indent(indent, w)?;
        let configuration: &str = self.configuration.into();
        writeln!(w, "configuration: {}", configuration)
    }
}
