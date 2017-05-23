use bitstream::Bitstream;
use huffman;
use huffman::Node;

pub fn build_tree(data: &Vec<u16>) -> Box<Node> {
    let freq_list = build_freq_list(data);
    huffman::build_tree_internal(freq_list)
}

pub fn decode_bitstream(root: &Node, in_stream: &Box<Bitstream>) -> Option<Vec<u16>> {
    let mut node = root;
    let mut s = Box::new(in_stream.reverse());
    let mut acc = Vec::new();

    loop {
        match (node.val, &node.left, &node.right) {
            (Some(val), _, _) => { acc.push(val); node = root; },
            (None, &Some(ref left), &Some(ref right)) =>
                match s.pop() {
                    Some(0) => { node = &left; },
                    Some(1) => { node = &right; },
                    None => return Some(acc),
                    _ => return None,
                },
            _ => return None
        }
    }
}
