use crate::*;

pub struct CacheValue<T: Sized> {
    value: T,
    timestamp: Instant,
}

impl<T: Sized> CacheValue<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            timestamp: Instant::now(),
        }
    }

    pub const fn value(&self) -> &T {
        &self.value
    }
    
    pub const fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    pub const fn timestamp(&self) -> Instant {
        self.timestamp
    }
}

impl<T: Debug> Debug for CacheValue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CacheValue")
            .field("value", &self.value)
            .field("timestamp", &self.timestamp)
            .finish()
    }
}

impl<T: Clone> Clone for CacheValue<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            timestamp: self.timestamp
        }
    }
}

impl<T: Copy> Copy for CacheValue<T> {}

impl<T: PartialEq> PartialEq for CacheValue<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.timestamp == other.timestamp
    }
}

impl<T: Eq> Eq for CacheValue<T> {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheMiss(Instant);

impl CacheMiss {
    pub fn new() -> Self {
        Self(Instant::now())
    }

    pub const fn timestamp(&self) -> Instant {
        self.0
    }
}

pub enum Cached<T> {
    None,
    Hit(CacheValue<T>),
    Miss(CacheMiss),
}

impl<T> Cached<T> {
    pub fn hit(value: T) -> Self {
        Cached::Hit(CacheValue::new(value))
    }
    
    pub fn miss() -> Self {
        Cached::Miss(CacheMiss::new())
    }
}

impl<T: Debug> Debug for Cached<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Hit(hit) => f.debug_tuple("Hit").field(hit).finish(),
            Self::Miss(miss) => f.debug_tuple("Miss").field(miss).finish(),
        }
    }
}

impl<T: Clone> Clone for Cached<T> {
    fn clone(&self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Hit(hit) => Self::Hit(hit.clone()),
            Self::Miss(miss) => Self::Miss(miss.clone()),
        }
    }
}

impl<T: Copy> Copy for Cached<T> {}

impl<T: PartialEq> PartialEq for Cached<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::Hit(l), Self::Hit(r)) => l == r,
            (Self::Miss(l), Self::Miss(r)) => l == r,
            _ => false,
        }
    }
}

impl<T: Eq> Eq for Cached<T> {}

pub(crate) trait CacheDeterminedTrait<T> {
    fn set(&mut self, value: T) -> CrossResult<()>;
    fn determine<F>(&mut self, determine_fn: F) -> CrossResult<&T>
    where
        F: Fn() -> CrossResult<T>;
}

pub(crate) type CacheDetermined<T> = Option<CachedResult<T>>;

impl<T> CacheDeterminedTrait<T> for CacheDetermined<T> {
    fn set(&mut self, value: T) -> CrossResult<()> {
        *self = Some(Ok(CacheValue::new(value)));
        Ok(())
    }
    
    fn determine<F>(&mut self, determine_fn: F) -> CrossResult<&T>
    where
        F: Fn() -> CrossResult<T>
    {
        match self {
            Some(Ok(cached_value)) => Ok(cached_value.value()),
            Some(Err(miss)) => Err(CrossError::from(miss)),
            None => {
                // throw the original error the first time, cached error after
                match determine_fn() {
                    Ok(v) => {
                        *self = Some(Ok(CacheValue::new(v)));
                    },
                    Err(e) => {
                        *self = Some(Err((&e).into()));
                        return Err(e);
                    }
                };
                
                Ok(self.as_ref().expect("determined").as_ref().expect("ok").value())
            }
        }
    }
}

pub enum MaybeShared<T, S> {
    Owned(T),
    Shared(S)
}

impl<T, S> MaybeShared<T, S> {
    pub const fn is_owned(&self) -> bool {
        matches!(self, Self::Owned(_))
    }
    
    pub const fn is_shared(&self) -> bool {
        matches!(self, Self::Owned(_))
    }
}

impl<T> MaybeShared<T, Arc<T>> {
    pub fn own(self) -> T
    where
        Arc<T>: Into<T>,
    {
        match self {
            Self::Owned(t) => t,
            Self::Shared(r) => r.into(),
        }
    }
    
    pub fn borrow(&self) -> &T
    where
        Arc<T>: AsRef<T>,
    {
        match self {
            Self::Owned(t) => t,
            Self::Shared(r) => r.as_ref(),
        }
    }
    
    pub fn borrow_mut(&mut self) -> &mut T
    where
        Arc<T>: AsMut<T>,
    {
        match self {
            Self::Owned(t) => t,
            Self::Shared(r) => r.as_mut(),
        }
    }
    
    pub fn share(self) -> Arc<T>
    where
        T: Clone
    {
        match self {
            Self::Owned(t) => Arc::new(t.clone()),
            Self::Shared(r) => r,
        }
    }
}

pub(crate) type StaticCache<T> = Arc<Mutex<CachedResult<T>>>;

pub(crate) fn new_static_cache_value<T>(value: T) -> StaticCache<T> {
        Arc::new(Mutex::new(Ok(CacheValue::new(value))))
}

#[allow(dead_code)]
pub(crate) fn new_static_cache<T>(result: CachedResult<T>) -> StaticCache<T> {
        Arc::new(Mutex::new(result))
}

pub(crate) type StaticCacheLock<'lock, T> =  MutexGuard<'lock, CachedResult<T>>;

#[allow(dead_code)]
pub(crate) fn cache_locked_value<'mutex, 'lock, T>(lock: &'mutex MutexGuard<'lock, CachedResult<T>>) -> CrossResult<&'mutex T> {
   lock.as_ref().map_err(CrossError::from).map(CacheValue::value)
}

pub(crate) fn cache_locked_value_mut<'mutex, 'lock, T>(lock: &'mutex mut MutexGuard<'lock, CachedResult<T>>) -> CrossResult<&'mutex mut T> {
   lock.as_mut().map_err(CrossError::from).map(CacheValue::value_mut)
}

