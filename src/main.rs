mod bitstream;
mod huffman;
mod lzw;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use bitstream::Bitstream;

fn read_file(path: &String) -> Vec<u8> {
    let path = Path::new(path);
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display,
                                                   why.description()),
        Ok(file) => file,
    };

    let mut bytes: Vec<u8> = Vec::new();
    match file.read_to_end(&mut bytes) {
        Ok(_) => bytes,
        Err(e) => panic!("Error reading file: {}", e),
    }
}

fn main() {
    let data = read_file(&String::from("../testfile.txt"));
    let enc = lzw::encode(&data);
    println!("enc {:?}", enc);

    /*
    let root = huffman::build_tree(&data);

    let calc = match huffman::precalc_bitstreams(&root) {
        Ok(calc) => calc,
        Err(_) => panic!("Couldn't precalc bitstream"),
    };

    let enc = data.iter().
        map(|c| &calc[*c as usize]).
        fold(Bitstream::new(), |acc, x| acc + x);

    let dec = huffman::decode_bitstream(&root, enc);
    println!("{:?}", dec)
    */
}
