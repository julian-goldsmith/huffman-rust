#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate byteorder;
extern crate time;

mod bitstream;
mod huffman;
mod rle;
mod bwt;
mod mtf;
use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;

fn open_file(path: &Path) -> File {
    let display = path.display();

    match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display,
                                                   why.description()),
        Ok(file) => file,
    }
}

fn read_file(path: &Path) -> Vec<u8> {
    let mut file = open_file(path);

    let mut bytes: Vec<u8> = Vec::new();
    match file.read_to_end(&mut bytes) {
        Ok(_) => bytes,
        Err(e) => panic!("Error reading file: {}", e),
    }
}

fn create_file(path: &Path) -> File {
    let display = path.display();

    match OpenOptions::new().write(true).create(true).truncate(true).open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display,
                                                   why.description()),
        Ok(file) => file,
    }
}

fn encode(mut write_file: &File, data: &[u8]) {
    for chunk in data.chunks(900_000) {
        let chunk_start = time::now();

        let start = time::now();
        let bwted = bwt::encode(chunk);
        println!("bwt chunk in {}", time::now() - start);

        let start = time::now();
        let mtfed = mtf::encode(&bwted);
        println!("mtf chunk in {}", time::now() - start);

        let start = time::now();
        let rled = rle::encode(&mtfed);
        println!("rle chunk in {}", time::now() - start);
        
        let start = time::now();
        let huffed = match huffman::encode(&rled) {
            Ok(huffed) => huffed,
            Err(()) => panic!("Error encoding"),
        };
        println!("huff chunk in {}", time::now() - start);

        match huffed.write(&mut write_file) {
            Ok(_) => (),
            _ => panic!("Couldn't write file"),
        };

        println!("encoded chunk in {}", time::now() - chunk_start);
    };
}

fn decode(mut read_file: &File) -> Vec<u8> {
    let mut bytes = Vec::new();

    loop {
        let hd = match huffman::HuffmanData::read(&mut read_file) {
            Ok(Some(hd)) => hd,
            Ok(None) => return bytes,
            Err(err) => panic!("Couldn't read file: {:?}", err),
        };

        let unhuffed = huffman::decode(&hd).unwrap();
        let unrled = rle::decode(&unhuffed);
        let unmtfed = mtf::decode(&unrled);
        let unbwted = bwt::decode(&unmtfed);

        // FIXME: would this take a lot of time?
        bytes.extend_from_slice(&unbwted);
    };
}

fn main() {
    let data = read_file(Path::new("../excspeed.tar"));
    let outpath = Path::new("../excspeed.tar.zzz");

    let write_file = create_file(outpath);
    encode(&write_file, &data);

    let read_file = open_file(outpath);
    let dec = decode(&read_file);

    assert_eq!(data, dec);
}
