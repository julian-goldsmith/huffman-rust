use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[derive(Debug)]
struct Node {
    count: usize,
    val: Option<char>,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

fn build_freq_list(data: &String) -> Vec<Node> {
    let mut nodes: Vec<Node> = (0..256).
        map(|i| Node { 
            count: 0, 
            val: Some(i as u8 as char),
            left: None, 
            right: None 
        }).
        collect();

    for idx in data.chars().map(|c| c as usize) {
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

fn build_tree(data: &String) -> Node {
    build_tree_internal(build_freq_list(data))
}

fn encode_char_internal(root: &Node, c: char, acc: String) -> Option<String> {
    match (root.val, &root.left, &root.right) {
        (Some(val), _, _) if val == c => Some(acc),
        (None, &Some(ref left), &Some(ref right)) =>
            match encode_char_internal(&left, c, acc.clone() + &"0") {
                Some(leftret) => Some(leftret),
                None => encode_char_internal(&right, c, acc.clone() + &"1"),
            },
        _ => None,
    }
}

fn encode_char(root: &Node, c: char) -> Option<String> {
    encode_char_internal(root, c, String::from(""))
}

fn decode_char_internal(root: &Node, mut s: String) -> Option<char> {
    match (root.val, &root.left, &root.right) {
        (Some(val), _, _) => Some(val),
        (None, &Some(ref left), &Some(ref right)) =>
            match s.pop() {
                Some('0') => decode_char_internal(&left, s),
                Some('1') => decode_char_internal(&right, s),
                _ => None,
            },
        _ => None
    }
}

fn decode_char(root: &Node, s: String) -> Option<char> {
    decode_char_internal(root, s.chars().rev().collect())
}

fn get_test_data() -> String {
    let path = Path::new("../testfile.txt");
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display,
                                                   why.description()),
        Ok(file) => file,
    };

    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display,
                                                   why.description()),
        Ok(_) => s,
    }
}

fn main() {
    let data = get_test_data();
    let root = build_tree(&data);

    let enc = data.chars().map(|c| encode_char(&root, c).unwrap());

    let dec: String = enc.map(|s| decode_char(&root, s).unwrap()).collect();
    print!("{:?}\n", dec)
}
