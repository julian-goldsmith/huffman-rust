mod bitstream;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::mem;
use std::ptr;
use bitstream::Bitstream;

#[derive(Debug)]
struct Node {
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

fn build_tree(data: &Vec<u8>) -> Node {
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

fn decode_char_internal(root: &Node, mut s: Bitstream) -> (Option<u8>, Bitstream) {
    match (root.val, &root.left, &root.right) {
        (Some(val), _, _) => (Some(val), s),
        (None, &Some(ref left), &Some(ref right)) =>
            match s.pop() {
                Some(0) => decode_char_internal(&left, s),
                Some(1) => decode_char_internal(&right, s),
                _ => (None, s)
            },
        _ => (None, s)
    }
}

fn decode_bitstream_internal(root: &Node, s: Bitstream, mut acc: Vec<u8>) -> Option<Vec<u8>> {
    match decode_char_internal(root, s) {
        (Some(c), ns) => { acc.push(c); decode_bitstream_internal(root, ns, acc) },
        (None, _) => Some(acc),
    }
}

fn decode_bitstream(root: &Node, s: Bitstream) -> Option<Vec<u8>> {
    decode_bitstream_internal(root, s.reverse(), Vec::new())
}

fn get_test_data() -> Vec<u8> {
    let path = Path::new("../testfile.txt");
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display,
                                                   why.description()),
        Ok(file) => file,
    };

    let mut bytes: Vec<u8> = Vec::new();
    match file.read_to_end(&mut bytes) {
        Ok(_) => bytes,
        Err(e) => panic!("Error reading file: {}", e),
    }
}

fn precalc_bitstreams(root: &Node) -> Result<[Bitstream; 256],()> {
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

fn main() {
    let data = get_test_data();
    let root = build_tree(&data);

    let calc = match precalc_bitstreams(&root) {
        Ok(calc) => calc,
        Err(_) => panic!("Couldn't precalc bitstream"),
    };

    let enc = data.iter().
        map(|c| &calc[*c as usize]).
        fold(Bitstream::new(), |acc, x| acc + x);

    let dec = decode_bitstream(&root, enc);
    println!("{:?}", dec)
}
