use std::fmt::{self, Display};
use std::io::{self, Read, Write, BufReader, BufRead};
use std::str::FromStr;

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{self, Visitor, Unexpected};
use serde_json;
use serde_json::de::IoRead as JsonIoRead;

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

macro_rules! read_input {
    ($token:expr, $upper:expr, $lower:expr) => {
        {
            match $token {
                $upper => true,
                $lower => false,
                _ => return Err(()),
            }
        }
    };
}

macro_rules! bytestring {
    ($name:ident, $visitor_name: ident, $length:expr) => {
        #[derive(Clone, Copy, Debug)]
        pub struct $name(pub [u8; $length]);

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer,
            {
                let mut bytestring = String::new();
                for val in self.0.iter() {
                    bytestring += &format!("{:02X}", val);
                }

                serializer.serialize_str(&bytestring)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: Deserializer<'de>,
            {
                deserializer.deserialize_str($visitor_name)
            }
        }

        struct $visitor_name;

        impl<'de> Visitor<'de> for $visitor_name {
            type Value = $name;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a {}-byte long all-caps hex string", $length)
            }

            fn visit_str<E>(self, value: &str) ->  Result<Self::Value, E>
                where E: de::Error,
            {
                let mut buffer = [0; $length];

                if value.len() != $length * 2 {
                    return Err(de::Error::invalid_type(Unexpected::Other("string of invalid length"), &self));
                }

                let mut value_iter = value.chars();
                let mut idx = 0;
                while let Some(high_char) = value_iter.next() {
                    // Should be guaranteed because previous check.
                    let low_char = value_iter.next().unwrap();

                    let high_nibble = if high_char <= 'F' && high_char >= 'A' {
                        high_char as u8 - 'A' as u8 + 10
                    }
                    else if high_char <= '9' && high_char >= '0' {
                        high_char as u8 - '0' as u8
                    }
                    else {
                        return Err(de::Error::invalid_type(Unexpected::Other("invalid character"), &self));
                    };

                    let low_nibble = if low_char <= 'F' && low_char >= 'A' {
                        low_char as u8 - 'A' as u8 + 10
                    }
                    else if low_char <= '9' && low_char >= '0' {
                        low_char as u8 - '0' as u8
                    }
                    else {
                        return Err(de::Error::invalid_type(Unexpected::Other("invalid character"), &self));
                    };

                    buffer[idx] = (high_nibble << 4) | low_nibble;

                    idx += 1;
                }

                Ok($name(buffer))
            }
        }
    };
}

const DTM_MAGIC: &[u8; 4] = b"DTM\x1A";

const START_MASK: u8 = 0x01;
const A_MASK: u8 = 0x02;
const B_MASK: u8 = 0x04;
const X_MASK: u8 = 0x08;
const Y_MASK: u8 = 0x10;
const Z_MASK: u8 = 0x20;
const UP_MASK: u8 = 0x40;
const DOWN_MASK: u8 = 0x80;
const LEFT_MASK: u8 = 0x01;
const RIGHT_MASK: u8 = 0x02;
const L_MASK: u8 = 0x04;
const R_MASK: u8 = 0x08;
const CHANGE_DISC_MASK: u8 = 0x10;
const RESET_MASK: u8 = 0x20;
const CONTROLLER_CONNECTED_MASK: u8 = 0x40;
const RESERVED_MASK: u8 = 0x80;

trait ReadDtmExt: Read {
    fn read_string(&mut self, len: usize) -> io::Result<String> {
        let mut buffer = vec![0; len];
        self.read_exact(&mut buffer)?;

        while let Some(0) = buffer.last() {
            buffer.pop();
        }

        Ok(String::from_utf8(buffer).unwrap())
    }

    fn read_bool(&mut self) -> io::Result<bool> {
        self.read_u8().map(|val| val != 0)
    }
}

impl<R> ReadDtmExt for R where R: Read {}

trait WriteDtmExt: Write {
    fn write_str(&mut self, val: &str, len: usize) -> io::Result<()> {
        let bytes = val.as_bytes();
        if bytes.len() > len {
            panic!("String too long.");
        }

        let mut buffer = vec![0; len];
        for (byte, buf_element) in bytes.iter().zip(buffer.iter_mut()) {
            *buf_element = *byte;
        }

        self.write_all(&buffer)
    }

