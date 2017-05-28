mod decode;
mod encode;
use bitstream::Bitstream;
use std::io;
use std::io::Write;
use std::io::Read;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

pub use self::encode::encode;
pub use self::decode::decode;

#[derive(Debug)]
pub struct Node {
    val: Option<u16>,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

#[derive(Debug,Clone,Copy)]
pub struct Freq {
    val: u16,
    count: u16,
}

pub struct HuffmanData {
    freqs: Vec<Freq>,
    bs: Bitstream
}

impl HuffmanData {
    fn write_freqs(&self, mut writer: &mut Write) -> io::Result<usize> {
        let freqs_filtered: Vec<&Freq> = self.freqs.iter().
            filter(|freq| freq.count > 0).
            collect();

        println!("Writing {} freqs", freqs_filtered.len());
        writer.write_u16::<BigEndian>(freqs_filtered.len() as u16)?;

        for freq in freqs_filtered.iter() {
            writer.write_u16::<BigEndian>(freq.val)?;   // FIXME: handle errors
        }

        let bytes_out = 2 + (freqs_filtered.len() * 2);
        Ok(bytes_out)
    }

    fn read_freqs(reader: &mut Read) -> io::Result<Option<Vec<Freq>>> {
        let freqs_len = match reader.read_u16::<BigEndian>() {
            Err(_) => return Ok(None),                // FIXME: errors other than EOF?
            Ok(freqs_len) => freqs_len as usize,
        };

        println!("Reading {} freqs", freqs_len);

        let mut freqs: Vec<Freq> = Vec::with_capacity(freqs_len);
        for _ in 0..freqs_len {
            freqs.push(Freq {
                val: reader.read_u16::<BigEndian>()?,   // FIXME: error handling
                count: 0,       // FIXME: kind of hacky
            });
        };

        Ok(Some(freqs))
    }

    pub fn write(&self, mut writer: &mut Write) -> io::Result<usize> {
        let bytes_out = 
            match self.write_freqs(writer) {
                Err(err) => return Err(err),
                Ok(nb) => nb,
            }
            +
            match self.bs.write(&mut writer) {
                Err(err) => return Err(err),
                Ok(nb) => nb,
            };
        Ok(bytes_out)
    }

    pub fn read(mut reader: &mut Read) -> io::Result<Option<HuffmanData>> {
        println!("Reading freqs");
        let freqs = match HuffmanData::read_freqs(&mut reader) {
            Err(err) => return Err(err),
            Ok(Some(freqs)) => freqs,
            Ok(None) => return Ok(None),
        };

        println!("Reading bitstream");
        let bs = match Bitstream::read(reader) {
            Err(err) => return Err(err),
            Ok(bs) => bs,
        };

        Ok(Some(HuffmanData { freqs, bs }))
    }
}

// freqs should be sorted when we come in ehre
fn build_tree(freqs: &Vec<Freq>) -> Box<Node> {
    let mut nodes: Vec<Box<Node>> = freqs.iter().
        map(|freq| Box::new(
            Node {
                val: Some(freq.val),
                left: None,
                right: None,
            })).
        collect();

    loop {
        let lo = nodes.pop();
        let ro = nodes.pop();

        match (lo, ro) {
            (Some(left), None) => return left,
            (Some(left), Some(right)) => {
                let node = Box::new(Node {
                    val: None,
                    left: Some(left),
                    right: Some(right),
                });

                nodes.insert(0, node);
            },
            _ => panic!("Must have nodes to build_tree"),
        };
    };
}
