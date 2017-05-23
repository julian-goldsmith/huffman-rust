use bitstream::Bitstream;

#[derive(Debug)]
pub struct Node {
    count: usize,
    val: Option<u16>,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

enum State<'a> {
    Right(&'a Box<Node>),
    Left(&'a Box<Node>),
    Done,
}

fn build_freq_list(data: &Vec<u16>) -> Vec<Box<Node>> {
    let mut nodes: Vec<Box<Node>> = (0..65536).
        map(|i| Box::new(
            Node {
                count: 0,
                val: Some(i as u16),
                left: None,
                right: None,
            })).
        collect();

    for code in data.iter() {
        nodes[*code as usize].count += 1;
    };

    nodes.sort_by_key(|node| node.count);

    nodes
}

fn find_pos(nodes: &Vec<Box<Node>>, node: &Box<Node>) -> usize {
    // FIXME: use a better algorithm
    match nodes.iter().position(|other| other.count > node.count) {
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

pub fn build_tree(data: &Vec<u16>) -> Box<Node> {
    let freq_list = build_freq_list(data);
    build_tree_internal(freq_list)
}

pub fn precalc_bitstreams(node: &Box<Node>) -> Result<Vec<Option<Bitstream>>,()> {
    let mut values: Vec<Option<Bitstream>> = (0..65536).map(|_| None).collect();
    let mut history: Vec<State> = Vec::new();
    let mut acc = Bitstream::new();

    let initial_state = State::Left(node);
    history.push(initial_state);

    loop {
        match history.pop() {
            None => return Ok(values),
            Some(curr_state) =>
                match curr_state {
                    State::Done => { let _ = acc.pop(); },

                    State::Right(node) if node.val.is_none() => {
                        let next_node = node.right.as_ref().unwrap();

                        acc.pop();
                        acc.append(1);
                        history.push(State::Done);
                        history.push(State::Left(next_node));
                    },

                    State::Left(node) if node.val.is_none() => {
                        let next_node = node.left.as_ref().unwrap();

                        acc.append(0);
                        history.push(State::Right(node));
                        history.push(State::Left(next_node));
                    },

                    State::Right(node) | State::Left(node) => values[node.val.unwrap() as usize] = Some(acc.clone()),
                },
        }
    }
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
