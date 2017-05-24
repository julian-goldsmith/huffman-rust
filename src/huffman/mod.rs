mod decode;
mod encode;
use bitstream::Bitstream;

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

fn find_pos(nodes: &Vec<Box<Node>>, node: &Box<Node>) -> usize {
    // FIXME: use a binary search
    match nodes.iter().position(|other| other.count < node.count) {
        Some(idx) => idx,
        None => nodes.len(),
    }
}

fn build_tree(freqs: &Box<[Freq; 65536]>) -> Box<Node> {
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

pub fn encode(data: &Vec<u16>) -> Result<(Box<[Freq; 65536]>, Bitstream),()> {
    encode::encode_internal(data)
}

pub fn decode(freqs: &Box<[Freq; 65536]>, bs: &Bitstream) -> Result<Vec<u16>, String> {
    decode::decode_internal(freqs, bs)
}
