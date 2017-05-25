use huffman;
use huffman::HuffmanData;

pub fn decode(data: &HuffmanData) -> Result<Vec<u16>, String> {
    let root = huffman::build_tree(&data.freqs);
    let mut node = &root;
    let mut s = Box::new(data.bs.reverse());
    let mut acc = Vec::new();

    loop {
        match (node.val, &node.left, &node.right) {
            (Some(val), _, _) => { acc.push(val); node = &root; },
            (None, &Some(ref left), &Some(ref right)) =>
                match s.pop() {
                    Some(0) => { node = &left; },
                    Some(1) => { node = &right; },
                    None => return Ok(acc),
                    _ => return Err(String::from("Bad value from Bitstream in huffman::decode::decode_internal")),
                },
            _ => return Err(String::from("Invalid state in huffman::decode::decode_internal")),
        }
    }
}
