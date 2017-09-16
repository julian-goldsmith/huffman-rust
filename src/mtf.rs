pub fn encode(data: &[u8]) -> Vec<u8> {
    let mut vals = (0..256 as usize).map(|i| i as u8).collect::<Vec<u8>>();

    let mut out = Vec::with_capacity(data.len());

    for &c in data {
        let idx = vals.iter().position(|&v| v == c).unwrap();

        vals.remove(idx);
        vals.insert(0, c);

        out.push(idx as u8);;
    };

    out
}

pub fn decode(data: &[u8]) -> Vec<u8> {
    let mut vals = (0..256 as usize).map(|i| i as u8).collect::<Vec<u8>>();

    let mut out = Vec::with_capacity(data.len());

    for &i in data {
        out.push(vals[i as usize]);

        let c = vals.remove(i as usize);
        vals.insert(0, c);
    };

    out
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn encode_test() {
        let data = [2, 2, 2, 2, 1, 5, 3, 2];
        let expected_result = [2, 0, 0, 0, 2, 5, 4, 3];

        let result = encode(&data);

        assert_eq!(&expected_result, &result[0..]);
    }

    #[test]
    fn decode_test() {
        let data = [2, 0, 0, 0, 2, 5, 4, 3];
        let expected_result = [2, 2, 2, 2, 1, 5, 3, 2];

        let result = decode(&data);

        assert_eq!(&expected_result, &result[0..]);
    }
}
