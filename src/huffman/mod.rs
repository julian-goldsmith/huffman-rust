pub mod decode;
pub mod encode;
use bitstream::Bitstream;

#[derive(Debug)]
pub struct Node {
    count: u16,
    val: Option<u16>,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

fn find_pos(nodes: &Vec<Box<Node>>, node: &Box<Node>) -> usize {
    // FIXME: use a better algorithm
    match nodes.iter().position(|other| other.count < node.count) {
        Some(idx) => idx,
        None => nodes.len(),
    }
}

fn build_tree_internal(mut nodes: Vec<Box<Node>>) -> Box<Node> {
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
