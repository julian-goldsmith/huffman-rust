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

pub struct HuffmanData {
    max: u16,
    bs: Bitstream
}

impl HuffmanData {
    fn write_freqs(&self, mut writer: &mut Write) -> io::Result<usize> {
        match writer.write_u16::<BigEndian>(self.max) {
            Err(err) => Err(err),
            Ok(()) => Ok(2),
        }
    }

    fn read_freqs(reader: &mut Read) -> io::Result<Option<u16>> {
        let max = match reader.read_u16::<BigEndian>() {
            Err(_) => return Ok(None),                // FIXME: errors other than EOF?
            Ok(freqs_len) => freqs_len,
        };

        Ok(Some(max))
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
        let max = match HuffmanData::read_freqs(&mut reader) {
            Err(err) => return Err(err),
            Ok(Some(freqs)) => freqs,
            Ok(None) => return Ok(None),
        };

        let bs = match Bitstream::read(reader) {
            Err(err) => return Err(err),
            Ok(bs) => bs,
        };

        Ok(Some(HuffmanData { max, bs }))
    }
}

// freqs should be sorted when we come in ehre
fn build_tree(max: u16) -> Box<Node> {
    let mut nodes: Vec<Box<Node>> = (0..max).
        map(|i| Box::new(
            Node {
                val: Some(i),
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
