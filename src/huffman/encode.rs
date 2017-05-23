use bitstream::Bitstream;
use huffman;
use huffman::Node;

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

pub fn build_tree(data: &Vec<u16>) -> Box<Node> {
    let freq_list = build_freq_list(data);
    huffman::build_tree_internal(freq_list)
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
