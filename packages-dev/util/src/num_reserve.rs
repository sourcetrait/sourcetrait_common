//! Reserves a number for a given numeric type.
use std::{collections::HashSet, hash::Hash};

pub trait Num: PartialEq + Eq + Copy + Hash {
    fn one() -> Self;
    fn next(&self, set: &HashSet<Self>) -> Option<Self>;
}

impl Num for u8 {
    fn one() -> Self { 1 }

    fn next(&self, set: &HashSet<Self>) -> Option<Self> {
        for i in *self..Self::MAX {
            if !set.contains(&i) {
                return Some(i);
            }
        }

        for i in 1..*self {
            if !set.contains(&i) {
                return Some(i);
            }
        }

        None
    }
}

impl Num for u16 {
    fn one() -> Self { 1 }

    fn next(&self, set: &HashSet<Self>) -> Option<Self> {
        for i in *self..Self::MAX {
            if !set.contains(&i) {
                return Some(i);
            }
        }

        for i in 1..*self {
            if !set.contains(&i) {
                return Some(i);
            }
        }

        None
    }
}


impl Num for u32 {
    fn one() -> Self { 1 }

    fn next(&self, set: &HashSet<Self>) -> Option<Self> {
        for i in *self..Self::MAX {
            if !set.contains(&i) {
                return Some(i);
            }
        }

        for i in 1..*self {
            if !set.contains(&i) {
                return Some(i);
            }
        }

        None
    }
}

impl Num for u64 {
    fn one() -> Self { 1 }

    fn next(&self, set: &HashSet<Self>) -> Option<Self> {
        for i in *self..Self::MAX {
            if !set.contains(&i) {
                return Some(i);
            }
        }

        for i in 1..*self {
            if !set.contains(&i) {
                return Some(i);
            }
        }

        None
    }
}

impl Num for u128 {
    fn one() -> Self { 1 }

    fn next(&self, set: &HashSet<Self>) -> Option<Self> {
        for i in *self..Self::MAX {
            if !set.contains(&i) {
                return Some(i);
            }
        }

        for i in 1..*self {
            if !set.contains(&i) {
                return Some(i);
            }
        }

        None
    }
}

/// Reserves a number for a given numeric type, within the range of 1 thru T::MAX.
pub struct NumReserve<T: Num> {
    pub(self) num: T,
    pub(self) claimed: HashSet<T>,
}

impl<T: Num> NumReserve<T> {
    pub fn new() -> Self {
        Self {
            num: T::one(),
            claimed: HashSet::new(),
        }
    }

    /// Gets the next available number, updates the internal index, but does not reserve it.
    /// It is not necessary to call [Self::release()] later for this number as no reservation exists.
    pub fn next(&mut self) -> Option<T> {
        match self.num.next(&mut self.claimed) {
            None => None,
            Some(num) => {
                self.num = num;
                Some(num)
            }
        }
    }

    /// Reserves the next available number.
    /// [Self::release()] should be called when the number is no longer needed.
    pub fn reserve(&mut self) -> Option<T> {
        match self.num.next(&mut self.claimed) {
            None => None,
            Some(num) => {
                self.num = num;
                self.claimed.insert(num);
                Some(num)
            }
        }
    }

    /// Release a reserved number
    pub fn release(&mut self, id: T) {
        self.claimed.remove(&id);
    }

    /// Release all reserved numbers and reset the internal index to 1
    pub fn reset(&mut self) {
        self.num = T::one();
        self.claimed.clear();
    }
}

pub mod sync {
    use std::sync::Arc;
    use std::sync::Mutex;

    pub struct NumReserve<T: super::Num> {
        locked: Arc<Mutex<super::NumReserve<T>>>
    }

    impl<T: super::Num> NumReserve<T> {
        pub fn new() -> Self {
            Self {
                locked: Arc::new(Mutex::new(super::NumReserve::new())),
            }
        }

        pub fn reserve(&self) -> Option<T> {
            let mut locked = self.locked.lock().unwrap();
            locked.reserve()
        }

        pub fn next(&self) -> Option<T> {
            let mut locked = self.locked.lock().unwrap();
            locked.next()
        }


        pub fn release(&self, id: T) {
            let mut locked = self.locked.lock().unwrap();
            locked.release(id);
        }
    }

    impl<T: super::Num> Clone for NumReserve<T> {
        fn clone(&self) -> Self {
            Self {
                locked: Arc::clone(&self.locked),
            }
        }
    }
}
