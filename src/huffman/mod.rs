mod decode;
mod encode;
use bitstream::Bitstream;
use std::io;
use std::io::Write;
use std::io::Read;
use std::rc::Rc;
use byteorder::{BigEndian, ByteOrder};

pub use self::encode::encode;
pub use self::decode::decode;

#[derive(Debug, PartialEq, PartialOrd)]
pub enum Node {
    Leaf { freq: usize, val: u8, },
    Tree { freq: usize, left: Rc<Node>, right: Rc<Node>, },
}

impl Node {
    pub fn get_freq(&self) -> usize {
        match self {
            &Node::Leaf { val: _, freq } => freq,
            &Node::Tree { left: _, right: _, freq } => freq,
        }
    }
}

pub struct HuffmanData {
    pub freqs: Box<[usize; 256]>,
    pub bs: Bitstream,
}

impl HuffmanData {
    fn write_freqs(&self, mut writer: &mut Write) -> io::Result<usize> {
        let mut new_freqs = [0; 256];

        for i in 0..256 {
            new_freqs[i] = self.freqs[i] as u16;
        };

        let mut bytes = [0; 512];
        BigEndian::write_u16_into(&new_freqs, &mut bytes);

        match writer.write(&bytes) {
            Err(err) => Err(err),
            Ok(count) => Ok(count),
        }
    }

    fn read_freqs(reader: &mut Read) -> io::Result<Option<Box<[usize; 256]>>> {
        let mut bytes = [0; 512];

        match reader.read(&mut bytes) {
            Err(_) => return Ok(None),                // FIXME: errors other than EOF?
            Ok(_) => (),
        };

        let mut freqs_u16 = [0; 256];
        BigEndian::read_u16_into(&bytes, &mut freqs_u16[0..]);

        let mut freqs = Box::new([0; 256]);

        for i in 0..256 {
            freqs[i] = freqs_u16[i] as usize;
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
fn build_tree(vals: &[usize; 256]) -> Rc<Node> {
    // this produces an improper tree: values always end up being 8 bits
    let mut nodes: Vec<_> = (0..256).
        map(|i| (i as u8, vals[i])).
        map(|val| Rc::new(Node::Leaf { val: val.0, freq: val.1 })).
        collect();

    nodes.sort_unstable_by(|n1, n2| n1.partial_cmp(n2).unwrap().reverse());

    // need to go down from root, find a spot for our node, then insert it
    loop {
        let lo = nodes.pop();
        let ro = nodes.pop();

        match (lo, ro) {
            (Some(left), None) => return left,
            (Some(left), Some(right)) => {
                let freq = left.get_freq() + right.get_freq();
                let node = Rc::new(Node::Tree { left, right, freq });

                nodes.push(node);

                nodes.sort_unstable_by(|n1, n2| n1.partial_cmp(n2).unwrap().reverse());
            },
            _ => panic!("Must have nodes to build_tree"),
        };
    };
}
