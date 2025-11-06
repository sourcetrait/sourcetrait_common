use crate::*;

pub trait AsTwoStr {
    fn as_two_str<'s>(&'s self) -> TwoStr<'s>;
}

// SAFETY: this is safe for hash check calls. don't store results.
// we're lying about the lifetime, which is valid so long as we don't store it.
impl<'a> Borrow<TwoStr<'a>> for TwoString {
    fn borrow(&self) -> &TwoStr<'a> {
        unsafe {
            match self {
                Self::String(s) => {
                    std::mem::transmute::<&TwoStr<'_>, &TwoStr<'a>>(
                        &TwoStr::Str(s.as_str())
                    )
                },
                Self::OsString(s) => {
                    std::mem::transmute::<&TwoStr<'_>, &TwoStr<'a>>(
                        &TwoStr::OsStr(s.as_os_str())
                    )
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::collections::HashMap;
    
    const ALPHA_STR: &'static str = "alpha";
    const BRAVO_STR: &'static str = "bravo";
    
    #[test]
    fn test_as_two_str() {
        let actual = TwoString::from_utf8_str(ALPHA_STR);
        assert_eq!(ALPHA_STR, actual.as_two_str());
    }
    
    #[test]
    fn test_borrow_hashmap() {
        let alpha = TwoString::from_utf8_str(ALPHA_STR);
        let bravo = TwoString::from_utf8_str(BRAVO_STR);
        let mut map: HashMap<TwoString, usize> = HashMap::new();
        map.insert(alpha.clone(), 1);
        map.insert(bravo.clone(), 2);
        assert_eq!(Some(&2), map.get(&bravo));
    }
}