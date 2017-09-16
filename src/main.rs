extern crate byteorder;

mod bitstream;
mod huffman;
mod lzw;
mod bwt;
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

fn encode(mut write_file: &File, data: &Vec<u8>) {
    for chunk in data.chunks(16 * 65536) {
        let bwted = bwt::encode(chunk);

        let lz_enc = lzw::encode(&bwted);
        
        let huff_enc = match huffman::encode(&lz_enc) {
            Ok(huff_enc) => huff_enc,
            Err(_) => panic!("Error encoding"),
        };

        match huff_enc.write(&mut write_file) {
            Ok(n) => println!("huff_enc len {} bytes", n),
            _ => panic!("Couldn't write file"),
        };
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

        let huff_dec = huffman::decode(&hd).unwrap();

        let lz_dec = lzw::decode(&huff_dec);

        let unbwted = bwt::decode(&lz_dec);

        bytes.extend_from_slice(&unbwted);
    };
}

fn main() {
    let data = read_file(&Path::new("../excspeed.tar.small"));
    let outpath = Path::new("../excspeed.tar.small.zzz");

    let mut write_file = create_file(outpath);
    encode(&mut write_file, &data);

    let mut read_file = open_file(outpath);
    let lz_dec = decode(&mut read_file);

    assert_eq!(data, lz_dec);
}
