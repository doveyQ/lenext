fn pad(message_len_bytes: usize) -> Vec<u8> {
    let mut padding: Vec<u8> = Vec::new();
    
    // Add one byte after the message
    padding.push(0x80);

    // Add the remaining 0 bytes
    while (message_len_bytes + padding.len() + 8) % 16 != 0 {
        padding.push(0x00);
    }
    
    // Fill in the length as 8 bytes at the end
    let len_bytes = ((message_len_bytes as u64) * 8).to_be_bytes();
    padding.extend_from_slice(&len_bytes);


    return padding;
}

// Simple ARX (addition, rotate, XOR)
fn encrypt(key: &[u8; 16], state: u64) -> u64 {
    let bytes_a: [u8; 8] = key[0..8].try_into().unwrap();
    let bytes_b: [u8; 8] = key[8..16].try_into().unwrap();

    let key_int_a = u64::from_be_bytes(bytes_a);
    let key_int_b = u64::from_be_bytes(bytes_b);
    let subkeys = [key_int_a, key_int_b];
    let mut s = state;

    for round in 0..32 {
        let k = subkeys[round % 2];

        s = s.wrapping_add(k);
        s ^= s.rotate_left(7);
        s ^= round as u64;
    }
    return s;
}

// Davies-Meyer compression function
fn compress(state: u64, block: &[u8; 16]) -> u64 {
    return encrypt(block, state) ^ state;
} 

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = pad(5);
        assert_eq!(result, vec![0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x28]);    }
}
