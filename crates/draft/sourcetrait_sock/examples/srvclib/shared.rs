#![allow(dead_code)]
use sourcetrait_sock as sock;
use std::{
    ops::Deref,
};

pub struct ServiceDef(pub sock::TlsServiceDef);

impl Deref for ServiceDef {
    type Target = sock::TlsServiceDef;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl ServiceDef {
    pub const LANGUAGE_STRID: &'static str = "sourcetrait_sdk::examples::server::ClientServer";
    pub const LANGUAGE: sock::Language = sock::Language {
        strid: Self::LANGUAGE_STRID,
        version: sock::LanguageVersion(1, 1, 1),
        hashcode: 12,
    };
    
    pub const LANGUAGE_INDEX: sock::IndexedLanguage = sock::IndexedLanguage(1, &Self::LANGUAGE);
}

pub const SERVICE: ServiceDef = ServiceDef(sock::TlsServiceDef {
    languages: &[
        &ServiceDef::LANGUAGE_INDEX,
    ],
});

pub struct SetTallyRequest { pub tally: u64 }
pub struct SetTallyResponse;

pub struct AddRequest { pub operand: u64 }
pub struct AddResponse { pub tally: u64 }
