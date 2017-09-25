pub fn encode(data: &[u8]) -> Vec<u8> {
    let mut i = 0;
    let mut out = Vec::with_capacity(data.len());

    loop {
        if i >= data.len() {
            break;
        };

        let mut count = 0 as usize;
        let c = data[i];

        while i < data.len() && c == data[i] && count < 255 {
            count += 1;
            i += 1;
        };

        if count < 4 {
            for _ in 0..count {
                out.push(c);
            };
        } else {
            for _ in 0..4 {
                out.push(c);
            };

            out.push(count as u8 - 4);
        };
    };

    out
}

pub fn decode(data: &Vec<u8>) -> Vec<u8> {
    let mut i = 0;
    let mut out = Vec::with_capacity(data.len() * 2);

    loop {
        if i >= data.len() {
            break;
        };

        let mut count = 0 as usize;
        let c = data[i];

        while i < data.len() && c == data[i] && count < 4 {
            count += 1;
            i += 1;
        };

        if count < 4 {
            for _ in 0..count {
                out.push(c);
            };
        } else {
            let run_count = data[i];
            i += 1;

            for _ in 0..(4 + run_count) {
                out.push(c);
            };
        };
    };

    out.shrink_to_fit();

    out
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn encode_test() {
        let encode_data = [1, 1, 1, 1, 1, 1, 3, 3];
        let expected_result = vec![1, 1, 1, 1, 2, 3, 3];

        let encoded = encode(&encode_data);

        assert_eq!(&expected_result[0..], &encoded[0..]);
    }

    #[test]
    fn decode_test() {
        let decode_data = vec![1, 1, 1, 1, 2, 3, 3];
        let expected_result = [1, 1, 1, 1, 1, 1, 3, 3];

        let decoded = decode(&decode_data);

        assert_eq!(&expected_result[0..], &decoded[0..]);
    }
}
