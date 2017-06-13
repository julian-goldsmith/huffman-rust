use std::collections::HashMap;

fn build_initial_dictionary() -> HashMap<Vec<u8>, u32> {
    let mut dict = HashMap::new();

    for i in 0..256 {
        dict.insert(vec![i as u8], i as u32);
    };

    dict
}

pub fn encode(data: &[u8]) -> Vec<u32> {
    let mut entries = build_initial_dictionary();

    let mut encoded: Vec<u8> = vec![data[0]];
    let mut outvalues: Vec<u32> = Vec::new();

    let mut i = 1;

    let mut out_val = 0;

    while i < data.len() {
        while i < data.len() {
            match entries.get(&encoded) {
                None => break,
                Some(val) => { out_val = *val; },
            };

            encoded.push(data[i]);
            i += 1;
        };

        let count = entries.len() as u32;
        entries.insert(encoded, count);

        outvalues.push(out_val);

        encoded = vec![data[i - 1]];
    };

    match entries.get(&encoded) {
        None => panic!("Couldn't get entry"),
        Some(val) => outvalues.push(*val),
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
