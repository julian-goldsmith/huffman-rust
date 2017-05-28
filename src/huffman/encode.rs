use bitstream::Bitstream;
use huffman;
use huffman::*;

enum State<'a> {
    Right(&'a Box<Node>),
    Left(&'a Box<Node>),
    Done,
}

fn build_freq_list(data: &Vec<u16>) -> Vec<Freq> {
    // tried doing this functionally with itertools, but it didn't go well
    let mut freqs: Vec<Freq> = (0..65536).
        map(|i|
            Freq {
                val: i as u16,
                count: 0,
            }).
        collect();

    for code in data {
        freqs[*code as usize].count += 1;
    };

    freqs.retain(|freq| freq.count > 0);
    freqs.sort_by_key(|freq| freq.count);

    assert!(freqs.iter().fold(0, |sum, freq| sum + freq.count as usize) < 65536);

    freqs
}

fn precalc_bitstreams(freqs: &Vec<Freq>) -> Result<Vec<Option<Bitstream>>,()> {
    let root = huffman::build_tree(freqs);

    let mut values: Vec<Option<Bitstream>> = (0..65536).map(|_| None).collect();
    let mut history: Vec<State> = Vec::new();
    let mut acc = Bitstream::new();

    let initial_state = State::Left(&root);
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

pub fn encode(data: &Vec<u16>) -> Result<Box<HuffmanData>,()> {
    let freqs = build_freq_list(data);
    let streams = precalc_bitstreams(&freqs).unwrap();
    let bs = data.iter().
        map(|c| streams[*c as usize].as_ref().unwrap()).
        fold(Bitstream::new(), 
             |mut acc, x| { acc.append_bitstream(x); acc });
    Ok(Box::from(HuffmanData { freqs, bs }))
}
