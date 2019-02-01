use std::ops::Range;
use byteorder::{BigEndian, ByteOrder};
use time;

struct Perm<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Perm<'a> {
    fn get(&self, digit: usize) -> u8 {
        let idx = self.offset + digit;

        if idx < self.data.len() {
            self.data[idx]
        } else {
            self.data[idx - self.data.len()]
        }
    }
}

fn get_partitions(perms: &[Perm], p_range: &Range<usize>,
                  digit: usize, partitions: &mut Vec<Range<usize>>) {
    let mut curr_range = p_range.start..p_range.start;
    let mut prev = 0;

    while curr_range.end < p_range.end {
        let perm = &perms[curr_range.end];
        let val = perm.get(digit);

        if val != prev {
            if curr_range.len() > 1 {
                partitions.push(curr_range.clone());
            };

            curr_range.start = curr_range.end;
        };

        curr_range.end += 1;
        prev = val;
    };
    
    if curr_range.len() > 1 {
        partitions.push(curr_range);
    };
}

fn radix_sort(perms: &mut [Perm]) {
    let mut part_ranges = vec![0..perms.len()];
    let mut next_part_ranges = Vec::new();

    for digit in 0..perms.len() {
        for p_range in &part_ranges {
            perms[p_range.clone()].
                sort_unstable_by_key(|perm| perm.get(digit));

            get_partitions(perms, &p_range, digit, &mut next_part_ranges);
        };

        let mut temp = part_ranges;
        part_ranges = next_part_ranges;
        next_part_ranges = temp;
        next_part_ranges.clear();
    };
}

pub fn encode(data: &[u8]) -> Vec<u8> {
    let len = data.len();

    let start = time::now();
    let mut looped: Vec<u8> = Vec::with_capacity(len + len - 1);
    looped.extend_from_slice(data);
    looped.extend_from_slice(&data[..len-1]);

    let mut test_sorted = looped.
        windows(len).
        collect::<Vec<&[u8]>>();
    test_sorted.sort();
    println!("test sort perms in {}", time::now() - start);

    let start = time::now();
    let mut perms = (0..len).
        map(|offset| Perm { data: &data, offset: offset, }).
        collect::<Vec<Perm>>();
    radix_sort(&mut perms);
    println!("radix sort perms in {}", time::now() - start);

    let actualperms = perms.iter().
        map(|perm| &looped[perm.offset..(perm.offset + len)]).
        collect::<Vec<&[u8]>>();
    if test_sorted != actualperms {
        println!("correct: {:?}", test_sorted);
        println!("actual: {:?}", actualperms);
        panic!("sort failed");
    };

    let idx = perms.iter().
        position(|perm| perm.offset == 0).
        unwrap();

    let mut buf = Vec::with_capacity(4 + len);
    buf.append(&mut vec![0, 0, 0, 0]);

    BigEndian::write_u32(&mut buf[0..4], idx as u32);

    for perm in &perms {
        buf.push(perm.get(len - 1));
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
