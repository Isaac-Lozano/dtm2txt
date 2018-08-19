use std::fmt;

use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{self, Visitor, Unexpected};

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

bytestring!(AudioEmulator, AudioEmulatorVisitor, 16);
bytestring!(Md5, Md5Visitor, 16);
bytestring!(Reserved2, Reserved2Visitor, 12);
bytestring!(GitRevision, GitRevisionVisitor, 20);
bytestring!(Reserved3, Reserved3Visitor, 11);

#[derive(Clone, Copy, Debug)]
pub struct ControllerInput {
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

#[derive(Clone, Debug)]
pub struct Dtm {
    pub header: DtmHeader,
    pub controller_data: Vec<ControllerInput>,
}