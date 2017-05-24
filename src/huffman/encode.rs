use bitstream::Bitstream;
use huffman;
use huffman::*;
use std::mem;
use std::ptr;

enum State<'a> {
    Right(&'a Box<Node>),
    Left(&'a Box<Node>),
    Done,
}

fn build_freq_list(data: &Vec<u16>) -> Box<[Freq; 65536]> {
    let mut freqs: Box<[Freq; 65536]> = 
        unsafe {
            let mut freqs: Box<[Freq; 65536]> = mem::uninitialized();
            for i in 0..65536 {
                ptr::write(&mut freqs[i], 
                    Freq {
                        val: i as u16,
                        count: 0,
                    });
            };
            freqs
        };

    for code in data.iter() {
        freqs[*code as usize].count += 1;
    };

    freqs.sort_by_key(|freq| freq.count);

    freqs
}

fn precalc_bitstreams(freqs: &Box<[Freq; 65536]>) -> Result<Vec<Option<Bitstream>>,()> {
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

pub fn encode_internal(data: &Vec<u16>) -> Result<(Box<[Freq; 65536]>, Bitstream),()> {
    let freqs = build_freq_list(data);
    let streams = precalc_bitstreams(&freqs).unwrap();
    let enc = data.iter().
        map(|c| streams[*c as usize].as_ref().unwrap()).
        fold(Bitstream::new(), 
             |mut acc, x| { acc.append_bitstream(x); acc });
    Ok((freqs, enc))
}
