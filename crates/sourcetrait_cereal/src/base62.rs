
const ALPHABET: &[u8; 62] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub fn base62_from_u64(mut n: u64) -> [u8; 11] {
    let mut buf = [0u8; 11];
    if n == 0 { return buf; }
    
    let mut i = 0;
    while n > 0 {
        buf[i] = ALPHABET[(n % 62) as usize];
        n /= 62;
        i += 1;
    }
    
    buf[..i].reverse();
    buf
}

pub fn is_base62(s: &str) -> bool {
    !s.is_empty()
    && s.as_bytes()
        .iter()
        .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z'))
}
