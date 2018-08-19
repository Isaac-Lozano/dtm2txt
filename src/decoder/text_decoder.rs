use std::io::{self, Read, BufRead, BufReader, Error as IoError};

use serde::Deserialize;
use serde_json;
use serde_json::de::IoRead as JsonIoRead;

use dtm::{Dtm, ControllerInput};
use error::{Dtm2txtError, ControllerInputParseError, Dtm2txtResult};

macro_rules! get_token {
    ($option:expr, $line:expr) => {
        match $option {
            Some(value) => value,
            None => return Err(Dtm2txtError::ControllerInputParseError {
                line: $line,
                reason: ControllerInputParseError::MissingTokenError,
            }),
        }
    };
}

macro_rules! read_input {
    ($line:expr, $token:expr, $upper:expr, $lower:expr) => {
        {
            match get_token!($token, $line) {
                $upper => true,
                $lower => false,
                _ => return Err(Dtm2txtError::ControllerInputParseError {
                    line: $line,
                    reason: ControllerInputParseError::InvalidButtonError,
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

    fn read_controller_input(&mut self, line_result: Result<String, IoError>) -> Dtm2txtResult<ControllerInput> {
        let line = line_result
            .map_err(|err| Dtm2txtError::ControllerInputParseError {
                reason: ControllerInputParseError::IoError(err),
                line: self.line,
            })?;
        let mut tokens = line.split_whitespace();
        let start = read_input!(self.line, tokens.next(), "S", "s");
        let a = read_input!(self.line, tokens.next(), "A", "a");
        let b = read_input!(self.line, tokens.next(), "B", "b");
        let x = read_input!(self.line, tokens.next(), "X", "x");
        let y = read_input!(self.line, tokens.next(), "Y", "y");
        let z = read_input!(self.line, tokens.next(), "Z", "z");
        let up = read_input!(self.line, tokens.next(), "U", "u");
        let down = read_input!(self.line, tokens.next(), "D", "d");
        let left = read_input!(self.line, tokens.next(), "L", "l");
        let right = read_input!(self.line, tokens.next(), "R", "r");
        let l = read_input!(self.line, tokens.next(), "LT", "lt");
        let r = read_input!(self.line, tokens.next(), "RT", "rt");
        let l_pressure = get_token!(tokens.next(), self.line)
            .parse::<u8>()
            .map_err(|err| Dtm2txtError::ControllerInputParseError {
                reason: ControllerInputParseError::ParseIntError(err),
                line: self.line,
            })?;
        let r_pressure = get_token!(tokens.next(), self.line)
            .parse::<u8>()
            .map_err(|err| Dtm2txtError::ControllerInputParseError {
                reason: ControllerInputParseError::ParseIntError(err),
                line: self.line,
            })?;
        let analog_x = get_token!(tokens.next(), self.line)
            .parse::<u8>()
            .map_err(|err| Dtm2txtError::ControllerInputParseError {
                reason: ControllerInputParseError::ParseIntError(err),
                line: self.line,
            })?;
        let analog_y = get_token!(tokens.next(), self.line)
            .parse::<u8>()
            .map_err(|err| Dtm2txtError::ControllerInputParseError {
                reason: ControllerInputParseError::ParseIntError(err),
                line: self.line,
            })?;
        let c_x = get_token!(tokens.next(), self.line)
            .parse::<u8>()
            .map_err(|err| Dtm2txtError::ControllerInputParseError {
                reason: ControllerInputParseError::ParseIntError(err),
                line: self.line,
            })?;
        let c_y = get_token!(tokens.next(), self.line)
            .parse::<u8>()
            .map_err(|err| Dtm2txtError::ControllerInputParseError {
                reason: ControllerInputParseError::ParseIntError(err),
                line: self.line,
            })?;

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
                _ => return Err(Dtm2txtError::ControllerInputParseError {
                    reason: ControllerInputParseError::InvalidButtonError,
                    line: self.line,
                }),
            }
        }

        self.line += 1;

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
            controller_data.push(self.input_reader.read_controller_input(line)?);
        }

        Ok(Dtm {
            header: header,
            controller_data: controller_data,
        })
    }
}