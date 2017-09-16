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
    let idx = BigEndian::read_u32(&in_data[0..5]) as usize;
    let data = &in_data[4..];

    let mut num_appearances = Vec::<(u8, usize)>::new();
    let mut num_less_than = iter::repeat(0).take(256).collect::<Vec<_>>();
    
    for &c in data {
        let n = num_appearances.iter().filter(|a| a.0 == c).count();

        num_appearances.push((c, n));

        for i in (c as usize + 1)..256 {
            num_less_than[i] += 1;
        };
    };

    let mut out_bytes = Vec::<u8>::new();
    out_bytes.resize(data.len(), 0);

    let mut ap = num_appearances[idx];
    out_bytes[data.len() - 1] = ap.0;

    for i in 2..(data.len() + 1) {
        println!("{:?}    {}    {}", ap, num_less_than[ap.0 as usize], ap.1 + num_less_than[ap.0 as usize]);

        let idx = ap.1 + num_less_than[ap.0 as usize];
        ap = num_appearances[idx];

        out_bytes[data.len() - i] = ap.0;
    };

    out_bytes.into_boxed_slice()
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
        let output = bwt::decode(&encoded);

        assert_eq!(input, &output as &[u8]);
    }
}
