extern crate dtm2txt;

use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{PathBuf};

use dtm2txt::encoder::text_encoder::TextEncoder;
use dtm2txt::encoder::dtm_encoder::DtmEncoder;
use dtm2txt::decoder::text_decoder::TextDecoder;
use dtm2txt::decoder::dtm_decoder::DtmDecoder;

fn main() {
    let mut args = env::args().skip(1);
    let filename_string = args.next().unwrap();

    let filename: PathBuf = filename_string.into();
    let file = BufReader::new(File::open(&filename).unwrap());

    match filename.extension().unwrap().to_str().unwrap() {
        "dtm" => {
            let decoder = DtmDecoder::new(file);
            let dtm_bin = decoder.decode().unwrap();

            let output_filename = filename.with_extension("txt");
            let output_file = BufWriter::new(File::create(output_filename).unwrap());

            let encoder = TextEncoder::new(output_file);
            encoder.encode(&dtm_bin).unwrap();
        }
        "txt" => {
            let decoder = TextDecoder::new(file);
            let dtm_txt = decoder.decode().unwrap();

            let output_filename = filename.with_extension("dtm");
            let output_file = BufWriter::new(File::create(output_filename).unwrap());

            let encoder = DtmEncoder::new(output_file);
            encoder.encode(&dtm_txt).unwrap();
        }
        _ => panic!("File must be a txt or a dtm."),
    }
}