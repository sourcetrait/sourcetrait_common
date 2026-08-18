use crate::*;

pub type NonceStr = [u8; 11];

#[cereal::derived(Data, Copy, Eq, Hash)]
pub struct Nonce(pub(crate) u64);
impl Nonce {
    pub const fn new_unchecked(hash: u64) -> Self { Self(hash) }
    pub const fn get(&self) -> u64 { self.0 }
    pub const fn take(self) -> u64 { self.0 }
    
    pub fn generate(generator: &NonceGenerator) -> Self { generator.generate() }
    pub fn generate_with<T: Hash>(generator: &NonceGenerator, hashable: &T) -> Self {
        generator.generate_with(&hashable)
    }
    
    pub fn into_pair(self) -> NoncePair { NoncePair(self.0, cereal::base62_from_u64(self.0)) }
}

impl std::fmt::Display for Nonce {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.into_pair().str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoncePair(pub(crate) u64, pub(crate) NonceStr);
impl NoncePair {
    pub const fn new_unchecked(nonce: u64, nonce_str: NonceStr) -> Self { Self(nonce, nonce_str) }
    pub const fn from_tuple_unchecked(tuple: (u64, NonceStr)) -> Self { Self(tuple.0, tuple.1) }
    pub const fn take(self) -> (u64, NonceStr) { (self.0, self.1) }
    pub const fn nom(&self) -> u64 { self. 0 }
    pub const fn str(&self) -> &str { unsafe { str::from_utf8_unchecked(&self.1) } }
    pub const fn str_bytes(&self) -> &NonceStr { &self.1 }
    pub const fn to_nonce(&self) -> Nonce { Nonce(self.0) }
    pub const fn into_nonce(self) -> Nonce { Nonce(self.0) }
}

pub struct NonceGenerator {
    counter: AtomicUsize,
}

impl NonceGenerator {
    pub const fn new() -> Self { Self { counter: AtomicUsize::new(0) } }

    pub fn generate_with<T: Hash>(&self, hashable: &T) -> Nonce {
        let mut hasher = cereal::DefaultHasher::new();
        hashable.hash(&mut hasher);
        self.counter
            .fetch_add(1, atomic::Ordering::SeqCst)
            .hash(&mut hasher);
        let time_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        time_ns.hash(&mut hasher);
        Nonce(hasher.digest())
    }
    
    pub fn generate(&self) -> Nonce { self.generate_with(&()) }
}
