use std::ops::Range;
use byteorder::{BigEndian, ByteOrder};
use time;

fn get_wrapped(data: &[u8], idx: usize) -> u8 {
    if idx < data.len() {
        data[idx]
    } else {
        data[idx - data.len()]
    }
}

fn get_perm_wrapped(data: &[u8], perms: &[usize], pi: usize, digit: usize) -> u8 {
    get_wrapped(data, perms[pi] + digit)
}

fn get_partitions(data: &[u8], perms: &[usize], digit: usize, base: usize) -> Vec<Range<usize>> {
    let mut pstart = 0;
    let mut prev = get_perm_wrapped(data, perms, 0, digit);
    let mut partitions = Vec::new();

    for pi in 0..perms.len() {
        let val = get_perm_wrapped(data, perms, pi, digit);
        if val != prev {
            if pi - pstart > 1 {
                partitions.push((base + pstart)..(base + pi));
            };

            pstart = pi;
            prev = val;
        };
    };
    
    if perms.len() - pstart > 1 {
        partitions.push((base + pstart)..(base + perms.len()));
    };

    partitions
}

fn radix_sort(data: &[u8], perms: &mut [usize]) {
    let mut part_ranges = vec![0..perms.len()];
    let mut next_part_ranges = Vec::new();

    for digit in 0..data.len() {
        for p_range in part_ranges {
            let partition = &mut perms[&p_range];
            partition.sort_unstable_by_key(|&perm| get_wrapped(data, perm + digit));

            let mut sub_ranges = get_partitions(data, partition, digit, p_range.start);
            next_part_ranges.append(&mut sub_ranges);
        };

        part_ranges = next_part_ranges;
        next_part_ranges = Vec::new();
    };
}

pub fn encode(data: &[u8]) -> Vec<u8> {
    let len = data.len();

    let start = time::now();
    let mut looped: Vec<u8> = Vec::with_capacity(len * 2);
    looped.extend_from_slice(data);
    looped.extend_from_slice(data);
    looped.pop();

    let mut test_sorted = looped.
        windows(len).
        collect::<Vec<&[u8]>>();
    test_sorted.sort();
    println!("test sort perms in {}", time::now() - start);

    let start = time::now();
    let mut perms = (0..len).collect::<Vec<usize>>();
    radix_sort(data, &mut perms);
    println!("radix sort perms in {}", time::now() - start);

    let actualperms = perms.iter().
        map(|&perm| &looped[perm..(perm + len)]).
        collect::<Vec<&[u8]>>();
    if test_sorted != actualperms {
        println!("correct: {:?}", test_sorted);
        println!("actual: {:?}", actualperms);
        panic!("sort failed");
    };

    let idx = perms.iter().
        position(|&perm| perm == 0).
        unwrap();

    let mut buf = Vec::with_capacity(4 + len);
    buf.append(&mut vec![0, 0, 0, 0]);

    BigEndian::write_u32(&mut buf[0..4], idx as u32);

    for &perm in &perms {
        buf.push(looped[perm + len - 1]);
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

    #[test]
    fn sort_test() {
        let mut base_data = [
            [1, 2, 3],
            [1, 1, 1],
            [2, 1, 2],
            [2, 3, 1],
        ];

        let mut data = [
            &base_data[2][..],
            &base_data[0][..],
            &base_data[2][..],
            &base_data[1][..],
            &base_data[3][..],
            &base_data[2][..],
            &base_data[2][..],
            &base_data[3][..],
            &base_data[3][..],
            &base_data[2][..],
            &base_data[3][..],
        ];

        let mut sorted = data.clone();
        bwt::quicksort(&mut sorted[..]);

        let mut refsorted = data.clone();
        refsorted.sort();

        assert_eq!(refsorted, sorted);
    }

    #[test]
    fn sort2_test() {
        let mut base_data = [
            [1],
            [2],
            [3],
            [4],
            [5],
            [6],
            [7],
            [8],
            [9],
            [10],
            [11],
            [12],
            [13],
            [14],
            [15],
        ];

        let data = [
            &base_data[14][..],
            &base_data[13][..],
            &base_data[12][..],
            &base_data[11][..],
            &base_data[10][..],
            &base_data[9][..],
            &base_data[8][..],
            &base_data[7][..],
            &base_data[6][..],
            &base_data[5][..],
            &base_data[4][..],
            &base_data[3][..],
            &base_data[2][..],
            &base_data[1][..],
            &base_data[0][..],
        ];

        let mut sorted = data.clone();
        bwt::quicksort(&mut sorted[..]);

        let mut refsorted = data.clone();
        refsorted.sort();

        assert_eq!(refsorted, sorted);
    }
}
