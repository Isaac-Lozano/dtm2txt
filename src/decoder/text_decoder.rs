use std::io::{self, Read, BufRead, BufReader, Error as IoError};

use serde::Deserialize;
use serde_json;
use serde_json::de::IoRead as JsonIoRead;

use dtm::{Dtm, DtmHeader, ControllerInput};
use error::{Dtm2txtError, ControllerInputParseError, Dtm2txtResult};

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
            // 1-indexed line numbers.
            lines: 1,
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

    fn get_token<'a>(&self, token_opt: Option<&'a str>) -> Dtm2txtResult<&'a str> {
        match token_opt {
            Some(token) => Ok(token),
            None => Err(Dtm2txtError::ControllerInputParseError {
                reason: ControllerInputParseError::MissingTokenError,
                line: self.line,
            })
        }
    }

    fn read_button(&self, token_opt: Option<&str>, upper: &'static str, lower: &'static str) -> Dtm2txtResult<bool> {
        let token = self.get_token(token_opt)?;

        if token == upper {
            Ok(true)
        }
        else if token == lower {
            Ok(false)
        }
        else {
            Err(Dtm2txtError::ControllerInputParseError {
                reason: ControllerInputParseError::InvalidButtonError,
                line: self.line,
            })
        }
    }

    fn read_axis(&self, token_opt: Option<&str>) -> Dtm2txtResult<u8> {
        self.get_token(token_opt)?
            .parse::<u8>()
            .map_err(|err| Dtm2txtError::ControllerInputParseError {
                reason: ControllerInputParseError::ParseIntError(err),
                line: self.line,
            })
    }

    fn read_controller_input(&mut self, line_result: Result<String, IoError>) -> Dtm2txtResult<ControllerInput> {
        let line = line_result
            .map_err(|err| Dtm2txtError::ControllerInputParseError {
                reason: ControllerInputParseError::IoError(err),
                line: self.line,
            })?;
        let mut tokens = line.split_whitespace();
        let start = self.read_button(tokens.next(), "S", "s")?;
        let a = self.read_button(tokens.next(), "A", "a")?;
        let b = self.read_button(tokens.next(), "B", "b")?;
        let x = self.read_button(tokens.next(), "X", "x")?;
        let y = self.read_button(tokens.next(), "Y", "y")?;
        let z = self.read_button(tokens.next(), "Z", "z")?;
        let up = self.read_button(tokens.next(), "U", "u")?;
        let down = self.read_button(tokens.next(), "D", "d")?;
        let left = self.read_button(tokens.next(), "L", "l")?;
        let right = self.read_button(tokens.next(), "R", "r")?;
        let l = self.read_button(tokens.next(), "LT", "lt")?;
        let r = self.read_button(tokens.next(), "RT", "rt")?;
        let l_pressure = self.read_axis(tokens.next())?;
        let r_pressure = self.read_axis(tokens.next())?;
        let analog_x = self.read_axis(tokens.next())?;
        let analog_y = self.read_axis(tokens.next())?;
        let c_x = self.read_axis(tokens.next())?;
        let c_y = self.read_axis(tokens.next())?;

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
        let mut header: DtmHeader = {
            let mut de = serde_json::Deserializer::new(JsonIoRead::new(&mut self.inner));
            Deserialize::deserialize(&mut de)?
        };

        // Add one to account for the fact that reading stops after last bracket.
        self.input_reader.line += self.inner.lines_read() + 1;

        let line_reader = BufReader::new(self.inner.inner);
        let mut controller_data = Vec::new();
        for line in line_reader.lines().skip(1) {
            controller_data.push(self.input_reader.read_controller_input(line)?);
        }

        header.input_count = controller_data.len() as u64;

        Ok(Dtm {
            header: header,
            controller_data: controller_data,
        })
    }
}