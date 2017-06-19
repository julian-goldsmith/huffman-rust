use std::collections::HashMap;
use std::mem;
use std::ptr;

const NUM_BUCKETS: usize = 1024;

struct Bucket {
    pub keys: Vec<u64>,
    pub values: Vec<u32>,
}

fn create_hash_table() -> [Bucket; NUM_BUCKETS] {
    unsafe {
        let mut buckets: [Bucket; NUM_BUCKETS] = mem::uninitialized();

        for (i, element) in buckets.iter_mut().enumerate() {
            let bucket = Bucket { keys: Vec::new(), values: Vec::new() };

            ptr::write(element, bucket);
        };

        buckets
    }
}

fn hash_insert(buckets: &mut [Bucket; NUM_BUCKETS], key: &[u8], value: u32) {
    let hash = fnv_hash(key);
    let bi = hash as usize & (NUM_BUCKETS - 1);
    let mut bucket = &mut buckets[bi];

    assert_eq!(bucket.keys.len(), bucket.values.len());

    bucket.keys.push(hash);
    bucket.values.push(value);
}

fn hash_get_by_hash(buckets: &[Bucket; NUM_BUCKETS], hash: u64) -> Option<u32> {
    let bi = hash as usize & (NUM_BUCKETS - 1);
    let bucket = &buckets[bi];

    for i in 0..bucket.keys.len() {
        if bucket.keys[i] == hash {
            return Some(bucket.values[i]);
        }
    }

    return None;
}

fn fnv_hash(key: &[u8]) -> u64 {
    let mut h = 0xcbf29ce484222325;

    for b in key {
        h = (h ^ *b as u64).wrapping_mul(0x100000001b3);
    };

    h
}

fn build_initial_dictionary<'a>(mut buckets: &mut [Bucket; NUM_BUCKETS]) {
    let mut arr = [0 as u8; 256];

    for i in 0..256 {
        arr[i] = i as u8;
    };

    for i in 0..256 {
        //dict.insert(&arr[i..i+1], i as u32);
        hash_insert(&mut buckets, &arr[i..i+1], i as u32);
    };
}

pub fn encode(data: &[u8]) -> Vec<u32> {
    let mut buckets = create_hash_table();
    build_initial_dictionary(&mut buckets);

    let mut count = 256;

    let mut encoded: &[u8] = &data[0..1];
    let mut outvalues: Vec<u32> = Vec::new();

    let mut lower = 0;
    let mut upper = 1;

    let mut out_val = 0;

    while upper < data.len() {
        while upper < data.len() {
            let hash = fnv_hash(encoded);
            match hash_get_by_hash(&buckets, hash) {
                None => break,
                Some(val) => { out_val = val; },
            };

            upper += 1;
            encoded = &data[lower..upper];
        };

        //entries.insert(encoded, count);
        hash_insert(&mut buckets, encoded, count);
        count += 1;

        outvalues.push(out_val);

        lower = upper - 1;
        encoded = &data[lower..upper];
    };

    let hash = fnv_hash(encoded);
    match hash_get_by_hash(&buckets, hash) {
        None => panic!("Couldn't get entry"),
        Some(val) => outvalues.push(val),
    };

    outvalues
}

pub fn decode(data: &Vec<u32>) -> Vec<u8> {
    let mut entries: Vec<Vec<u8>> = (0..256).map(|i| vec![i as u8]).collect();
    let mut outbytes: Vec<u8> = Vec::new();

    let mut prev_code = data[0];                                    // FIXME: can panic here
    outbytes.extend(&entries[prev_code as usize]);

    for code in data.iter().skip(1) {
        let mut val = entries[prev_code as usize].clone();

        val.push(
            if *code == entries.len() as u32 {
                entries[prev_code as usize][0]
            } else {
                entries[*code as usize][0]
            });

        entries.push(val);

        outbytes.extend(&entries[*code as usize]);
        prev_code = *code;
    };

    outbytes
}