    fn write_bool(&mut self, val: bool) -> io::Result<()> {
        self.write_u8(if val {1} else {0})
    }
}

impl<W> WriteDtmExt for W where W: Write {}

bytestring!(AudioEmulator, AudioEmulatorVisitor, 16);
bytestring!(Md5, Md5Visitor, 16);
bytestring!(Reserved2, Reserved2Visitor, 12);
bytestring!(GitRevision, GitRevisionVisitor, 20);
bytestring!(Reserved3, Reserved3Visitor, 11);

#[derive(Clone, Copy, Debug)]
pub struct ControllerFrame {
    pub start: bool,
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
    pub z: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub l: bool,
    pub r: bool,
    pub change_disc: bool,
    pub reset: bool,
    pub controller_connected: bool,
    pub reserved: bool,
    pub l_pressure: u8,
    pub r_pressure: u8,
    pub analog_x: u8,
    pub analog_y: u8,
    pub c_x: u8,
    pub c_y: u8,
}

impl ControllerFrame {
    fn read<R>(mut reader: R) -> io::Result<ControllerFrame>
        where R: Read,
    {
        let mut bytes = [0; 2];
        reader.read_exact(&mut bytes)?;
        let l_pressure = reader.read_u8()?;
        let r_pressure = reader.read_u8()?;
        let analog_x = reader.read_u8()?;
        let analog_y = reader.read_u8()?;
        let c_x = reader.read_u8()?;
        let c_y = reader.read_u8()?;

        Ok(ControllerFrame {
            start: bytes[0] & START_MASK != 0,
            a: bytes[0] & A_MASK != 0,
            b: bytes[0] & B_MASK != 0,
            x: bytes[0] & X_MASK != 0,
            y: bytes[0] & Y_MASK != 0,
            z: bytes[0] & Z_MASK != 0,
            up: bytes[0] & UP_MASK != 0,
            down: bytes[0] & DOWN_MASK != 0,
            left: bytes[1] & LEFT_MASK != 0,
            right: bytes[1] & RIGHT_MASK != 0,
            l: bytes[1] & L_MASK != 0,
            r: bytes[1] & R_MASK != 0,
            change_disc: bytes[1] & CHANGE_DISC_MASK != 0,
            reset: bytes[1] & RESET_MASK != 0,
            controller_connected: bytes[1] & CONTROLLER_CONNECTED_MASK != 0,
            reserved: bytes[1] & RESERVED_MASK != 0,
            l_pressure: l_pressure,
            r_pressure: r_pressure,
            analog_x: analog_x,
            analog_y: analog_y,
            c_x: c_x,
            c_y: c_y,
        })
    }

    fn write_to_dtm<W>(&self, mut writer: W) -> io::Result<()>
        where W: Write,
    {
        let mut byte1 = self.start as u8;
        byte1 |= (self.a as u8) << 1;
        byte1 |= (self.b as u8) << 2;
        byte1 |= (self.x as u8) << 3;
        byte1 |= (self.y as u8) << 4;
        byte1 |= (self.z as u8) << 5;
        byte1 |= (self.up as u8) << 6;
        byte1 |= (self.down as u8) << 7;
        writer.write_u8(byte1)?;

        let mut byte2 = self.left as u8;
        byte2 |= (self.right as u8) << 1;
        byte2 |= (self.l as u8) << 2;
        byte2 |= (self.r as u8) << 3;
        byte2 |= (self.change_disc as u8) << 4;
        byte2 |= (self.reset as u8) << 5;
        byte2 |= (self.controller_connected as u8) << 6;
        byte2 |= (self.reserved as u8) << 7;
        writer.write_u8(byte2)?;

        writer.write_u8(self.l_pressure)?;
        writer.write_u8(self.r_pressure)?;
        writer.write_u8(self.analog_x)?;
        writer.write_u8(self.analog_y)?;
        writer.write_u8(self.c_x)?;
        writer.write_u8(self.c_y)?;

        Ok(())
    }
}

