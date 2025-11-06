use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LanguageVersion(pub u16, pub u16, pub u16);

impl LanguageVersion {
    pub const fn major(&self) -> u16 { self. 0 }
    pub const fn minor(&self) -> u16 { self. 0 }
    pub const fn patch(&self) -> u16 { self. 0 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Language {
    pub strid: &'static str,
    pub version: LanguageVersion,
    pub hashcode: u64,
}

impl Language {
    pub const fn hashcode(&self) -> u64 { self.hashcode }
    
    #[cfg(debug_assertions)]
    fn generate_hashcode() -> u64 {
        todo!()
    }
}

impl Hash for Language {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hashcode.hash(state);
    }
}

pub struct TlsServiceDef {
    pub languages: &'static [&'static IndexedLanguage],
}

impl TlsServiceDef {
    pub fn lookup_id(&self, language_id: u8) -> Option<&'static Language> {
        self.languages.iter()
            .find(|pair| pair.0 == language_id)
            .map(|p| p.1)
    }
    
    pub fn lookup_strid(&self, language_strid: &str) -> Option<&'static Language> {
        self.languages.iter()
            .find(|pair| pair.1.strid == language_strid)
            .map(|p| p.1)
    }
}

pub struct IndexedLanguage(pub u8, pub &'static Language);