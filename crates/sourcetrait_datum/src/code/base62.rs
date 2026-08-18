
pub struct CodeBase62(pub String);
impl CodeBase62 {
    const ALPHABET: &[u8; 62] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

    pub const fn get(&self) -> &str { self.0.as_str() }
    pub fn take(self) -> String { self.0 }

    pub fn from_u64(mut n: u64) -> Self {
        if n == 0 { return Self(String::from("0")); }
        
        let mut buf = [0u8; 11];
        let mut i = 0;
        while n > 0 {
            buf[i] = Self::ALPHABET[(n % 62) as usize];
            n /= 62;
            i += 1;
        }
        
        buf[..i].reverse();
        let s = std::str::from_utf8(&buf[..i])
            .expect("UTF8")
            .to_string();
        
        Self(s)
    }

    pub fn conforms(s: &str) -> bool {
        !s.is_empty()
        && s.as_bytes()
            .iter()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z'))
    }
}

impl From<u64> for CodeBase62 { fn from(v: u64) -> Self { Self::from_u64(v) } }

