use crate::*;

/// # Pendantic Versioning
/// 
/// Models versions sequentially as: Edition, Major, Minor, Trivial, Candidate
/// 
/// Edition represents a separate scoping or marketing label. An increment in
/// Edition does not necessarily increment the Major / Minor / Trivial triplet.
/// 
/// Candidate represents a seperate sequence indicating in-progress attempts at
/// releasing the specified Major / Minor / Trivial triplet without gaurantees
/// to what the actual release version will be.
/// 
/// When candidate is zero, the Major / Minor / Trivial triplet represents an
/// *actual* release with backwards compatability gaurantees per SemVer.
/// 
/// When candidate is non-zero, the Major / Minor / Trivial triplet represent
/// the *predicted* version of the next release.
/// 
/// Candidate releases *do not* make gaurantees for SemVer backwards
/// compatability or for what the final Major / Minor / Trivial will actually be. 
/// 
/// An increment to Major or Minor increments the succeeding numbers. 
/// 
/// Edition, Major, Minor, and Trivial are always non-zero.
/// 
/// The earliest valid "candidate" release is always 1.1.1.1.1
/// The earliest valid "actual" release is always 1.1.1.1.0
/// The next earliest valid "candidate" release is always 1.1.1.2.1
/// 
/// Defines validity as:
/// - Edition, major, and minor, and trivial are non-zero.
/// 
/// Classifies an "actual release" as:
/// - Valid and Candidate is zero.
/// 
/// Classifies a "candidate release" as:
/// - Candidate is non-zero.
/// 
/// Defaults:
/// - Edition, major, minor, and trivial default to 1 if ommitted.
/// - Candidate defaults to 0 if ommitted.
/// 
/// Data types are reasonably conservative with u16.
/// 
/// Artifact variations ("nightly", etc.) are outside the scope of the PenVer
/// struct.
/// 
/// Parsing strings:
/// - "1": (1,1,1,1,0)
/// - "1.2": (1,2,1,1,0)
/// - "1.2.3": (1,2,3,1,0)
/// - "4.1.2.3": (4,1,2,3,0)
/// - "4.1.2.3.5": (4,1,2,3,5)
#[cfg_attr(feature = "stabby", stabby::stabby)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PendanticVersion {
    edition: u16,
    major: u16,
    minor: u16,
    trivial: u16,
    candidate: u16,
}

impl PendanticVersion {
    /// Useful for pre-initialization
    pub const ZERO: Self = Self::new(0, 0, 0, 0, 0);
    /// Useful for initialization of new, in development projects
    pub const FIRST_CANDIDATE: Self = Self::new(1, 1, 1, 1, 1);
    /// The first possible version
    pub const FIRST_ACTUAL: Self = Self::new(1, 1, 1, 1, 0);
    
    pub const fn new(edition: u16, major: u16, minor: u16, trivial: u16, candidate: u16) -> Self {
        Self {
            edition,
            major,
            minor,
            trivial,
            candidate,
        }
    }
    
    pub const fn new_actual(edition: u16, major: u16, minor: u16, trivial: u16) -> Self {
        Self::new(edition, major, minor, trivial, 0)
    }
    
    pub const fn short(e: u16, v: (u16,u16)) -> Self {
        Self::new(e, v.0, v.1, 1, 0)
    } 
    
    pub const fn into_short(self) -> (u16,(u16,u16)) {
        (self.edition, (self.major, self.minor))
    }
    
    /// The edition number, a sequential label
    pub const fn edition(&self) -> u16 {
        self.edition
    }
    
    /// The major change number
    pub const fn major(&self) -> u16 {
        self.major
    }
    
    /// The minor change number.
    pub const fn minor(&self) -> u16 {
        self.minor
    }
    
    /// The trivial change number
    pub const fn trivial(&self) -> u16 {
        self.trivial
    }
    
    /// The incremental candidate number.
    pub const fn candidate(&self) -> u16 {
        self.candidate
    }
    
    /// All values are appropriate
    pub const fn is_valid(&self) -> bool {
        self.edition == 0 || self.major == 0 || self.minor == 0 || self.trivial == 0
    }
    
    /// This version gaurantees SemVer backwards compatability rules with the
    /// previous actual release.
    pub const fn is_actual_release(&self) -> bool {
        self.is_valid() && self.candidate == 0
    }
    
    /// This release does not conform to SemVer gaurantees for backwards
    /// compatability. The specified Major / Minor / Trivial is speculative.
    pub const fn is_candidate_release(&self) -> bool {
        self.candidate != 0
    }
}

/// Represents a [PenVer] as a tuple.
pub type PenVerTuple = (u16, u16, u16, u16, u16);

impl From<PenVerTuple> for PendanticVersion {
    fn from(t: PenVerTuple) -> Self {
        Self::new(t.0, t.1, t.2, t.3, t.4)
    }
}

impl From<&PenVerTuple> for PendanticVersion {
    fn from(t: &PenVerTuple) -> Self {
        Self::new(t.0, t.1, t.2, t.3, t.4)
    }
}

