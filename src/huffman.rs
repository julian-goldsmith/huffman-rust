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

fn find_pos(nodes: &Vec<Box<Node>>, node: &Box<Node>) -> usize {
    // FIXME: use a better algorithm
    return nodes.iter().position(|other| other.count > node.count)
}

fn build_tree_internal(mut nodes: Vec<Box<Node>>) -> Box<Node> {
    // FIXME: can we just insert the node instead of sorting every time?

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

// TODO: add parent in here, get rid of struct
enum State<'a> {
    Right(&'a Box<Node>),
    Left(&'a Box<Node>),
    Done,
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

fn decode_char(root: &Node, mut s: Box<Bitstream>) -> (Option<u16>, Box<Bitstream>) {
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

fn decode_bitstream_internal(root: &Node, s: Box<Bitstream>, mut acc: Vec<u16>) -> Option<Vec<u16>> {
    match decode_char(root, s) {
        (Some(c), ns) => { acc.push(c); decode_bitstream_internal(root, ns, acc) },
        (None, _) => Some(acc),
    }
}

pub fn decode_bitstream(root: &Node, s: Box<Bitstream>) -> Option<Vec<u16>> {
    decode_bitstream_internal(root, Box::new(s.reverse()), Vec::new())
}
