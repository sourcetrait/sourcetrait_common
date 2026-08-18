use crate::*;

/// Represents a XXH364 hash.
/// Conversions occur as little-endian where there is a choice.
/// Implements both From and const `from_` conversions for key core types.
#[cereal::derived(Data, Copy, Eq)]
pub struct Nom(pub u64);
impl Nom {
    pub const fn get(&self) -> u64 { self.0 }
    pub const fn take(self) -> u64 { self.0 }

    pub const fn from_u64(n: u64) -> Self { Self(xxh3_64(&n.to_le_bytes())) }
    pub const fn from_i64(n: i64) -> Self { Self(xxh3_64(&n.to_le_bytes())) }
    pub const fn from_f64(n: f64) -> Self { Self(xxh3_64(&n.to_le_bytes())) }
    pub const fn from_u32(n: u32) -> Self { Self(xxh3_64(&n.to_le_bytes())) }
    pub const fn from_i32(n: i32) -> Self { Self(xxh3_64(&n.to_le_bytes())) }
    pub const fn from_f32(n: f32) -> Self { Self(xxh3_64(&n.to_le_bytes())) }
    pub const fn from_u16(n: u16) -> Self { Self(xxh3_64(&n.to_le_bytes())) }
    pub const fn from_i16(n: i16) -> Self { Self(xxh3_64(&n.to_le_bytes())) }
    pub const fn from_u8(n: u8) -> Self { Self(xxh3_64(&n.to_le_bytes())) }
    pub const fn from_i8(n: i8) -> Self { Self(xxh3_64(&n.to_le_bytes())) }
    pub const fn from_usize(n: usize) -> Self { Self(xxh3_64(&n.to_le_bytes())) }
    
    pub const fn from_str(s: &str) -> Self { Self(xxh3_64(s.as_bytes())) }
    
    pub fn to_base62(&self) -> CodeBase62 { CodeBase62::from_u64(self.0) }

    pub fn to_string(&self) -> String { self.to_base62().take() }
}

impl From<u64> for Nom { fn from(v: u64) -> Self { Self::from_u64(v) } }
impl From<i64> for Nom { fn from(v: i64) -> Self { Self::from_i64(v) } }
impl From<f64> for Nom { fn from(v: f64) -> Self { Self::from_f64(v) } }
impl From<u32> for Nom { fn from(v: u32) -> Self { Self::from_u32(v) } }
impl From<i32> for Nom { fn from(v: i32) -> Self { Self::from_i32(v) } }
impl From<f32> for Nom { fn from(v: f32) -> Self { Self::from_f32(v) } }
impl From<u16> for Nom { fn from(v: u16) -> Self { Self::from_u16(v) } }
impl From<i16> for Nom { fn from(v: i16) -> Self { Self::from_i16(v) } }
impl From<u8> for Nom { fn from(v: u8) -> Self { Self::from_u8(v) } }
impl From<i8> for Nom { fn from(v: i8) -> Self { Self::from_i8(v) } }
impl From<usize> for Nom { fn from(v: usize) -> Self { Self::from_usize(v) } }

impl From<&str> for Nom { fn from(v: &str) -> Self { Self::from_str(v) } }
impl From<String> for Nom { fn from(v: String) -> Self { Self::from_str(v.as_str()) } }

impl From<&Path> for Nom { fn from(v: &Path) -> Self { Self::from_str(v.to_str().expect("UTF8")) } }
impl From<PathBuf> for Nom { fn from(v: PathBuf) -> Self { Self::from_str(v.to_str().expect("UTF8")) } }

