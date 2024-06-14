// Asmov Common Dataset: Library for application data modeling between clients and servers
// Copyright (C) 2024 Asmov LLC
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # Asmov Common Dataset :: Model
//! 
//! Concepts:
//! - [ID](id) A mutual ID system for synchronizing data between clients and servers in offline and online modes.
//! - [Meta](meta) Composes core attributes for every data model
//! 
//! We use UTC [Timestamps](Timestamp) for all time-related fields.
//! 
//! We use a [Hashcode] to represent a unique hash of each data model, which is used for synchronization and caching.

pub mod id;
pub mod meta;

use std::hash::{self, Hash, Hasher};

pub type Timestamp = chrono::DateTime<chrono::Utc>;

pub trait TimestampTrait {
    fn new(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32, nanosecond: u32) -> Option<Timestamp> {
        Some(chrono::NaiveDate::from_ymd_opt(year, month, day)?
            .and_hms_nano_opt(hour, minute, second, nanosecond)?
            .and_utc())
    }

    fn now() -> Timestamp {
        chrono::Utc::now()
    }
}

impl TimestampTrait for Timestamp {}

pub type Hashcode = u64;

pub trait HashcodeTrait {
    fn calculate_hash<T: crate::MetaModel>(model: &T) -> Hashcode {
        let mut hasher = hash::DefaultHasher::new();
        model.hashcode().hash(&mut hasher);
        hasher.finish()
    }
}

impl HashcodeTrait for Hashcode {}

