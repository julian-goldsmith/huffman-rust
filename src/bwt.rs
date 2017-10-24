use byteorder::{BigEndian, ByteOrder};

pub fn encode(data: &[u8]) -> Vec<u8> {
    let len = data.len();

    let mut looped: Vec<u8> = Vec::with_capacity(len * 2);
    looped.extend_from_slice(data);
    looped.extend_from_slice(data);
    looped.pop();

    let mut perms = looped.windows(len).collect::<Vec<_>>();

    perms.sort();

    let idx = perms.iter().
        position(|perm| perm as &[u8] == data).
        unwrap();

    let mut buf = Vec::with_capacity(4 + len);
    buf.append(&mut vec![0, 0, 0, 0]);

    BigEndian::write_u32(&mut buf[0..4], idx as u32);

    for perm in &perms {
        buf.push(perm[len - 1]);
    };

    buf
}

pub fn decode(in_data: &[u8]) -> Vec<u8> {
    let data = &in_data[4..];

    let mut num_appearances = Vec::<(u8, usize)>::with_capacity(data.len());
    let mut num_char_appearances = vec![0; 256];
    
    for &c in data {
        num_appearances.push((c, num_char_appearances[c as usize]));
        num_char_appearances[c as usize] += 1;
    };

    let mut num_less_than = Vec::with_capacity(256);
    let mut less_than = 0;

    for appearances in &num_char_appearances {
        num_less_than.push(less_than);
        less_than += appearances;
    };

    let mut out_bytes = Vec::with_capacity(data.len());

    unsafe {
        out_bytes.set_len(data.len());
    };

    let mut idx = BigEndian::read_u32(&in_data[0..4]) as usize;
    let mut ob_idx = data.len();

    for _ in 0..data.len() {
        let ap = num_appearances[idx];

        ob_idx -= 1;
        out_bytes[ob_idx] = ap.0;

        idx = ap.1 + num_less_than[ap.0 as usize];
    };

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
