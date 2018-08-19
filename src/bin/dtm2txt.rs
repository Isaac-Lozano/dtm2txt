extern crate dtm2txt;

use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{PathBuf};

use dtm2txt::dtm::Dtm;

fn main() {
    let mut args = env::args().skip(1);
    let filename_string = args.next().unwrap();

    let filename: PathBuf = filename_string.into();
    let file = BufReader::new(File::open(&filename).unwrap());

    match filename.extension().unwrap().to_str().unwrap() {
        "dtm" => {
            let output_filename = filename.with_extension("txt");
            let dtm_bin = Dtm::read(file).unwrap();
            let output_file = BufWriter::new(File::create(output_filename).unwrap());
            dtm_bin.write(output_file).unwrap();
        }
        "txt" => {
            let output_filename = filename.with_extension("dtm");
            let dtm_txt = Dtm::read_from_text(file).unwrap();
            let output_file = BufWriter::new(File::create(output_filename).unwrap());
            dtm_txt.write_to_dtm(output_file).unwrap();
        }
        _ => panic!("File must be a txt or a dtm."),
    }
}