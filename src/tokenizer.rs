pub const VOCAB_SIZE: usize = 256;

pub fn encode(text: &str) -> Vec<u8> {
    text.as_bytes().to_vec()
}

pub fn decode(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let text = "RICK: Wubba lubba dub dub!\n";
        let encoded = encode(text);
        let decoded = decode(&encoded);
        assert_eq!(decoded, text);
    }

    #[test]
    fn test_vocab_size() {
        assert_eq!(VOCAB_SIZE, 256);
    }

    #[test]
    fn test_encode_length() {
        let text = "abc";
        assert_eq!(encode(text).len(), 3);
    }
}
