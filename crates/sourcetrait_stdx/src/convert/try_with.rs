/// A vartiation of [TryFrom] that uses a transformer type (With) for more complex
/// transforms.
/// 
/// See the [TryFromTransformer] if you'd like to handle transforms generically.
pub trait TryFromWith<'w, T>
where
    Self: Sized
{
    type Error;
    type With;
    
    fn try_from_with(value: T, with: Self::With) -> ::std::result::Result<Self, Self::Error>;
}

/// A vartiation of [TryInto] that uses a transformer type (With) for more complex
/// transforms.
/// 
/// See the [TryFromTransformer] if you'd like to handle transforms generically.
pub trait TryIntoWith<'w, T> {
    type Error;
    type With;
    
    fn try_into_with(self, with: Self::With) -> ::std::result::Result<T, Self::Error>;
}

/// Anything that implements [TryFromWith] gets a complimentary [TryIntoWith],
/// similar to how [TryFrom] and [TryInto] work.
impl<'w, T, U> TryIntoWith<'w, U> for T
where
    U: TryFromWith<'w, T>,
{
    type Error = <U as TryFromWith<'w, T>>::Error;
    type With = <U as TryFromWith<'w, T>>::With;
    
    fn try_into_with(self, with: Self::With) -> ::std::result::Result<U, Self::Error> {
        U::try_from_with(self, with)
    }
}

/// Generically handles transforming types that implement [TryFromWith].
/// This isn't required to use [TryFromWith] or [TryIntoWith], but it does
/// provide an idiomatic way to handle their transformers.
/// 
/// An example implementation, retaining all generics:
/// ```rust,ignore
/// impl<'w, T, U> TryFromTransformer<'w, T, U> for MyTransformer<'w>
/// where
///    T: TryFromWith<'w, U, With = &'w MyTransformer<'w>>
/// {
///    fn try_transform_from(&'w self, value: U) -> std::result::Result<T, <T as TryFromWith<'w, U>>::Error> {
///        T::try_from_with(value, self)
///    }
/// }
/// ```
pub trait TryFromTransformer<'w, T, U>
where
    Self: 'w,
    T: TryFromWith<'w, U, With = &'w Self>
{
    fn try_transform_from(&'w self, value: U) -> std::result::Result<T, <T as TryFromWith<'w, U>>::Error>;
}
