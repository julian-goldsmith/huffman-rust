use huffman;
use huffman::{HuffmanData, Node};

pub fn decode(data: &HuffmanData) -> Result<Vec<u8>, String> {
    let root = huffman::build_tree(&data.freqs);
    let mut node: &Node = &root;
    let mut s = Box::new(data.bs.clone());
    let mut acc = Vec::new();

    loop {
        match node {
            &Node::Leaf(val) => { acc.push(val); node = &root; },
            &Node::Tree { ref left, ref right } =>
                match s.pop_start() {
                    Some(0) => { node = &left; },
                    Some(1) => { node = &right; },
                    None => return Ok(acc),
                    _ => return Err(String::from("Bad value from Bitstream in huffman::decode")),
                },
        }
    }
}
