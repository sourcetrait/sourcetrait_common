#[macro_export]
macro_rules! s {
    ($s:expr) => { String::from($s) };
}

pub mod log;
pub mod num_reserve;

pub mod sync {
    pub use super::num_reserve::sync as num_reserve;
}
