mod bitstream;
mod huffman;
mod lzw;
use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;

fn open_file(path: &String) -> File {
    let path = Path::new(path);
    let display = path.display();

    match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display,
                                                   why.description()),
        Ok(file) => file,
    }
}

fn read_file(path: &String) -> Vec<u8> {
    let mut file = open_file(path);

    let mut bytes: Vec<u8> = Vec::new();
    match file.read_to_end(&mut bytes) {
        Ok(_) => bytes,
        Err(e) => panic!("Error reading file: {}", e),
    }
}

fn create_file(path: &String) -> File {
    let path = Path::new(path);
    let display = path.display();

    match OpenOptions::new().write(true).create(true).open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display,
                                                   why.description()),
        Ok(file) => file,
    }
}

fn main() {
    let data = read_file(&String::from("../testfile.txt"));


    let lz_enc = lzw::encode(&data);

    let huff_enc = match huffman::encode(&lz_enc) {
        Ok(huff_enc) => huff_enc,
        Err(_) => panic!("Error encoding"),
    };

    let mut write_file = create_file(&String::from("../testfile.zzz"));
    match huff_enc.write(&mut write_file) {
        Ok(n) => println!("Wrote {} bytes", n),
        _ => panic!("Couldn't write file"),
    };



    let mut read_file = open_file(&String::from("../testfile.zzz"));
    let hd = match huffman::HuffmanData::read(&mut read_file) {
        Ok(hd) => hd,
        _ => panic!("Couldn't read file"),
    };

    let huff_dec = huffman::decode(&hd).unwrap();
    let lz_dec = lzw::decode(&huff_dec);

    assert_eq!(lz_enc, huff_dec);
    assert_eq!(lz_dec, data);
}
