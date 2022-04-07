use std::fs::File;
use std::io;
use std::io::BufReader;

use clap::{arg, Command};

use brotlic::{CompressorWriter, DecompressorReader};

fn main() {
    let matches = Command::new("br")
        .version("0.1")
        .about("File brotli compression tool")
        .arg(
            arg!([FILE] "The file to compress")
                .required(true))
        .arg(arg!(-d --decompress))
        .get_matches();

    let path = matches.value_of("FILE").expect("supplied by clap");
    let compress = !matches.is_present("decompress");

    if compress {
        let mut input_file = File::open(path).expect("failed to open input file");

        let mut output_file = {
            let write_path = [path, ".br"].concat();

            CompressorWriter::new(
                File::create(write_path).expect("failed to create output file")
            )
        };

        io::copy(&mut input_file, &mut output_file).expect("io error");
    } else {
        let mut input_file = {
            DecompressorReader::new(
                BufReader::new(
                    File::open(path).expect("failed to read input file")
                )
            )
        };

        let mut output_file = {
            let write_path = path
                .strip_suffix(".br")
                .expect("not a a valid .br file");

            File::create(write_path).expect("failed to create output file")
        };

        io::copy(&mut input_file, &mut output_file).expect("io error");
    }
}
