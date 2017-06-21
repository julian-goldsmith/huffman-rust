use std::mem;
use std::ptr;

const NUM_BUCKETS: usize = 1024;

struct Bucket {
    pub keys: Vec<u64>,
    pub values: Vec<u32>,
}

struct HashTable {
    pub buckets: [Bucket; NUM_BUCKETS],
    pub count: u32,
}

impl HashTable {
    pub fn new() -> HashTable {
        unsafe {
            let mut table: HashTable = mem::uninitialized();

            for (_, element) in table.buckets.iter_mut().enumerate() {
                let bucket = Bucket { keys: Vec::new(), values: Vec::new() };

                ptr::write(element, bucket);
            };

            table
        }
    }

    pub fn insert(&mut self, key: &[u8], value: u32) {
        let hash = fnv_hash(key);
        let bi = hash as usize & (NUM_BUCKETS - 1);
        let mut bucket = &mut self.buckets[bi];

        assert_eq!(bucket.keys.len(), bucket.values.len());

        bucket.keys.push(hash);
        bucket.values.push(value);
        self.count += 1;
    }

    pub fn get_by_hash(&self, hash: u64) -> Option<u32> {
        let bi = hash as usize & (NUM_BUCKETS - 1);
        let bucket = &self.buckets[bi];

        for i in 0..bucket.keys.len() {
            if bucket.keys[i] == hash {
                return Some(bucket.values[i]);
            }
        }

        return None;
    }
}

#[inline(always)]
fn fnv_hash_partial(h: u64, b: u8) -> u64 {
    (h ^ b as u64).wrapping_mul(0x100000001b3)
}

const FNV_BASE: u64 = 0xcbf29ce484222325;

fn fnv_hash(key: &[u8]) -> u64 {
    let mut h = FNV_BASE;

    for b in key {
        h = fnv_hash_partial(h, *b);
    };

    h
}

fn build_initial_dictionary<'a>(table: &mut HashTable) {
    let mut arr = [0 as u8; 256];

    for i in 0..256 {
        arr[i] = i as u8;
    };

    for i in 0..256 {
        table.insert(&arr[i..i+1], i as u32);
    };
}

pub fn encode(data: &[u8]) -> Vec<u32> {
    let mut table = HashTable::new();
    build_initial_dictionary(&mut table);

    let mut encoded: &[u8] = &data[0..1];
    let mut outvalues: Vec<u32> = Vec::new();

    let mut lower = 0;
    let mut upper = 1;

    let mut out_val = 0;

    while upper < data.len() {
        let mut hash = FNV_BASE;

        while upper < data.len() {
            hash = fnv_hash_partial(hash, data[upper - 1]);

            match table.get_by_hash(hash) {
                None => break,
                Some(val) => { out_val = val; },
            };

            upper += 1;
            encoded = &data[lower..upper];
        };

        let count = table.count;
        table.insert(encoded, count);

        outvalues.push(out_val);

        lower = upper - 1;
        encoded = &data[lower..upper];
    };

    let hash = fnv_hash(encoded);
    match table.get_by_hash(hash) {
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
