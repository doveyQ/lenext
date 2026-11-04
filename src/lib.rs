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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = pad(5);
        assert_eq!(result, vec![0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x28]);    }
}
