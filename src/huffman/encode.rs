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
fn precalc_bitstreams(freqs: &[u8]) -> Result<Vec<Option<Bitstream>>,()> {
    // TODO: byte-wise table rather than bit-
    // https://www.reddit.com/r/rust/comments/54jlxf/huffman_coding_implementation_in_rust/d82frgt/
    let root = huffman::build_tree(freqs);

    let mut values = vec![None; 256];
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

fn build_freqs(data: &[u8]) -> Vec<u8> {
    let mut freqs = (0..256).map(|i| (i as u8, 0)).collect::<Vec<_>>();

    for &c in data {
        freqs[c as usize].1 += 1;
    };

    freqs.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(b.0.cmp(&a.0)));

    freqs.iter().map(|freq| freq.0).collect()
}

pub fn encode(data: &Vec<u8>) -> Result<HuffmanData,()> {
    let freqs = build_freqs(data);
    let streams = precalc_bitstreams(&freqs).unwrap();
    let bs = data.iter().
        map(|c| streams[*c as usize].as_ref().unwrap()).
        fold(Bitstream::new(), 
             |mut acc, x| { acc.append_bitstream(x); acc });
    Ok(HuffmanData { freqs: freqs.into_boxed_slice(), bs })
}
