mod decode;
mod encode;
use bitstream::Bitstream;
use std::io;
use std::io::Write;
use std::io::Read;
use std::rc::Rc;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

pub use self::encode::encode;
pub use self::decode::decode;

#[derive(Debug)]
pub enum Node {
    Leaf(u32),
    Tree { left: Rc<Node>, right: Rc<Node> },
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

fn build_tree(max: u32) -> Rc<Node> {
    let mut nodes: Vec<Rc<Node>> = (0..max).
        map(|i| Rc::new(Node::Leaf(i))).
        collect();

    loop {
        let lo = nodes.pop();
        let ro = nodes.pop();

        match (lo, ro) {
            (Some(left), None) => return left,
            (Some(left), Some(right)) => {
                let node = Rc::new(Node::Tree { left, right });

                nodes.insert(0, node);
            },
            _ => panic!("Must have nodes to build_tree"),
        };
    };
}
