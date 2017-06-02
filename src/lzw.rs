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

    let mut encoded: Vec<u8> = Vec::new();
    let mut outvalues: Vec<u32> = Vec::new();

    for b in data {
        encoded.push(*b);

        if !entries.contains_key(&encoded) {
            let old_val = Vec::from(&encoded[0..encoded.len()-1]);

            let count = entries.len() as u32;
            entries.insert(encoded, count);

            match entries.get(&old_val) {
                None => panic!("Couldn't get entry"),
                Some(val) => outvalues.push(*val),
            };

            encoded = vec![*b];
        }
    }

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
