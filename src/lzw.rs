use std::collections::HashMap;

fn build_initial_dictionary<'a>(arr: &'a mut [u8; 256]) -> HashMap<&'a [u8], u32> {
    let mut dict = HashMap::new();

    for i in 0..256 {
        arr[i] = i as u8;
    };

    for i in 0..256 {
        dict.insert(&arr[i..i+1], i as u32);
    };

    dict
}

pub fn encode(data: &[u8]) -> Vec<u32> {
    let mut arr = [0 as u8; 256];
    let mut entries = build_initial_dictionary(&mut arr);

    let mut encoded: &[u8] = &data[0..1];
    let mut outvalues: Vec<u32> = Vec::new();

    let mut lower = 0;
    let mut upper = 1;

    let mut out_val = 0;

    while upper < data.len() {
        while upper < data.len() {
            match entries.get(&encoded) {
                None => break,
                Some(val) => { out_val = *val; },
            };

            upper += 1;
            encoded = &data[lower..upper];
        };

        let count = entries.len() as u32;
        entries.insert(encoded, count);

        outvalues.push(out_val);

        lower = upper - 1;
        encoded = &data[lower..upper];
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
