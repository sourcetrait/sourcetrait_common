// Asmov Common Dataset: Library for application data modeling between clients and servers
// Copyright (C) 2024 Asmov LLC
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Implementation of what we call a "Mutual ID" system for synchronizing data between clients and servers.
//!
//! Refer to the documentation for [ID] for more information.
use std::fmt::Display;
use std::sync::atomic::{AtomicU64, Ordering};
use std::hash::{Hash, Hasher};
use crate::*;

/// [ID] represents either an [ID::Local] (client-side) id, an [ID::Authorative] (server-side) id,
/// or an [ID::Mutual] (synchronized client-side) id. Each variant is used within context.
///
/// On a client application, [ID::Local] is used initially to reference data records that have not been synchronized with a server.
///
/// On a server, only [ID::Authorative] is used to reference data records. It is unaware of client-side IDs.
///
/// On a client application that has been synchronized with a server, [ID::Mutual] is used to maintain both the "local_id"
/// and the authorative "id".
///
/// Client applications will always use the "local_id" as its primary key internally and use the authorative "id" when communicating
/// with remote systems.
#[derive(Debug, Clone, Copy, Eq, bincode::Encode, bincode::Decode)]
pub enum ID {
    Local(u64),
    Authorative(u64),
    Mutual(u64, u64)
}

impl PartialEq for ID {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ID::Local(a), ID::Local(b)) => a == b,
            (ID::Authorative(a), ID::Authorative(b)) => a == b,
            (ID::Mutual(a, aa), ID::Mutual(b, bb)) => a == b || aa == bb,
            (ID::Local(a), ID::Mutual(b, _)) => a == b,
            (ID::Authorative(a), ID::Mutual(_, b)) => a == b,
            (ID::Mutual(a, _), ID::Local(b)) => a == b,
            (ID::Mutual(_, a), ID::Authorative(b)) => a == b,
            _ => false
        }
    }
}

impl ID {
    pub const AUTHORATIVE_NONE: ID = ID::Authorative(u64::MIN);
    pub const AUTHORATIVE_RESERVED: ID = ID::Authorative(u64::MAX);
    pub const LOCAL_RESERVED: ID = ID::Local(u64::MAX);
    pub const LOCAL_NONE: ID = ID::Local(u64::MIN);

    pub const LOCAL_USER: ID = ID::Local(1);
}

impl From<(u64, u64)> for ID {
    fn from(local_and_auth: (u64, u64)) -> Self {
        let (l,a) = local_and_auth;
        let local_valid = l != u64::MIN && l != u64::MAX;
        let auth_valid = a != u64::MIN && a != u64::MAX;

        match (local_valid, auth_valid) {
            (true, true) => ID::Mutual(l, a),
            (true, false) => ID::Local(l),
            (false, true) => ID::Authorative(a),
            _ => panic!("Invalid ID")
        }
    }
}

impl Into<(u64, u64)> for ID {
    fn into(self) -> (u64, u64) {
        match self {
            ID::Local(id) => (id, u64::MIN),
            ID::Authorative(id) => (u64::MIN, id),
            ID::Mutual(l, a) => (l, a),
        }
    }
}

pub enum OptionID {
    /// Database equivalent of NULL
    None,
    /// Intended for use by application developers. Logic only. Not valid for database storage.
    Reserved,
    /// Valid ID. Not NULL and not Reserved.
    Some(ID)
}

impl Hash for ID {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ID::Local(id) => { 1.hash(state) ; id.hash(state) },
            ID::Authorative(id) => { 2.hash(state) ; id.hash(state) },
            ID::Mutual(_, id) => { 2.hash(state) ; id.hash(state) },
        }
    }
}

impl ID {
    /// Returns the authorative ID value if available.
    pub fn authorative(&self) -> Option<u64> {
        match self {
            ID::Authorative(id) => Some(*id),
            ID::Mutual(_, id) => Some(*id),
            _ => None
        }
    }

    /// Returns the local ID value if available.
    pub fn local(&self) -> Option<u64> {
        match self {
            ID::Local(id) => Some(*id),
            ID::Mutual(id, _) => Some(*id),
            _ => None
        }
    }

    pub fn contextual(&self, context: ModelingContext) -> Option<u64> {
        match context {
            ModelingContext::Local => self.local(),
            ModelingContext::Authorative => self.authorative()
        }
    }

    pub fn contextual_sql(&self, context: ModelingContext) -> i64 {
        self.contextual(context).and_then(|id| Some(id as i64))
            .expect(&format!("Expected ID for context {:?}", context))
    }

    pub fn valid(&self) -> bool {
        match *self {
            Self::Authorative(u64::MIN) => false,
            Self::Authorative(u64::MAX) => false,
            Self::Authorative(_) => true,
            Self::Local(u64::MIN) => false,
            Self::Local(u64::MAX) => false,
            Self::Local(_) => true,

            #[cfg(debug_assertions)]
            Self::Mutual(l, a) if l == u64::MIN || l == u64::MAX || a == u64::MIN || a == u64::MAX
                => panic!("Mutual ID should have valid local and authorative IDs"),

            Self::Mutual(_, _) => true
        }
    }

    /// Converts this ID into an OptionID.
    pub fn into_option(self) -> OptionID {
        match self {
            Self::Authorative(u64::MIN) => OptionID::None,
            Self::Authorative(u64::MAX) => OptionID::Reserved,
            Self::Authorative(_) => OptionID::Some(self),
            Self::Local(u64::MIN) => OptionID::None,
            Self::Local(u64::MAX) => OptionID::Reserved,
            Self::Local(_) => OptionID::Some(self),

            #[cfg(debug_assertions)]
            Self::Mutual(l, a) if l == u64::MIN || l == u64::MAX || a == u64::MIN || a == u64::MAX
                => panic!("Mutual ID should have valid local and authorative IDs"),

            Self::Mutual(_, _) => OptionID::Some(self)
        }
    }
}

static LOCAL_SERIAL: AtomicU64 = AtomicU64::new(101);

pub fn init_local_id_generator(last_serial: u64) -> &'static AtomicU64 {
    LOCAL_SERIAL.store(last_serial, Ordering::Relaxed);
    &LOCAL_SERIAL
}

pub fn generate_local_id() -> ID {
    let serial = LOCAL_SERIAL.fetch_add(1, Ordering::Relaxed);
    ID::Local(serial)
}

impl Display for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id: (u64, u64) = (*self).into();
        write!(f, "{}:{}", id.0, id.1)
    }
}
