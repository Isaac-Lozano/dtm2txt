extern crate dtm2txt;

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{PathBuf};
use std::process;

use dtm2txt::encoder::text_encoder::TextEncoder;
use dtm2txt::encoder::dtm_encoder::DtmEncoder;
use dtm2txt::decoder::text_decoder::TextDecoder;
use dtm2txt::decoder::dtm_decoder::DtmDecoder;

trait UnwrapOrBarfExt<T> {
    fn unwrap_or_barf(self, err_str: &str) -> T;
}

impl<T, E> UnwrapOrBarfExt<T> for Result<T, E>
    where E: Error
{
    fn unwrap_or_barf(self, err_desc: &str) -> T {
        self.unwrap_or_else(|err| {
            let err_string = format!("{}: {}", err_desc, err);
            barf(&err_string);
        })
    }
}

impl<T> UnwrapOrBarfExt<T> for Option<T> {
    fn unwrap_or_barf(self, err_desc: &str) -> T {
        self.unwrap_or_else(|| {
            let err_string = format!("{}", err_desc);
            barf(&err_string);
        })
    }
}

fn barf(message: &str) -> ! {
    println!("Error: {}", message);
    process::exit(1);
}

fn main() {
    let mut args = env::args().skip(1);
    let filename_string = match args.next() {
        Some(value) => value,
        None => {
            println!("dtm2txt (version {})", env!("CARGO_PKG_VERSION"));
            println!("by OnVar");
            return;
        }
    };

    let filename: PathBuf = filename_string.into();
    let output_opt = args.next();
    let file = BufReader::new(File::open(&filename).unwrap_or_barf("Could not file"));

    match filename.extension().unwrap_or_barf("Filename has no extension").to_str().unwrap_or_barf("Error processing filename") {
        "dtm" => {
            let decoder = DtmDecoder::new(file);
            let dtm_bin = decoder.decode().unwrap_or_barf("Could not make dtm decoder");

            let output_filename = output_opt
                .map(|val| val.into())
                .unwrap_or(filename.with_extension("txt"));
            let output_file = BufWriter::new(File::create(output_filename).unwrap_or_barf("Could not create file"));

            let encoder = TextEncoder::new(output_file);
            encoder.encode(&dtm_bin).unwrap_or_barf("Could not encode dtm");

            println!("Successfully converted from dtm to txt.")
        }
        "txt" => {
            let decoder = TextDecoder::new(file);
            let dtm_txt = decoder.decode().unwrap_or_barf("Could not make text decoder");

            let output_filename = output_opt
                .map(|val| val.into())
                .unwrap_or(filename.with_extension("dtm"));
            let output_file = BufWriter::new(File::create(output_filename).unwrap_or_barf("Could not create file"));

            let encoder = DtmEncoder::new(output_file);
            encoder.encode(&dtm_txt).unwrap_or_barf("Could not encode dtm");
            println!("Successfully converted from txt to dtm.")
        }
        _ => barf("File must be a txt or a dtm."),
    }
}