// Asmov Common Dataset: Library for application data modeling between clients and servers
// Copyright (C) 2024 Asmov LLC
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # Asmov Common Dataset :: Meta Model
//! The [Meta] model composes core attributes contained in every data model. When stored in
//! a database, this data is usually flattened.
//!
//! Each data model has:
//! - A unique [ID] that is either local, authorative, or mutual.
//! - A [Timestamp] for when the data was created and last modified.
//! - A [Hashcode] of the data in this model.
//! - A sharding ID used to segment data, typically against the owning User ID.
//!
//! Note: Hashcodes do not include most of the meta data itself.

use derive_builder;
use bincode;
use chrono;
use derivative;
use asmov_common_dataset_enum::{EnumTrait, DatasetFieldEnum, Treatment};
use crate::*;

/// Database meta information that is associated with every standalone data class / database table.
#[derive(Debug, Clone, derive_builder::Builder, derivative::Derivative, bincode::Encode, bincode::Decode)]
#[derivative(PartialEq, Eq, Hash)]
pub struct Meta {
    #[derivative(PartialEq = "ignore")]
    #[derivative(Hash = "ignore")]
    id: ID,
    #[derivative(PartialEq = "ignore")]
    #[derivative(Hash = "ignore")]
    shard_id: ID,
    #[bincode(with_serde)]
    #[derivative(PartialEq = "ignore")]
    #[derivative(Hash = "ignore")]
    time_created: Timestamp,
    #[bincode(with_serde)]
    #[derivative(PartialEq = "ignore")]
    #[derivative(Hash = "ignore")]
     time_modified: Timestamp,
    #[derivative(PartialEq = "ignore")]
    #[derivative(Hash = "ignore")]
    #[builder(default)]
    hashcode: Hashcode,
}

impl Meta {
    pub fn id(&self) -> ID {
        self.id
    }

    pub fn shard_id(&self) -> ID {
        self.shard_id
    }

    pub fn time_created(&self) -> Timestamp {
        self.time_created
    }

    pub fn time_modified(&self) -> Timestamp {
        self.time_modified
    }
}

#[derive(asmov_common_dataset_enum_derive::DatasetFieldEnum)]
pub enum MetaField {
    #[traitenum(treatment(Treatment::PrimaryMutualID), local_name("local_id"))]
    ID,
    #[traitenum(treatment(Treatment::MutualID), local_name("shard_local_id"))]
    ShardID,
    TimeCreated,
    TimeModified,
    Hashcode,
}

pub trait MetaModel: Sized + std::fmt::Debug + std::hash::Hash + Clone + ToOwned<Owned = Self> {
    type FieldEnum: DatasetFieldEnum + EnumTrait;

    const SCHEMA_NAME: &'static str;
    const SCHEMA_NAME_PLURAL: &'static str;

    fn meta(&self) -> &Meta;


    fn schema_name() -> &'static str {
        Self::SCHEMA_NAME
    }

    fn schema_name_plural() -> &'static str {
        Self::SCHEMA_NAME_PLURAL
    }

    fn id(&self) -> ID {
        self.meta().id
    }

    fn shard_id(&self) -> ID {
        self.meta().shard_id
    }

    fn time_created(&self) -> Timestamp {
        self.meta().time_created
    }

    fn time_modified(&self) -> Timestamp {
        self.meta().time_modified
    }

    fn hashcode(&self) -> Hashcode {
        self.meta().hashcode
    }

    fn schema_fields() -> <Self::FieldEnum as EnumTrait>::Iterator {
        Self::FieldEnum::iter()
    }
}

pub trait MetaModelMut: MetaModel {
    fn meta_mut(&mut self) -> &mut Meta;

    fn modified_now(&mut self) {
        self.meta_mut().time_modified = Timestamp::now();
    }

    fn rehash(&mut self) {
        self.meta_mut().hashcode = Hashcode::calculate_hash(self);
    }

    fn set_local_id(&mut self, local_id: u64) {
        if !self.id().valid() {
            self.meta_mut().id = ID::Local(local_id);
        } else { match self.id() {
            ID::Local(_) => self.meta_mut().id = ID::Local(local_id),
            ID::Authorative(id) => self.meta_mut().id = ID::Mutual(local_id, id),
            ID::Mutual(_, id) => self.meta_mut().id = ID::Mutual(local_id, id),
        }}
    }

    fn set_authorative_id(&mut self, id: u64) {
        if !self.id().valid() {
            self.meta_mut().id = ID::Authorative(id);
        } else { match self.id() {
            ID::Local(local_id) => self.meta_mut().id = ID::Mutual(local_id, id),
            ID::Authorative(_) => self.meta_mut().id = ID::Authorative(id),
            ID::Mutual(local_id, _) => self.meta_mut().id = ID::Mutual(local_id, id),
        }}
    }

    fn rehashed(mut self) -> Self {
        self.rehash();
        self
    }
}

impl MetaModelMut for Meta {
    fn modified_now(&mut self) {
        self.time_modified = chrono::Utc::now();
    }

    fn meta_mut(&mut self) -> &mut Meta {
        self
    }
}

impl MetaModel for Meta {
    type FieldEnum = MetaField;

    const SCHEMA_NAME: &'static str = "meta";
    const SCHEMA_NAME_PLURAL: &'static str = "meta";

    fn meta(&self) -> &Meta {
        self
    }
}

// schema constants
impl Meta {
    pub const SCHEMA: &'static str = "meta";
    pub const SCHEMA_PLURAL: &'static str = "meta";
    pub const SCHEMA_FIELD_USER_ID: &'static str = "shard_id";
    pub const SCHEMA_FIELD_TIME_CREATED: &'static str = "time_created";
    pub const SCHEMA_FIELD_TIME_MODIFIED: &'static str = "time_modified";
    pub const SCHEMA_FIELD_HASHCODE: &'static str = "hashcode";
}

#[cfg(feature = "postgres")]
mod postgresql  {
    use super::*;
    use sqlx::{self, Row, postgres};

    impl sqlx::FromRow<'_, postgres::PgRow> for Meta {
        fn from_row(row: &postgres::PgRow) -> sqlx::Result<Self> {
            Ok(Self {
                id: ID::Authorative(row.try_get::<i64, _>("id")? as u64),
                shard_id: ID::Authorative(row.try_get::<i64, _>("shard_id")? as u64),
                time_created: row.try_get("time_created")?,
                time_modified: row.try_get("time_modified")?,
                hashcode: row.try_get::<i64, _>("hashcode")? as u64,
            })
        }
    }
}

#[cfg(feature = "sqlite")]
mod sqlite  {
    use super::*;
    use sqlx::{self, Row, sqlite};

    impl sqlx::FromRow<'_, sqlite::SqliteRow> for Meta {
        fn from_row(row: &sqlite::SqliteRow) -> sqlx::Result<Self> {

            let local_id = row.try_get::<i64, _>("local_id")? as u64;
            let id = 0;//row.try_get::<i64, _>("id")? as u64;

            let shard_local_id = row.try_get::<i64, _>("shard_local_id")? as u64;
            let shard_id = 0 as u64;//row.try_get::<i64, _>("shard_id")? as u64;

            Ok(Self {
                id: ID::from((local_id, id)),
                shard_id: ID::from((shard_local_id, shard_id)),
                time_created: row.try_get("time_created")?,
                time_modified: row.try_get("time_modified")?,
                hashcode: row.try_get::<i64, _>("hashcode")? as u64,
            })
        }
    }
}
