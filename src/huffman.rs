use std::mem;
use std::ptr;
use bitstream::Bitstream;

#[derive(Debug)]
pub struct Node {
    count: usize,
    val: Option<u8>,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

fn build_freq_list(data: &Vec<u8>) -> Vec<Node> {
    let mut nodes: Vec<Node> = (0..256).
        map(|i| Node { 
            count: 0, 
            val: Some(i as u8),
            left: None, 
            right: None 
        }).
        collect();

    for idx in data.iter().map(|c| *c as usize) {
        nodes[idx].count += 1
    }

    nodes
}

fn build_tree_internal(mut nodes: Vec<Node>) -> Node {
    // FIXME: can we just insert the node instead of sorting every time?
    nodes.sort_by(|a, b| b.count.cmp(&a.count));

    let lo = nodes.pop();
    let ro = nodes.pop();

    match (lo, ro) {
        (Some(left), None) => left,
        (Some(left), Some(right)) => {
            nodes.push(Node {
                count: left.count + right.count,
                val: None,
                left: Some(Box::new(left)),
                right: Some(Box::new(right)),
            });
            build_tree_internal(nodes)
        },
        _ => panic!("Must have nodes to build_tree"),
    }
}

pub fn build_tree(data: &Vec<u8>) -> Node {
    let freq_list = build_freq_list(data);
    build_tree_internal(freq_list)
}

fn encode_char_internal(root: &Node, c: u8, acc: Bitstream) -> Option<Bitstream> {
    match (root.val, &root.left, &root.right) {
        (Some(val), _, _) if val == c => Some(acc),
        (None, &Some(ref left), &Some(ref right)) =>
            match encode_char_internal(&left, c, acc.clone() + 0) {
                Some(leftret) => Some(leftret),
                None => encode_char_internal(&right, c, acc.clone() + 1),
            },
        _ => None,
    }
}

fn encode_char(root: &Node, c: u8) -> Option<Bitstream> {
    encode_char_internal(root, c, Bitstream::new())
}

fn decode_char(root: &Node, mut s: Bitstream) -> (Option<u8>, Bitstream) {
    match (root.val, &root.left, &root.right) {
        (Some(val), _, _) => (Some(val), s),
        (None, &Some(ref left), &Some(ref right)) =>
            match s.pop() {
                Some(0) => decode_char(&left, s),
                Some(1) => decode_char(&right, s),
                _ => (None, s)
            },
        _ => (None, s)
    }
}

fn decode_bitstream_internal(root: &Node, s: Bitstream, mut acc: Vec<u8>) -> Option<Vec<u8>> {
    match decode_char(root, s) {
        (Some(c), ns) => { acc.push(c); decode_bitstream_internal(root, ns, acc) },
        (None, _) => Some(acc),
    }
}

pub fn decode_bitstream(root: &Node, s: Bitstream) -> Option<Vec<u8>> {
    decode_bitstream_internal(root, s.reverse(), Vec::new())
}

pub fn precalc_bitstreams(root: &Node) -> Result<[Bitstream; 256],()> {
    unsafe {
        let mut calc: [Bitstream; 256] = mem::uninitialized();
        for i in 0..256 {
            match encode_char(root, i as u8) {
                Some(item) => ptr::write(&mut calc[i], item),
                None => return Err(()),
            }
        };
        Ok(calc)
    }
}

