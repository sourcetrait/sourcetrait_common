// Asmov Common Dataset: Library for application data modeling between clients and servers
// Copyright (C) 2024 Asmov LLC
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # Asmov Common Dataset
//! *A library for application data modeling between clients and servers.*
//!
//! Core concepts:
//! - [Dataset](crate::dataset) Represents a collection of data models, with basic read/write/delete operations.
//! - [Model](crate::model) Standardizes how we represent data models.
//! - [Client(create::model)] The app-facing fascade that provides easy access to datasets.
pub mod error;
pub mod dataset;
pub mod client;

pub mod macros {
    pub mod imprint;
    pub use imprint::*;
}

pub use error::{Error, Result};

pub mod model;

pub mod ext {
    pub use rustc_hash;
}

pub use crate::{
    model::{
        Timestamp, TimestampTrait, Hashcode, HashcodeTrait,
        meta::{Meta, MetaModel, MetaModelMut, MetaBuilder, MetaBuilderError, MetaField},
        id::{ID, OptionID, init_local_id_generator, generate_local_id},
    },
    dataset::{
        Dataset, DatasetModel, DatasetDirect, DatasetIndirect, DatasetDirectModel, DatasetIndirectModel,
        memory::MemoryDataset,
        strategy::{StrategicDataset, Strategy, StandardAppStrategy, StrategicDatasetModel}
    }
};

pub use derive_builder;

#[cfg(feature = "sql")]
pub use crate::sql::{ToArguments, SqlxArgumentsExtended, ArgumentFormatType, ArgumentFormat};

pub use asmov_common_dataset_enum::{DatasetFieldEnum, DatasetFieldEnumComposite, EnumTrait, Treatment, ModelingContext};

pub mod derive {
    pub use asmov_common_dataset_enum_derive::DatasetFieldEnum;
}

pub mod prelude {
    pub use crate::{ModelingContext, MetaModel, MetaModelMut, MetaBuilder, Dataset, DatasetDirect, DatasetIndirect, DatasetModel,
        DatasetDirectModel, DatasetIndirectModel, StrategicDataset, Strategy, StandardAppStrategy, StrategicDatasetModel,
        TimestampTrait, HashcodeTrait, DatasetFieldEnum, DatasetFieldEnumComposite, EnumTrait, Treatment};

    pub use asmov_common_traitenum;

    #[cfg(feature = "sql")]
    pub use crate::{ToArguments, SqlxArgumentsExtended};
}

pub mod websocket;

#[cfg(feature = "sql")]
pub mod sql;

pub mod driver {
    #[cfg(feature = "postgres")]
    pub mod postgres;

    #[cfg(feature = "sqlite")]
    pub mod sqlite;

    pub mod websocket;
}

#[cfg(feature = "postgres")]
pub use crate::driver::postgres::{self, PostgresDataset};

#[cfg(feature = "sqlite")]
pub use crate::driver::sqlite::{self, SqliteDataset, SqliteDatasetConfig};

pub use crate::driver::websocket::{WebsocketDataset, WebsocketDatasetConfig};
pub use crate::websocket::{WebsocketClientRequest, WebsocketServerResponse, WebsocketErrorResponse, WebsocketPutRequest, WebsocketDeleteRequest, WebsocketGetResponse, WebsocketPutResponse, WebsocketDeleteResponse};
