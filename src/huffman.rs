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

// TODO: add parent in here, get rid of struct
enum StateVal {
    Right,
    Left,
    Done,
}

struct State<'a> {
    pub state: StateVal,
    pub node: &'a Box<Node>,
}

pub fn precalc_bitstreams_internal(node: &Box<Node>, values: &mut Vec<Option<Bitstream>>, acc: Bitstream) {
    let mut history: Vec<State> = Vec::new();

    let initial_state = State { state: StateVal::Right, node: node };
    history.push(initial_state);

    loop {
        match history.pop() {
            None => return,
            Some(mut curr_state) => match curr_state.state {
                StateVal::Done => continue,
                StateVal::Left | StateVal::Right => {
                    let next_node = match curr_state.state { 
                        StateVal::Right => match curr_state.node.right.as_ref() {
                            Some(node) => node,
                            None => panic!("don't have node"),
                        },
                        StateVal::Left => match curr_state.node.left.as_ref() {
                            Some(node) => node,
                            None => panic!("don't have node"),
                        },
                        StateVal::Done => panic!("impossible state"), 
                    };

                    curr_state.state = match curr_state.state { 
                        StateVal::Right => StateVal::Left,                          // FIXME: right order?
                        StateVal::Left => StateVal::Done, 
                        StateVal::Done => continue, 
                    };
                    
                    history.push(curr_state);

                    match next_node.val {
                        Some(val) => values[val as usize] = Some(acc.clone()),
                        None => history.push(State { state: StateVal::Right, node: &next_node }),
                    };
                },
            },
        }
    }
}

pub fn precalc_bitstreams(root: &Box<Node>) -> Result<Vec<Option<Bitstream>>,()> {
    let mut calc: Vec<Option<Bitstream>> = (0..65536).map(|_| None).collect();
    precalc_bitstreams_internal(root, &mut calc, Bitstream::new());
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
