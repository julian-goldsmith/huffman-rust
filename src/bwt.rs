use time;
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

pub fn decode(in_data: &[u8]) -> Vec<u8> {
    // NOTE: data.len() can't be larger than 65536
    let mut idx = BigEndian::read_u32(&in_data[0..4]) as usize;
    let data = &in_data[4..];

    let mut num_appearances = Vec::<(u8, usize)>::with_capacity(data.len());
    let mut num_char_appearances = vec![0; 256];
    
    let appearances_start = time::now();
    for &c in data {
        num_appearances.push((c, num_char_appearances[c as usize]));
        num_char_appearances[c as usize] += 1;
    };
    println!("done creating appearances in {}", time::now() - appearances_start);

    let lessthans_start = time::now();
    let mut sorted = data.to_vec();
    sorted.sort_unstable();

    let mut num_less_than = vec![0; 256];
    let mut i = 0;
    for c in 0..256 {
        num_less_than[c] = i;

        while i < sorted.len() && (sorted[i] as usize) <= c { 
            i += 1; 
        };
    };
    println!("done counting less thans in {}", time::now() - lessthans_start);

    let mut out_bytes = vec![0; data.len()];

    let rebuilding_start = time::now();

    let mut ap = (0, 0);

    for i in (0..data.len()).rev() {
        ap = num_appearances[idx];
        out_bytes[i] = ap.0;

        idx = ap.1 + num_less_than[ap.0 as usize];
    };

    out_bytes[0] = ap.0;
    println!("done rebuilding block in {}", time::now() - rebuilding_start);

    out_bytes
}

#[cfg(test)]
mod test {
    use byteorder::{BigEndian, ByteOrder};
    use std::str;
    use bwt;

    #[test]
    fn encode_test() {
        let input = "this is a test.".as_bytes();
        let expected_output = "ssat tt hiies .".as_bytes();
        let expected_idx = 14;

        let val = bwt::encode(&input);
        let idx = BigEndian::read_u32(&val[0..4]);

        println!("{}    {}", 
                 str::from_utf8(&val[4..]).unwrap(),
                 str::from_utf8(expected_output).unwrap());

        assert_eq!(expected_output, &val[4..] as &[u8]);
        assert_eq!(expected_idx, idx);
    }

    #[test]
    fn decode_test() {
        let input = [0, 0, 0, 14].iter().chain("ssat tt hiies .".as_bytes().iter()).cloned().collect::<Vec<_>>();
        let expected_string = "this is a test.".as_bytes();

        let output = bwt::decode(&input);

        println!("returned str: {}", str::from_utf8(&output as &[u8]).unwrap());

        assert_eq!(expected_string, &output as &[u8]);
    }

    #[test]
    fn encode_decode_test() {
        let input = "^BANANA|".as_bytes();

        let encoded = bwt::encode(&input);
        println!("i = {:?}", BigEndian::read_u32(&encoded[0..4]));
        let output = bwt::decode(&encoded);

        assert_eq!(input, &output as &[u8]);
    }
}
