use std::collections::HashMap;

fn build_initial_dictionary() -> HashMap<Vec<u8>, u16> {
    let mut dict = HashMap::new();

    for i in 0..256 {
        dict.insert(vec![i as u8], i as u16);
    };

    dict
}

pub fn encode(data: &Vec<u8>) -> Vec<u16> {
    let mut entries = build_initial_dictionary();
    let mut entriespos = 256;

    let mut encoded: Vec<u8> = Vec::new();
    let mut outvalues: Vec<u16> = Vec::new();

    for b in data {
        encoded.push(*b);

        if !entries.contains_key(&encoded) {
            let mut old_val = encoded.clone();
            let _ = old_val.pop();

            entries.insert(encoded, entriespos);
            entriespos += 1;

            outvalues.push(*entries.get(&old_val).unwrap());

            encoded = vec![*b];
        }
    }

    outvalues.push(*entries.get(&encoded).unwrap());

    outvalues
}

pub fn decode(data: &Vec<u16>) -> Vec<u8> {
    let mut entries: Vec<Vec<u8>> = (0..256).map(|i| vec![i as u8]).collect();
    let mut outbytes: Vec<u8> = Vec::new();

    let mut prev_code = data[0];
    outbytes.extend(&entries[prev_code as usize]);

    for code in data.iter().skip(1) {
        let mut val = entries[prev_code as usize].clone();

        val.push(
            if *code == entries.len() as u16 {
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
