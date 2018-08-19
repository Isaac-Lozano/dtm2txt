use std::io::{self, Read, BufRead, BufReader};

use serde::Deserialize;
use serde_json;
use serde_json::de::IoRead as JsonIoRead;

use dtm::{Dtm, ControllerInput};
use error::{Dtm2txtError, Dtm2txtResult};

macro_rules! read_input {
    ($line:expr, $token:expr, $upper:expr, $lower:expr) => {
        {
            match $token {
                $upper => true,
                $lower => false,
                _ => return Err(Dtm2txtError::ControllerInputParseError {
                    line: $line,
                    reason: "incorrect button value",
                }),
            }
        }
    };
}

struct LineCountRead<R> {
    inner: R,
    lines: u64,
}

impl<R> LineCountRead<R>
    where R: Read,
{
    fn new(inner: R) -> LineCountRead<R> {
        LineCountRead {
            inner: inner,
            lines: 0,
        }
    }

    fn lines_read(&self) -> u64 {
        self.lines
    }
}

impl<R> Read for LineCountRead<R>
    where R: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let bytes_read = self.inner.read(buf)?;
        for byte in buf[0..bytes_read].iter() {
            if *byte == '\n' as u8 {
                self.lines += 1;
            }
        }
        Ok(bytes_read)
    }
}

struct InputReader {
    line: u64,
}

impl InputReader {
    fn new() -> InputReader {
        InputReader {
            line: 0,
        }
    }

    fn read_controller_input(&mut self, line: &str) -> Dtm2txtResult<ControllerInput> {
        let mut tokens = line.split_whitespace();
        // TODO: More error checks.
        let start = read_input!(self.line, tokens.next().unwrap(), "S", "s");
        let a = read_input!(self.line, tokens.next().unwrap(), "A", "a");
        let b = read_input!(self.line, tokens.next().unwrap(), "B", "b");
        let x = read_input!(self.line, tokens.next().unwrap(), "X", "x");
        let y = read_input!(self.line, tokens.next().unwrap(), "Y", "y");
        let z = read_input!(self.line, tokens.next().unwrap(), "Z", "z");
        let up = read_input!(self.line, tokens.next().unwrap(), "U", "u");
        let down = read_input!(self.line, tokens.next().unwrap(), "D", "d");
        let left = read_input!(self.line, tokens.next().unwrap(), "L", "l");
        let right = read_input!(self.line, tokens.next().unwrap(), "R", "r");
        let l = read_input!(self.line, tokens.next().unwrap(), "LT", "lt");
        let r = read_input!(self.line, tokens.next().unwrap(), "RT", "rt");
        let l_pressure = tokens.next().unwrap().parse::<u8>().unwrap();
        let r_pressure = tokens.next().unwrap().parse::<u8>().unwrap();
        let analog_x = tokens.next().unwrap().parse::<u8>().unwrap();
        let analog_y = tokens.next().unwrap().parse::<u8>().unwrap();
        let c_x = tokens.next().unwrap().parse::<u8>().unwrap();
        let c_y = tokens.next().unwrap().parse::<u8>().unwrap();

        let mut change_disc = false;
        let mut reset = false;
        let mut controller_connected = false;
        let mut reserved = false;
        for token in tokens {
            match token {
                "CD" => change_disc = true,
                "RST" => reset = true,
                "CC" => controller_connected = true,
                "RSV" => reserved = true,
                _ => panic!("Too lazy to write an error function here."),
            }
        }

        Ok(ControllerInput {
            start: start,
            a: a,
            b: b,
            x: x,
            y: y,
            z: z,
            up: up,
            down: down,
            left: left,
            right: right,
            l: l,
            r: r,
            change_disc: change_disc,
            reset: reset,
            controller_connected: controller_connected,
            reserved: reserved,
            l_pressure: l_pressure,
            r_pressure: r_pressure,
            analog_x: analog_x,
            analog_y: analog_y,
            c_x: c_x,
            c_y: c_y,
        })
    }
}

pub struct TextDecoder<R> {
    inner: LineCountRead<R>,
    input_reader: InputReader,
}

impl<R> TextDecoder<R>
    where R: Read,
{
    pub fn new(inner: R) -> TextDecoder<R> {
        TextDecoder {
            inner: LineCountRead::new(inner),
            input_reader: InputReader::new(),
        }
    }

    pub fn decode(mut self) -> Dtm2txtResult<Dtm>
        where R: Read,
    {
        let header = {
            let mut de = serde_json::Deserializer::new(JsonIoRead::new(&mut self.inner));
            Deserialize::deserialize(&mut de)?
        };

        self.input_reader.line += self.inner.lines_read();

        let line_reader = BufReader::new(self.inner);
        let mut controller_data = Vec::new();
        for line in line_reader.lines().skip(1) {
            controller_data.push(self.input_reader.read_controller_input(&line.unwrap())?);
        }

        Ok(Dtm {
            header: header,
            controller_data: controller_data,
        })
    }
}