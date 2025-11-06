//use crate::*;

pub struct Buffer<const N: usize> {
    bytes: [u8; N],
    cursor: usize,
}

impl<const N: usize> Buffer<N> {
    pub fn new() -> Self {
        Self {
            bytes: [0; N],
            cursor: 0,
        }
    }
    
    pub fn as_bytes(&self) -> &[u8] { &self.bytes }
}

unsafe impl<const N: usize> bytes::BufMut for Buffer<N> {
    fn remaining_mut(&self) -> usize {
        N - self.cursor
    }

    unsafe fn advance_mut(&mut self, cnt: usize) {
        debug_assert!(self.cursor + cnt <= N);
        self.cursor += cnt;
    }

    fn chunk_mut(&mut self) -> &mut bytes::buf::UninitSlice {
        unsafe {
            bytes::buf::UninitSlice::from_raw_parts_mut(
                self.bytes.as_mut_ptr().add(self.cursor),
                N - self.cursor,
            )
        }
    }
}