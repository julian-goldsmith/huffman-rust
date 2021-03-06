use byteorder::{BigEndian, ByteOrder};

pub fn encode(data: &[u8]) -> Vec<u8> {
    let len = data.len();

    let mut data_looped = Vec::with_capacity(len + len - 1);
    data_looped.extend(data);
    data_looped.extend(&data[..(len-1)]);

    let mut suffixes = (0..len).map(|i| i).collect::<Vec<usize>>();
    suffixes.sort_unstable_by_key(|&s| &data_looped[s..(s + len)]);

    let mut buf = Vec::with_capacity(4 + len);
    buf.append(&mut vec![0, 0, 0, 0]);

    for i in 0..len {
        let suffix = suffixes[i];

        buf.push(data_looped[suffix + len - 1]);

        if suffix == 0 {
            // Write output index.
            BigEndian::write_u32(&mut buf[0..4], i as u32);
        };
    };

    buf
}

pub fn decode(buf: &[u8]) -> Vec<u8> {
    let data = &buf[4..];
    let len = data.len();
    let mut idx = BigEndian::read_u32(&buf[0..4]) as usize;
    let mut out_bytes = vec![0; len];

    let mut num_appearances = Vec::with_capacity(len);
    let mut num_char_appearances = [0; 256];
    let mut num_less_than = [0; 256];

    for &c in data {
        let ci = c as usize;
        num_appearances.push((c, num_char_appearances[ci]));
        num_char_appearances[ci] += 1;
    };

    for i in 0..255 {
        num_less_than[i + 1] = num_less_than[i] + num_char_appearances[i];
    };

    for ob_idx in (0..len).rev() {
        let ap = num_appearances[idx];

        idx = ap.1 + num_less_than[ap.0 as usize];
        out_bytes[ob_idx] = ap.0;
    };

    out_bytes
}

#[cfg(test)]
mod test {
    use byteorder::{BigEndian, ByteOrder};
    use std::str;
    use bwt;

    #[test]
    fn encode() {
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
    fn encode_2() {
        let expected_output = "BNN^AA|A".as_bytes();
        let expected_idx = 6;
        let input = "^BANANA|".as_bytes();

        let encoded = bwt::encode(&input);

        let idx = BigEndian::read_u32(&encoded[0..4]);
        let output = &encoded[4..] as &[u8];

        assert_eq!(expected_output, output);
        assert_eq!(expected_idx, idx);
    }

    #[test]
    fn decode() {
        let input = [0, 0, 0, 14].iter().chain("ssat tt hiies .".as_bytes().iter()).cloned().collect::<Vec<_>>();
        let expected_string = "this is a test.".as_bytes();

        let output = bwt::decode(&input);

        println!("returned str: {}", str::from_utf8(&output as &[u8]).unwrap());

        assert_eq!(expected_string, &output as &[u8]);
    }
}
