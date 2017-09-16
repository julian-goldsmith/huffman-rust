use std::iter;

pub fn encode(data: &[u8]) -> Vec<u8> {
    let mut i = 0;
    let mut out = Vec::with_capacity(data.len());

    loop {
        if i >= data.len() {
            break;
        };

        let mut count = 0 as usize;
        let c = data[i];

        while i < data.len() && c == data[i] && count < 256 {
            count += 1;
            i += 1;
        };

        out.push(count as u8);
        out.push(c);
    };

    out
}

pub fn decode(data: &Vec<u8>) -> Vec<u8> {
    assert_eq!(data.len() % 2, 0);

    data.
        chunks(2).
        map(|chunk| (chunk[0], chunk[1])).
        flat_map(|(count, c)| iter::repeat(c).take(count as usize)).
        collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn encode_test() {
        let encode_data = [1, 1, 1, 1, 3, 3];
        let expected_result = vec![4, 1, 2, 3];

        let encoded = encode(&encode_data);

        assert_eq!(&expected_result[0..], &encoded[0..]);
    }

    #[test]
    fn decode_test() {
        let decode_data = vec![4, 1, 2, 3];
        let expected_result = [1, 1, 1, 1, 3, 3];

        let decoded = decode(&decode_data);

        assert_eq!(&expected_result[0..], &decoded[0..]);
    }
}
