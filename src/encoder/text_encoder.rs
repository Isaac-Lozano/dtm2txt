use std::io::Write;

use serde_json;

use dtm::{Dtm, ControllerInput};
use error::Dtm2txtResult;

macro_rules! format_input {
    ($string:expr, $val:expr, $upper:expr, $lower:expr) => {
        if $val {
            $string += $upper;
        }
        else {
            $string += $lower;
        }
    };
}

pub struct TextEncoder<W> {
    inner: W,
}

impl<W> TextEncoder<W>
    where W: Write,
{
    pub fn new(inner: W) -> TextEncoder<W> {
        TextEncoder {
            inner: inner,
        }
    }

    pub fn encode(mut self, dtm: &Dtm) -> Dtm2txtResult<()> {
        serde_json::to_writer_pretty(&mut self.inner, &dtm.header)?;
        writeln!(&mut self.inner)?;
        for input in dtm.controller_data.iter() {
            self.write_controller_input(input)?;
        }
        Ok(())
    }

    // S A B X Y Z U D L R LT 0 0 0 0 0 0 [CD RST CC RSV]
    fn write_controller_input(&mut self, input: &ControllerInput) -> Dtm2txtResult<()> {
        let mut line = String::new();
        format_input!(line, input.start, "S ", "s ");
        format_input!(line, input.a, "A ", "a ");
        format_input!(line, input.b, "B ", "b ");
        format_input!(line, input.x, "X ", "x ");
        format_input!(line, input.y, "Y ", "y ");
        format_input!(line, input.z, "Z ", "z ");
        format_input!(line, input.up, "U ", "u ");
        format_input!(line, input.down, "D ", "d ");
        format_input!(line, input.left, "L ", "l ");
        format_input!(line, input.right, "R ", "r ");
        format_input!(line, input.l, "LT ", "lt ");
        format_input!(line, input.r, "RT ", "rt ");
        line += &(format!("{:3} ", input.l_pressure));
        line += &(format!("{:3} ", input.r_pressure));
        line += &(format!("{:3} ", input.analog_x));
        line += &(format!("{:3} ", input.analog_y));
        line += &(format!("{:3} ", input.c_x));
        line += &(format!("{:3}", input.c_y));
        format_input!(line, input.change_disc, " CD", "");
        format_input!(line, input.reset, " RST", "");
        format_input!(line, input.controller_connected, " CC", "");
        format_input!(line, input.reserved, " RSV", "");
        line += "\n";

        Ok(self.inner.write_all(line.as_bytes())?)
    }
}