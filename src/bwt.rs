use std::iter;
use byteorder::{BigEndian, ByteOrder};

pub fn encode(data: &[u8]) -> Box<[u8]> {
    let len = data.len();
    let looped: Vec<u8> = data.iter().cloned().cycle().take(len * 2 - 1).collect();
    let mut perms: Vec<&[u8]> = Vec::new();

    for w in looped.windows(len) {
        perms.push(w);
    };

    perms.sort();

    let idx = perms.iter().position(|perm| &perm as &[u8] == data).unwrap();
    let mut idx_buf = [0; 4];
    BigEndian::write_u32(&mut idx_buf, idx as u32);

    idx_buf.iter().
        cloned().
        chain(perms.iter().map(|perm| perm[len - 1])).
        collect::<Vec<u8>>().
        into_boxed_slice()
}

pub fn decode(in_data: &[u8]) -> Box<[u8]> {
    /*
    let idx = BigEndian::read_u32(&in_data[0..5]) as usize;
    let data = &in_data[4..];
    let mut out = iter::repeat(Vec::new()).take(data.len()).collect::<Vec<_>>();

    for _ in 0..data.len() {
        for i in 0..data.len() {
            out[i].insert(0, data[i]);
        };

        out.sort();
    };

    out[idx].clone().into_boxed_slice();
    */

    let mut num_appearances = iter::repeat(0).take(256).collect::<Vec<_>>();
    let mut num_less_than = iter::repeat(0).take(256).collect::<Vec<_>>();
    
    for *c in data {
        num_appearances[c as usize] += 1;

        for i in 0..c {
            num_less_than[c as usize] += 1;
        };
    };

    let out_bytes = Vec::<u8>::new();
    for i in 0..data.len() {
        
    };
}

#[cfg(test)]
mod test {
    use byteorder::{BigEndian, ByteOrder};
    use std::str;
    use bwt;

    #[test]
    fn encode_test() {
        let input = "^BANANA|".as_bytes();
        let expected_output = "BNN^AA|A".as_bytes();
        let expected_idx = 6;

        let val = bwt::encode(&input);
        let idx = BigEndian::read_u32(&val[0..4]);

        println!("{}    {}", 
                 str::from_utf8(&val[4..]).unwrap(),
                 str::from_utf8(expected_output).unwrap());

        assert_eq!(&val[4..] as &[u8], expected_output);
        assert_eq!(idx, expected_idx);
    }

    #[test]
    fn decode_test() {
        let input = [0, 0, 0, 6].iter().chain("BNN^AA|A".as_bytes().iter()).cloned().collect::<Vec<_>>();
        let expected_string = "^BANANA|".as_bytes();

        let output = bwt::decode(&input);

        assert_eq!(&output as &[u8], expected_string);
    }

    #[test]
    fn encode_decode_test() {
        let input = "^BANANA|".as_bytes();

        let encoded = bwt::encode(&input);
        let output = bwt::decode(&encoded);

        assert_eq!(input, &output as &[u8]);
    }
}
