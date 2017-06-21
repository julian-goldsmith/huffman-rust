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
pub struct Node<'a> {
    val: Option<u32>,
    left: Option<&'a Node<'a>>,
    right: Option<&'a Node<'a>>,
}

pub struct HuffmanData {
    max: u32,
    bs: Bitstream
}

impl HuffmanData {
    fn write_freqs(&self, mut writer: &mut Write) -> io::Result<usize> {
        match writer.write_u32::<BigEndian>(self.max) {
            Err(err) => Err(err),
            Ok(()) => Ok(2),
        }
    }

    fn read_freqs(reader: &mut Read) -> io::Result<Option<u32>> {
        let max = match reader.read_u32::<BigEndian>() {
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

fn build_tree<'a>(max: u32, nodes: &'a mut Vec<Box<Node<'a>>>) -> &'a Node<'a> {
    let mut nodes_ref: Vec<&'a Node<'a>> = Vec::new();

    for i in 0..max {
        nodes.push(
            Box::new(Node {
                val: Some(i),
                left: None,
                right: None,
            }));

        nodes_ref.push(&nodes[nodes.len()]);
    }

    loop {
        let lo = nodes_ref.pop();
        let ro = nodes_ref.pop();

        match (lo, ro) {
            (Some(left), None) => break,
            (Some(left), Some(right)) => {
                let node = Node {
                    val: None,
                    left: Some(left),
                    right: Some(right),
                };

                nodes.push(node);
                nodes_ref.insert(0, &nodes[nodes.len()]);
            },
            _ => panic!("Must have nodes to build_tree"),
        };
    };

    nodes_ref[0]
}
