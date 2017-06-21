use bitstream::Bitstream;
use huffman;
use huffman::*;

enum State<'a> {
    Right(&'a Node),
    Left(&'a Node),
    Done,
}

fn precalc_bitstreams(max: u32) -> Result<Vec<Option<Bitstream>>,()> {
    let root = huffman::build_tree(max);

    let mut values: Vec<Option<Bitstream>> = (0..max).map(|_| None).collect();
    let mut history: Vec<State> = Vec::new();
    let mut acc = Bitstream::new();

    let initial_state = State::Left(root.as_ref());
    history.push(initial_state);

    loop {
        match history.pop() {
            None => return Ok(values),
            Some(curr_state) =>
                match curr_state {
                    State::Done => { let _ = acc.pop(); },

                    State::Right(&Node::Leaf(val)) | State::Left(&Node::Leaf(val)) => values[val as usize] = Some(acc.clone()),

                    State::Right(&Node::Tree { left: _, ref right }) => {
                        let next_node = right.as_ref();

                        acc.pop();
                        acc.append(1);
                        history.push(State::Done);
                        history.push(State::Left(next_node));
                    },

                    State::Left(node @ &Node::Tree { left: _, right: _ }) => {
                        let left = match node {
                            &Node::Tree { ref left, right: _ } => left,
                            _ => unreachable!(),
                        };
                        let next_node = left.as_ref();

                        acc.append(0);
                        history.push(State::Right(node));
                        history.push(State::Left(next_node));
                    },
                },
        }
    }
}

pub fn encode(data: &Vec<u32>) -> Result<Box<HuffmanData>,()> {
    let max = *data.iter().max().unwrap() + 1;
    let streams = precalc_bitstreams(max).unwrap();
    let bs = data.iter().
        map(|c| streams[*c as usize].as_ref().unwrap()).
        fold(Bitstream::new(), 
             |mut acc, x| { acc.append_bitstream(x); acc });
    Ok(Box::from(HuffmanData { max, bs }))
}
