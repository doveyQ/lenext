
/// Fixed initial state for the toy hash. Any constant works.
const IV: u64 = 0x0123_4567_89ab_cdef;

/// MD padding: 0x80, zeros, then the length in bits (big-endian u64).
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


    padding
}

/// Toy 64-bit ARX block cipher, 32 rounds.
fn encrypt(key: &[u8; 16], mut state: u64) -> u64 {
    let bytes_a: [u8; 8] = key[0..8].try_into().unwrap();
    let bytes_b: [u8; 8] = key[8..16].try_into().unwrap();

    let key_int_a = u64::from_be_bytes(bytes_a);
    let key_int_b = u64::from_be_bytes(bytes_b);
    let subkeys = [key_int_a, key_int_b];

    for round in 0..32 {
        let k = subkeys[round % 2];

        // Mix in the round key; addition makes the cipher non-linear
        state = state.wrapping_add(k);
        // Spread each bit's influence to a bit 7 positions away
        state ^= state.rotate_left(7);
        // Round number makes every round different (prevents slide attacks)
        state ^= round as u64;
    }
    state
}


/// Davies–Meyer: E_M(H) ⊕ H.
fn compress(state: u64, block: &[u8; 16]) -> u64 {
    encrypt(block, state) ^ state
} 

/// Merkle–Damgård loop from any start state; `length` is used for padding.
fn hash_from(mut state: u64, data: &[u8], length: usize) -> u64 {
    let mut padded_input = data.to_vec();
    padded_input.extend_from_slice(&pad(length));

    let blocks = padded_input.chunks_exact(16);
    for block in blocks {
        state = compress(state, block.try_into().unwrap());
    }
    state
}

/// Toy hash from the fixed IV.
fn hash(data: &[u8]) -> u64 {
    hash_from(IV, data, data.len())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pad_works() {
        let result = pad(5);
        assert_eq!(result, vec![0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x28]);    
    }
}
