use std::io::Read;

use byteorder::{ReadBytesExt, LE};
use dtm::{Dtm, DtmHeader, ControllerInput, AudioEmulator, Md5, Reserved2, GitRevision, Reserved3};
use error::Dtm2txtResult;

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
    fn read_string(&mut self, len: usize) -> Dtm2txtResult<String> {
        let mut buffer = vec![0; len];
        self.read_exact(&mut buffer)?;

        while let Some(0) = buffer.last() {
            buffer.pop();
        }

        Ok(String::from_utf8(buffer)?)
    }

    fn read_bool(&mut self) -> Dtm2txtResult<bool> {
        Ok(self.read_u8().map(|val| val != 0)?)
    }
}

impl<R> ReadDtmExt for R where R: Read {}

pub struct DtmDecoder<R> {
    inner: R,
}

impl<R> DtmDecoder<R>
    where R: Read,
{
    pub fn new(inner: R) -> DtmDecoder<R> {
        DtmDecoder {
            inner: inner,
        }
    }

    pub fn decode(mut self) -> Dtm2txtResult<Dtm> {
        let header = self.decode_header()?;

        let mut controller_data = Vec::new();
        for _ in 0..header.input_count {
            controller_data.push(self.decode_controller_input()?);
        }

        Ok(Dtm {
            header: header,
            controller_data: controller_data,
        })
    }

    fn decode_header(&mut self) -> Dtm2txtResult<DtmHeader> {
        let mut magic_buffer = [0; 4];
        self.inner.read_exact(&mut magic_buffer)?;
        if magic_buffer != *DTM_MAGIC {
            panic!("Bad magic value");
        }

        let game_id = self.inner.read_string(6)?;
        let wii_game = self.inner.read_bool()?;
        let controllers = self.inner.read_u8()?;
        let savestate = self.inner.read_bool()?;
        let vi_count = self.inner.read_u64::<LE>()?;
        let input_count = self.inner.read_u64::<LE>()?;
        let lag_counter = self.inner.read_u64::<LE>()?;
        let reserved1 = self.inner.read_u64::<LE>()?;
        let rerecord_count = self.inner.read_u32::<LE>()?;
        let author = self.inner.read_string(32)?;
        let video_backend = self.inner.read_string(16)?;
        let mut audio_emulator_buffer = [0; 16];
        self.inner.read_exact(&mut audio_emulator_buffer)?;
        let audio_emulator = AudioEmulator(audio_emulator_buffer);
        let mut md5_buffer = [0; 16];
        self.inner.read_exact(&mut md5_buffer)?;
        let md5 = Md5(md5_buffer);
        let start_time = self.inner.read_u64::<LE>()?;
        let valid_config = self.inner.read_bool()?;
        let idle_skipping = self.inner.read_bool()?;
        let dual_core = self.inner.read_bool()?;
        let progressive_scan = self.inner.read_bool()?;
        let dsp_hle = self.inner.read_bool()?;
        let fast_disc = self.inner.read_bool()?;
        let cpu_core = self.inner.read_u8()?;
        let efb_access = self.inner.read_bool()?;
        let efb_copy = self.inner.read_bool()?;
        let efb_to_texture = self.inner.read_bool()?;
        let efb_copy_cache = self.inner.read_bool()?;
        let emulate_format_changes = self.inner.read_bool()?;
        let use_xfb = self.inner.read_bool()?;
        let use_real_xfb = self.inner.read_bool()?;
        let memory_cards = self.inner.read_u8()?;
        let memory_card_blank = self.inner.read_bool()?;
        let bongos_plugged = self.inner.read_u8()?;
        let sync_gpu = self.inner.read_bool()?;
        let netplay = self.inner.read_bool()?;
        let sysconf_pal60 = self.inner.read_bool()?;
        let mut reserved2_buffer = [0; 12];
        self.inner.read_exact(&mut reserved2_buffer)?;
        let reserved2 = Reserved2(reserved2_buffer);
        let second_disc = self.inner.read_string(40)?;
        let mut git_revision_buffer = [0; 20];
        self.inner.read_exact(&mut git_revision_buffer)?;
        let git_revision = GitRevision(git_revision_buffer);
        let dsp_irom_hash = self.inner.read_u32::<LE>()?;
        let dsp_coef_hash = self.inner.read_u32::<LE>()?;
        let tick_count = self.inner.read_u64::<LE>()?;
        let mut reserved3_buffer = [0; 11];
        self.inner.read_exact(&mut reserved3_buffer)?;
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

    fn decode_controller_input(&mut self) -> Dtm2txtResult<ControllerInput> {
        let mut bytes = [0; 2];
        self.inner.read_exact(&mut bytes)?;
        let l_pressure = self.inner.read_u8()?;
        let r_pressure = self.inner.read_u8()?;
        let analog_x = self.inner.read_u8()?;
        let analog_y = self.inner.read_u8()?;
        let c_x = self.inner.read_u8()?;
        let c_y = self.inner.read_u8()?;

        Ok(ControllerInput {
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
}