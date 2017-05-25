mod decode;
mod encode;
use bitstream::Bitstream;
use std::io;
use std::io::Write;
use std::io::Read;

pub use self::encode::encode;
pub use self::decode::decode;

#[derive(Debug)]
pub struct Node {
    count: u16,
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

        let freqs_filtered_len = freqs_filtered.len() as u16;
        let freqs_filtered_buf = [(freqs_filtered_len >> 8) as u8, (freqs_filtered_len & 0xff) as u8];

        let freqs_as_u8: Vec<u8> = freqs_filtered.iter().
            flat_map(|freq| 
                     vec![(freq.val >> 8) as u8,
                          (freq.val & 0xff) as u8,
                          (freq.count >> 8) as u8,
                          (freq.count & 0xff) as u8]).
            collect();

        let bytes_out =
            match writer.write(&freqs_filtered_buf) {
                Err(err) => return Err(err),
                Ok(nb) => nb,
            }
            +
            match writer.write(&freqs_as_u8) {
                Err(err) => return Err(err),
                Ok(nb) => nb,
            };
        Ok(bytes_out)
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

    pub fn read(reader: &mut Read) -> io::Result<HuffmanData> {
        let mut freqs_len_buf = [0 as u8; 2];
        reader.read(&mut freqs_len_buf)?;

        let freqs_len = (freqs_len_buf[0] as usize) << 8 | freqs_len_buf[1] as usize;

        let mut freqs_buf: Vec<u8> = Vec::with_capacity(freqs_len * 4);
        unsafe {
            freqs_buf.set_len(freqs_len * 4);
        }
        reader.read(&mut freqs_buf[0..])?;

        let mut freqs: Vec<Freq> = Vec::with_capacity(65536);
        for i in 0..freqs_len {
            freqs.push(Freq {
                val: (freqs_buf[i * 4] as u16) << 8 | freqs_buf[i * 4 + 1] as u16,
                count: (freqs_buf[i * 4 + 2] as u16) << 8 | freqs_buf[i * 4 + 3] as u16,
            });
        };
        freqs.shrink_to_fit();

        let bs = Bitstream::read(reader)?;

        Ok(HuffmanData { freqs, bs })
    }
}

fn find_pos(nodes: &Vec<Box<Node>>, node: &Box<Node>) -> usize {
    // FIXME: use a binary search
    match nodes.iter().position(|other| other.count < node.count) {
        Some(idx) => idx,
        None => nodes.len(),
    }
}

fn build_tree(freqs: &Vec<Freq>) -> Box<Node> {
    let mut nodes: Vec<Box<Node>> = freqs.iter().
        map(|freq| Box::new(
            Node {
                count: freq.count,
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
                    count: left.count + right.count,
                    val: None,
                    left: Some(left),
                    right: Some(right),
                });

                if node.count > 0 {
                    let idx = find_pos(&nodes, &node);
                    nodes.insert(idx, node);
                }
            },
            _ => panic!("Must have nodes to build_tree"),
        };
    };
}
