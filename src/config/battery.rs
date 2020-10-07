use core::fmt::Write;
use core::str::Split;

use crate::datastructures::decimal::IntegerDecimal;

use super::setter::{SetError, Setter};
use super::yaml::ToYAML;

#[derive(Copy, Clone, Debug)]
pub struct Battery {
    pub cells: u8,
    pub min_cell_voltage: IntegerDecimal,
    pub max_cell_voltage: IntegerDecimal,
    pub warning_cell_voltage: IntegerDecimal,
}

impl Default for Battery {
    fn default() -> Self {
        Self {
            cells: 0,
            min_cell_voltage: IntegerDecimal::from("3.3"),
            max_cell_voltage: IntegerDecimal::from("4.2"),
            warning_cell_voltage: IntegerDecimal::from("3.5"),
        }
    }
}

impl Setter for Battery {
    fn set(&mut self, path: &mut Split<char>, value: Option<&str>) -> Result<(), SetError> {
        let key = path.next().ok_or(SetError::MalformedPath)?;
        let value = value.ok_or(SetError::ExpectValue)?;
        match path.next() {
            "cells" => self.cells = value.parse().map_err(|_| SetError::UnexpectedValue)?,
            "min-cell-voltage" => self.min_cell_voltage = IntegerDecimal::from(value),
            "max-cell-voltage" => self.max_cell_voltage = IntegerDecimal::from(value),
            "warning-cell-voltage" => self.warning_cell_voltage = IntegerDecimal::from(value),
            _ => return Err(SetError::MalformedPath),
        }
    }
}

impl ToYAML for Battery {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        self.write_indent(indent, w)?;
        writeln!(w, "cells: {}", self.cells)?;
        self.write_indent(indent, w)?;
        writeln!(w, "min-cell-voltage: {}", self.min_cell_voltage)?;
        self.write_indent(indent, w)?;
        writeln!(w, "max-cell-voltage: {}", self.max_cell_voltage)?;
        self.write_indent(indent, w)?;
        writeln!(w, "warning-cell-voltage: {}", self.warning_cell_voltage)
    }
}