impl Display for ControllerFrame {
    // S A B X Y Z U D L R LT 0 0 0 0 0 0 [CD RST CC RSV]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let mut line = String::new();
        format_input!(line, self.start, "S ", "s ");
        format_input!(line, self.a, "A ", "a ");
        format_input!(line, self.b, "B ", "b ");
        format_input!(line, self.x, "X ", "x ");
        format_input!(line, self.y, "Y ", "y ");
        format_input!(line, self.z, "Z ", "z ");
        format_input!(line, self.up, "U ", "u ");
        format_input!(line, self.down, "D ", "d ");
        format_input!(line, self.left, "L ", "l ");
        format_input!(line, self.right, "R ", "r ");
        format_input!(line, self.l, "LT ", "lt ");
        format_input!(line, self.r, "RT ", "rt ");
        line += &(format!("{:3} ", self.l_pressure));
        line += &(format!("{:3} ", self.r_pressure));
        line += &(format!("{:3} ", self.analog_x));
        line += &(format!("{:3} ", self.analog_y));
        line += &(format!("{:3} ", self.c_x));
        line += &(format!("{:3}", self.c_y));
        format_input!(line, self.change_disc, " CD", "");
        format_input!(line, self.reset, " RST", "");
        format_input!(line, self.controller_connected, " CC", "");
        format_input!(line, self.reserved, " RSV", "");

        formatter.write_str(&line)
    }
}

impl FromStr for ControllerFrame {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = s.split_whitespace();
        let start = read_input!(tokens.next().unwrap(), "S", "s");
        let a = read_input!(tokens.next().unwrap(), "A", "a");
        let b = read_input!(tokens.next().unwrap(), "B", "b");
        let x = read_input!(tokens.next().unwrap(), "X", "x");
        let y = read_input!(tokens.next().unwrap(), "Y", "y");
        let z = read_input!(tokens.next().unwrap(), "Z", "z");
        let up = read_input!(tokens.next().unwrap(), "U", "u");
        let down = read_input!(tokens.next().unwrap(), "D", "d");
        let left = read_input!(tokens.next().unwrap(), "L", "l");
        let right = read_input!(tokens.next().unwrap(), "R", "r");
        let l = read_input!(tokens.next().unwrap(), "LT", "lt");
        let r = read_input!(tokens.next().unwrap(), "RT", "rt");
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