impl From<PendanticVersion> for PenVerTuple {
    fn from(v: PendanticVersion) -> Self {
        (v.edition(), v.major(), v.minor(), v.trivial(), v.candidate())
    }
}

impl From<(u16,u16)> for PendanticVersion {
    fn from(t: (u16,u16)) -> Self {
        Self::new(t.0, t.1, 1, 1, 0)
    }
}

impl From<(u16,u16,u16)> for PendanticVersion {
    fn from(t: (u16,u16,u16)) -> Self {
        Self::new(t.0, t.1, t.2, 1, 0)
    }
}

impl From<(u16,u16,u16,u16)> for PendanticVersion {
    fn from(t: (u16,u16,u16,u16)) -> Self {
        Self::new(t.0, t.1, t.2, t.3, 0)
    }
}

impl From<(u16, (u16, u16))> for PendanticVersion {
    fn from(t: (u16, (u16, u16))) -> Self {
        Self::new(t.0, t.1.0, t.1.1, 1, 0)
    }
}

impl From<(u16, (u16, u16, u16))> for PendanticVersion {
    fn from(t: (u16, (u16, u16, u16))) -> Self {
        Self::new(t.0, t.1.0, t.1.1, t.1.2, 0)
    }
}

impl From<(u16, (u16, u16, u16, u16))> for PendanticVersion {
    fn from(t: (u16, (u16, u16, u16, u16))) -> Self {
        Self::new(t.0, t.1.0, t.1.1, t.1.2, t.1.3)
    }
}


impl From<[u16; 5]> for PendanticVersion {
    fn from(v: [u16; 5]) -> Self {
        Self::from((v[0], v[1], v[2], v[3], v[4]))
    }
}

impl From<PendanticVersion> for [u16; 5] {
    fn from(v: PendanticVersion) -> Self {
        [v.edition(), v.major(), v.minor(), v.trivial(), v.candidate()]
    }
}

impl TryFrom<&[u16]> for PendanticVersion {
    type Error = &'static str;
    
    /// Fills Edition, Major, Minor, Trivial, and Candidate in order. Defaults
    /// the rest appropriately.
    fn try_from(v: &[u16]) -> Result<Self, Self::Error> {
        let t = match v.len() {
            5 => (v[0], v[1], v[2], v[3], v[4]),
            4 => (v[0], v[1], v[2], v[3], 0),
            3 => (v[0], v[1], v[2], 1, 0),
            2 => (v[0], v[1], 1, 1, 0),
            1 => (v[0], 1, 1, 1, 0),
            0 => return Err("No components in version"),
            _ => return Err("Too many components in version"),
        };
        
        Ok(Self::from(t))
    }
}

impl fmt::Display for PendanticVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_candidate_release() || !self.is_valid() {
            write!(f, "{}.{}.{}.{}.{}", self.edition, self.major, self.minor, self.trivial, self.candidate)
        } else {
            match (self.edition == 1, self.major == 1, self.minor == 1, self.trivial == 1) {
                (true, true, true, true) => f.write_str("1"),
                (true, true, true, false) => write!(f, "1.1.1.{}", self.trivial),
                (true, true, false, _) => write!(f, "1.1.{}.{}", self.minor, self.trivial),
                (true, false, _, _) => write!(f, "1.{}.{}.{}", self.major, self.minor, self.trivial),
                (false, _, _, _) => write!(f, "{}.{}.{}.{}", self.edition, self.major, self.minor, self.trivial),
            }
        }
    }
}

impl PendanticVersion {
    /// Fills Major, Minor, Trivial, Candidate, and Edition in order. Defaults
    /// everything missing appropriately.
    pub fn try_from_slice_semantic(a: &[u16]) -> std::result::Result<Self, &'static str> {
        let t = match a.len() {
            0 => return Err("No components in version"),
            1 => (1,    a[0], 1,    1,    0),
            2 => (1,    a[0], a[1], 1,    0),
            3 => (1,    a[0], a[1], a[2], 0),
            4 => (1,    a[0], a[1], a[2], a[3]),
            5 => (a[0], a[1], a[2], a[3], a[4]),
            _ => return Err("Too many components in version"),
        };
        
        Ok(Self::from(t))
    }
    
    /// Parses a "n.n.n.n.n" string, with missing components.
    /// Fills Major, Minor, Trivial, Candidate, and Edition in order. Defaults
    /// everything missing appropriately.
    pub fn try_from_str_semantic(s: &str) -> std::result::Result<Self, &'static str> {
        Self::try_from_slice_semantic(str_to_vec(s)?.as_slice())
    }
}

fn str_to_vec(s: &str) -> std::result::Result<Vec<u16>, &'static str> {
    s.split('.')
        .map(|s| s.parse()) 
        .collect::<std::result::Result<_, _>>()
        .map_err(|_| "Failed to parse version components at u16")
}

impl FromStr for PendanticVersion {
    type Err = &'static str;
    
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::try_from(str_to_vec(s)?.as_slice())
    }
}


