use bitstream::Bitstream;
use huffman;
use huffman::*;

enum State<'a> {
    Right(&'a Node),
    Left(&'a Node),
    Done,
}

// we could keep this around between blocks.  we would need to check if the new max is higher, and
// add new elements as needed
fn precalc_bitstreams(max: u8) -> Result<Vec<Option<Bitstream>>,()> {
    // TODO: byte-wise table rather than bit-
    // https://www.reddit.com/r/rust/comments/54jlxf/huffman_coding_implementation_in_rust/d82frgt/
    let root = huffman::build_tree(max);

    let mut values = vec![None; max as usize + 1];
    let mut history = Vec::new();
    let mut acc = Bitstream::new();

    let initial_state = State::Left(&root);
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
                        history.push(State::Left(&next_node));
                    },

                    State::Left(node @ &Node::Tree { left: _, right: _ }) => {
                        let left = match node {
                            &Node::Tree { ref left, right: _ } => left,
                            _ => unreachable!(),
                        };
                        let next_node = left;

                        acc.append(0);
                        history.push(State::Right(node));
                        history.push(State::Left(&next_node));
                    },
                },
        }
    }
}

pub fn encode(data: &Vec<u8>) -> Result<HuffmanData,()> {
    let max = *data.iter().max().unwrap();
    let streams = precalc_bitstreams(max).unwrap();
    let bs = data.iter().
        map(|c| streams[*c as usize].as_ref().unwrap()).
        fold(Bitstream::new(), 
             |mut acc, x| { acc.append_bitstream(x); acc });
    Ok(HuffmanData { max, bs })
}
