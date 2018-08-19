use std::io::Write;

use byteorder::{WriteBytesExt, LE};

use dtm::{Dtm, DtmHeader, ControllerInput};
use error::Dtm2txtResult;

const DTM_MAGIC: &[u8; 4] = b"DTM\x1A";

trait WriteDtmExt: Write {
    fn write_str(&mut self, val: &str, len: usize) -> Dtm2txtResult<()> {
        let bytes = val.as_bytes();
        if bytes.len() > len {
            panic!("String too long.");
        }

        let mut buffer = vec![0; len];
        for (byte, buf_element) in bytes.iter().zip(buffer.iter_mut()) {
            *buf_element = *byte;
        }

        Ok(self.write_all(&buffer)?)
    }

    fn write_bool(&mut self, val: bool) -> Dtm2txtResult<()> {
        Ok(self.write_u8(if val {1} else {0})?)
    }
}

impl<W> WriteDtmExt for W where W: Write {}

pub struct DtmEncoder<W> {
    inner: W,
}

impl<W> DtmEncoder<W>
    where W: Write,
{
    pub fn new(inner: W) -> DtmEncoder<W> {
        DtmEncoder {
            inner: inner,
        }
    }

    pub fn encode(mut self, dtm: &Dtm) -> Dtm2txtResult<()> {
        self.inner.write_all(DTM_MAGIC)?;
        self.encode_header(&dtm.header)?;
        for frame in dtm.controller_data.iter() {
            self.encode_controller_input(&frame)?;
        }
        Ok(())
    }

    fn encode_header(&mut self, header: &DtmHeader) -> Dtm2txtResult<()> {
        self.inner.write_str(&header.game_id, 6)?;
        self.inner.write_bool(header.wii_game)?;
        self.inner.write_u8(header.controllers)?;
        self.inner.write_bool(header.savestate)?;
        self.inner.write_u64::<LE>(header.vi_count)?;
        self.inner.write_u64::<LE>(header.input_count)?;
        self.inner.write_u64::<LE>(header.lag_counter)?;
        self.inner.write_u64::<LE>(header.reserved1)?;
        self.inner.write_u32::<LE>(header.rerecord_count)?;
        self.inner.write_str(&header.author, 32)?;
        self.inner.write_str(&header.video_backend, 16)?;
        self.inner.write_all(&header.audio_emulator.0)?;
        self.inner.write_all(&header.md5.0)?;
        self.inner.write_u64::<LE>(header.start_time)?;
        self.inner.write_bool(header.valid_config)?;
        self.inner.write_bool(header.idle_skipping)?;
        self.inner.write_bool(header.dual_core)?;
        self.inner.write_bool(header.progressive_scan)?;
        self.inner.write_bool(header.dsp_hle)?;
        self.inner.write_bool(header.fast_disc)?;
        self.inner.write_u8(header.cpu_core)?;
        self.inner.write_bool(header.efb_access)?;
        self.inner.write_bool(header.efb_copy)?;
        self.inner.write_bool(header.efb_to_texture)?;
        self.inner.write_bool(header.efb_copy_cache)?;
        self.inner.write_bool(header.emulate_format_changes)?;
        self.inner.write_bool(header.use_xfb)?;
        self.inner.write_bool(header.use_real_xfb)?;
        self.inner.write_u8(header.memory_cards)?;
        self.inner.write_bool(header.memory_card_blank)?;
        self.inner.write_u8(header.bongos_plugged)?;
        self.inner.write_bool(header.sync_gpu)?;
        self.inner.write_bool(header.netplay)?;
        self.inner.write_bool(header.sysconf_pal60)?;
        self.inner.write_all(&header.reserved2.0)?;
        self.inner.write_str(&header.second_disc, 40)?;
        self.inner.write_all(&header.git_revision.0)?;
        self.inner.write_u32::<LE>(header.dsp_irom_hash)?;
        self.inner.write_u32::<LE>(header.dsp_coef_hash)?;
        self.inner.write_u64::<LE>(header.tick_count)?;
        self.inner.write_all(&header.reserved3.0)?;
        Ok(())
    }

    fn encode_controller_input(&mut self, input: &ControllerInput) -> Dtm2txtResult<()> {
        let mut byte1 = input.start as u8;
        byte1 |= (input.a as u8) << 1;
        byte1 |= (input.b as u8) << 2;
        byte1 |= (input.x as u8) << 3;
        byte1 |= (input.y as u8) << 4;
        byte1 |= (input.z as u8) << 5;
        byte1 |= (input.up as u8) << 6;
        byte1 |= (input.down as u8) << 7;
        self.inner.write_u8(byte1)?;

        let mut byte2 = input.left as u8;
        byte2 |= (input.right as u8) << 1;
        byte2 |= (input.l as u8) << 2;
        byte2 |= (input.r as u8) << 3;
        byte2 |= (input.change_disc as u8) << 4;
        byte2 |= (input.reset as u8) << 5;
        byte2 |= (input.controller_connected as u8) << 6;
        byte2 |= (input.reserved as u8) << 7;
        self.inner.write_u8(byte2)?;

        self.inner.write_u8(input.l_pressure)?;
        self.inner.write_u8(input.r_pressure)?;
        self.inner.write_u8(input.analog_x)?;
        self.inner.write_u8(input.analog_y)?;
        self.inner.write_u8(input.c_x)?;
        self.inner.write_u8(input.c_y)?;

        Ok(())
    }
}