        Ok(ControllerFrame {
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DtmHeader {
    pub game_id: String,
    pub wii_game: bool,
    pub controllers: u8,
    pub savestate: bool,
    pub vi_count: u64,
    pub input_count: u64,
    pub lag_counter: u64,
    pub reserved1: u64,
    pub rerecord_count: u32,
    pub author: String,
    pub video_backend: String,
    pub audio_emulator: AudioEmulator,
    pub md5: Md5,
    pub start_time: u64,
    pub valid_config: bool,
    pub idle_skipping: bool,
    pub dual_core: bool,
    pub progressive_scan: bool,
    pub dsp_hle: bool,
    pub fast_disc: bool,
    pub cpu_core: u8,
    pub efb_access: bool,
    pub efb_copy: bool,
    pub efb_to_texture: bool,
    pub efb_copy_cache: bool,
    pub emulate_format_changes: bool,
    pub use_xfb: bool,
    pub use_real_xfb: bool,
    pub memory_cards: u8,
    pub memory_card_blank: bool,
    pub bongos_plugged: u8,
    pub sync_gpu: bool,
    pub netplay: bool,
    pub sysconf_pal60: bool,
    pub reserved2: Reserved2,
    pub second_disc: String,
    pub git_revision: GitRevision,
    pub dsp_irom_hash: u32,
    pub dsp_coef_hash: u32,
    pub tick_count: u64,
    pub reserved3: Reserved3,
}

impl DtmHeader {
    fn read<R>(mut reader: R) -> io::Result<DtmHeader>
        where R: Read,
    {
        let mut magic_buffer = [0; 4];
        reader.read_exact(&mut magic_buffer)?;
        if magic_buffer != *DTM_MAGIC {
            panic!("Bad magic value");
        }

        let game_id = reader.read_string(6)?;
        let wii_game = reader.read_bool()?;
        let controllers = reader.read_u8()?;
        let savestate = reader.read_bool()?;
        let vi_count = reader.read_u64::<LE>()?;
        let input_count = reader.read_u64::<LE>()?;
        let lag_counter = reader.read_u64::<LE>()?;
        let reserved1 = reader.read_u64::<LE>()?;
        let rerecord_count = reader.read_u32::<LE>()?;
        let author = reader.read_string(32)?;
        let video_backend = reader.read_string(16)?;
        let mut audio_emulator_buffer = [0; 16];
        reader.read_exact(&mut audio_emulator_buffer)?;
        let audio_emulator = AudioEmulator(audio_emulator_buffer);
        let mut md5_buffer = [0; 16];
        reader.read_exact(&mut md5_buffer)?;
        let md5 = Md5(md5_buffer);
        let start_time = reader.read_u64::<LE>()?;
        let valid_config = reader.read_bool()?;
        let idle_skipping = reader.read_bool()?;
        let dual_core = reader.read_bool()?;
        let progressive_scan = reader.read_bool()?;
        let dsp_hle = reader.read_bool()?;
        let fast_disc = reader.read_bool()?;
        let cpu_core = reader.read_u8()?;
        let efb_access = reader.read_bool()?;
        let efb_copy = reader.read_bool()?;
        let efb_to_texture = reader.read_bool()?;
        let efb_copy_cache = reader.read_bool()?;
        let emulate_format_changes = reader.read_bool()?;
        let use_xfb = reader.read_bool()?;
        let use_real_xfb = reader.read_bool()?;
        let memory_cards = reader.read_u8()?;
        let memory_card_blank = reader.read_bool()?;
        let bongos_plugged = reader.read_u8()?;
        let sync_gpu = reader.read_bool()?;
        let netplay = reader.read_bool()?;
        let sysconf_pal60 = reader.read_bool()?;
        let mut reserved2_buffer = [0; 12];
        reader.read_exact(&mut reserved2_buffer)?;
        let reserved2 = Reserved2(reserved2_buffer);
        let second_disc = reader.read_string(40)?;
        let mut git_revision_buffer = [0; 20];
        reader.read_exact(&mut git_revision_buffer)?;
        let git_revision = GitRevision(git_revision_buffer);
        let dsp_irom_hash = reader.read_u32::<LE>()?;
        let dsp_coef_hash = reader.read_u32::<LE>()?;
        let tick_count = reader.read_u64::<LE>()?;
        let mut reserved3_buffer = [0; 11];
        reader.read_exact(&mut reserved3_buffer)?;
        let reserved3 = Reserved3(reserved3_buffer);

        Ok(DtmHeader {
            game_id: game_id,
            wii_game: wii_game,
            controllers: controllers,
            savestate: savestate,
            vi_count: vi_count,
            input_count: input_count,
            lag_counter: lag_counter,
            reserved1: reserved1,
            rerecord_count: rerecord_count,
            author: author,
            video_backend: video_backend,
            audio_emulator: audio_emulator,
            md5: md5,
            start_time: start_time,
            valid_config: valid_config,
            idle_skipping: idle_skipping,
            dual_core: dual_core,
            progressive_scan: progressive_scan,
            dsp_hle: dsp_hle,
            fast_disc: fast_disc,
            cpu_core: cpu_core,
            efb_access: efb_access,
            efb_copy: efb_copy,
            efb_to_texture: efb_to_texture,
            efb_copy_cache: efb_copy_cache,
            emulate_format_changes: emulate_format_changes,
            use_xfb: use_xfb,
            use_real_xfb: use_real_xfb,
            memory_cards: memory_cards,
            memory_card_blank: memory_card_blank,
            bongos_plugged: bongos_plugged,
            sync_gpu: sync_gpu,
            netplay: netplay,
            sysconf_pal60: sysconf_pal60,
            reserved2: reserved2,
            second_disc: second_disc,
            git_revision: git_revision,
            dsp_irom_hash: dsp_irom_hash,
            dsp_coef_hash: dsp_coef_hash,
            tick_count: tick_count,
            reserved3: reserved3,
        })
    }

    fn write_to_dtm<W>(&self, mut writer: W) -> io::Result<()>
        where W: Write,
    {
        writer.write_str(&self.game_id, 6)?;
        writer.write_bool(self.wii_game)?;
        writer.write_u8(self.controllers)?;
        writer.write_bool(self.savestate)?;
        writer.write_u64::<LE>(self.vi_count)?;
        writer.write_u64::<LE>(self.input_count)?;
        writer.write_u64::<LE>(self.lag_counter)?;
        writer.write_u64::<LE>(self.reserved1)?;
        writer.write_u32::<LE>(self.rerecord_count)?;
        writer.write_str(&self.author, 32)?;
        writer.write_str(&self.video_backend, 16)?;
        writer.write_all(&self.audio_emulator.0)?;
        writer.write_all(&self.md5.0)?;
        writer.write_u64::<LE>(self.start_time)?;
        writer.write_bool(self.valid_config)?;
        writer.write_bool(self.idle_skipping)?;
        writer.write_bool(self.dual_core)?;
        writer.write_bool(self.progressive_scan)?;
        writer.write_bool(self.dsp_hle)?;
        writer.write_bool(self.fast_disc)?;
        writer.write_u8(self.cpu_core)?;
        writer.write_bool(self.efb_access)?;
        writer.write_bool(self.efb_copy)?;
        writer.write_bool(self.efb_to_texture)?;
        writer.write_bool(self.efb_copy_cache)?;
        writer.write_bool(self.emulate_format_changes)?;
        writer.write_bool(self.use_xfb)?;
        writer.write_bool(self.use_real_xfb)?;
        writer.write_u8(self.memory_cards)?;
        writer.write_bool(self.memory_card_blank)?;
        writer.write_u8(self.bongos_plugged)?;
        writer.write_bool(self.sync_gpu)?;
        writer.write_bool(self.netplay)?;
        writer.write_bool(self.sysconf_pal60)?;
        writer.write_all(&self.reserved2.0)?;
        writer.write_str(&self.second_disc, 40)?;
        writer.write_all(&self.git_revision.0)?;
        writer.write_u32::<LE>(self.dsp_irom_hash)?;
        writer.write_u32::<LE>(self.dsp_coef_hash)?;
        writer.write_u64::<LE>(self.tick_count)?;
        writer.write_all(&self.reserved3.0)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Dtm {
    pub header: DtmHeader,
    pub controller_data: Vec<ControllerFrame>,
}

impl Dtm {
    pub fn read<R>(mut reader: R) -> io::Result<Dtm>
        where R: Read,
    {
        let header = DtmHeader::read(&mut reader)?;

        let mut controller_data = Vec::new();
        for _ in 0..header.input_count {
            controller_data.push(ControllerFrame::read(&mut reader)?);
        }

        Ok(Dtm {
            header: header,
            controller_data: controller_data,
        })
    }

    pub fn write<W>(&self, mut writer: W) -> io::Result<()>
        where W: Write,
    {
        serde_json::to_writer_pretty(&mut writer, &self.header)?;
        writeln!(writer)?;
        for frame in self.controller_data.iter() {
            writeln!(writer, "{}", frame)?;
        }
        Ok(())
    }

    pub fn read_from_text<R>(mut reader: R) -> io::Result<Dtm>
        where R: Read,
    {
        let header = {
            let mut de = serde_json::Deserializer::new(JsonIoRead::new(&mut reader));
            Deserialize::deserialize(&mut de)?
        };

        let line_reader = BufReader::new(reader);
        let mut controller_data = Vec::new();
        for line in line_reader.lines().skip(1) {
            controller_data.push(ControllerFrame::from_str(&line.unwrap()).unwrap());
        }

        Ok(Dtm {
            header: header,
            controller_data: controller_data,
        })
    }

    pub fn write_to_dtm<W>(&self, mut writer: W) -> io::Result<()>
        where W: Write,
    {
        writer.write_all(DTM_MAGIC)?;
        self.header.write_to_dtm(&mut writer)?;
        for frame in self.controller_data.iter() {
            frame.write_to_dtm(&mut writer)?;
        }
        Ok(())
    }
}