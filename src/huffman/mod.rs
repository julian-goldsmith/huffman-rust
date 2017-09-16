mod decode;
mod encode;
use bitstream::Bitstream;
use std::io;
use std::io::Write;
use std::io::Read;
use std::rc::Rc;
use byteorder::{ReadBytesExt, WriteBytesExt};

pub use self::encode::encode;
pub use self::decode::decode;

#[derive(Debug)]
pub enum Node {
    Leaf(u8),
    Tree { left: Rc<Node>, right: Rc<Node> },
}

pub struct HuffmanData {
    freqs: Box<[u8]>,
    bs: Bitstream,
}

impl HuffmanData {
    fn write_freqs(&self, mut writer: &mut Write) -> io::Result<usize> {
        match writer.write(&self.freqs) {
            Err(err) => Err(err),
            Ok(count) => Ok(count),
        }
    }

    fn read_freqs(reader: &mut Read) -> io::Result<Option<Box<[u8]>>> {
        let mut freqs = Box::new([0; 256]);

        match reader.read(&mut freqs[0..]) {
            Err(_) => return Ok(None),                // FIXME: errors other than EOF?
            Ok(_) => (),
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
        let freqs = match HuffmanData::read_freqs(&mut reader) {
            Err(err) => panic!("read_freqs error: {:?}", err),
            Ok(Some(freqs)) => freqs,
            Ok(None) => return Ok(None),
        };

        let bs = match Bitstream::read(reader) {
            Err(err) => panic!("read bs error: {:?}", err),
            Ok(Some(bs)) => bs,
            Ok(None) => return Ok(None),
        };

        Ok(Some(HuffmanData { freqs, bs }))
    }
}

// input is ordered
fn build_tree(vals: &[u8]) -> Rc<Node> {
    let mut nodes: Vec<_> = vals.iter().
        map(|&val| Rc::new(Node::Leaf(val))).
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
