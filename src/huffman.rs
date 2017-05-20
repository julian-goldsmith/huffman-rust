use std;
use std::io;
use std::io::Write;
use bitstream::Bitstream;

#[derive(Debug)]
pub struct Node {
    count: usize,
    val: Option<u16>,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

fn build_freq_list(data: &Vec<u16>) -> Vec<Box<Node>> {
    let mut nodes: Vec<Box<Node>> = Vec::new();

    for i in 0..65536 {
        nodes.push(Box::new(
            Node { 
                count: 0, 
                val: Some(i as u16),
                left: None, 
                right: None 
            }));
    };

    for code in data.iter() {
        nodes[*code as usize].count += 1;
    };

    nodes
}

fn clamp(val: usize, min: usize, max: usize) -> usize {
    if val > max {
        max
    } else if val < min {
        min
    } else {
        val
    }
}

fn find_pos(nodes: &Vec<Box<Node>>, node: &Box<Node>) -> Option<usize> {
    // same as nodes.iter().position(|other| other.count > node.count)
    
    if nodes.len() == 0 {
        return Some(0)
    }

    let mut lower = 0;
    let mut upper = nodes.len() - 1;

    loop {
        if nodes[lower].count == node.count {
            return Some(lower)
        }

        if lower == upper {
            return None
        }

        let newlower = clamp(lower * 2 + 1, lower, upper);
        if nodes[lower].count > node.count {
            upper = newlower;
        } else {
            lower = newlower;
        }

        let newupper = clamp(upper / 2, lower, upper);
        if nodes[upper].count > node.count {
            lower = newupper;
        } else {
            upper = newupper;
        }
    }
}

fn build_tree_internal(mut nodes: Vec<Box<Node>>) -> Box<Node> {
    // FIXME: can we just insert the node instead of sorting every time?

    let mut count = 0;

    loop {
        let lo = nodes.pop();
        let ro = nodes.pop();

        match (lo, ro) {
            (Some(left), None) => { println!("Exiting build_tree_internal"); return left; },
            (Some(left), Some(right)) => {
                count += 1;

                if count % 256 == 0 {
                    println!("Count {}", count);
                };

                let node = Box::new(Node {
                    count: left.count + right.count,
                    val: None,
                    left: Some(left),
                    right: Some(right),
                });
                let idx = match find_pos(&nodes, &node) {
                    Some(idx) => idx,
                    None => nodes.len(),
                };
                nodes.insert(idx, node);
            },
            _ => panic!("Must have nodes to build_tree"),
        };
    };
}

pub fn build_tree(data: &Vec<u16>) -> Box<Node> {
    let freq_list = build_freq_list(data);
    build_tree_internal(freq_list)
}

fn encode_char_internal(root: &Node, c: u16, acc: Bitstream) -> Option<Bitstream> {
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

fn encode_char(root: &Node, c: u16) -> Option<Bitstream> {
    encode_char_internal(root, c, Bitstream::new())
}

pub fn precalc_bitstreams(root: &Node) -> Result<Vec<Bitstream>,()> {
    let mut calc: Vec<Bitstream> = Vec::new();
    for i in 0..65536 {
        match encode_char(root, i as u16) {
            Some(item) => calc.push(item),
            None => return Err(()),
        }
    };
    Ok(calc)
}

fn decode_char(root: &Node, mut s: Bitstream) -> (Option<u16>, Bitstream) {
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

fn decode_bitstream_internal(root: &Node, s: Bitstream, mut acc: Vec<u16>) -> Option<Vec<u16>> {
    match decode_char(root, s) {
        (Some(c), ns) => { acc.push(c); decode_bitstream_internal(root, ns, acc) },
        (None, _) => Some(acc),
    }
}

pub fn decode_bitstream(root: &Node, s: Bitstream) -> Option<Vec<u16>> {
    decode_bitstream_internal(root, s.reverse(), Vec::new())
}
