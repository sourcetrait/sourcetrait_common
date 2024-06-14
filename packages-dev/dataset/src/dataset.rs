// Asmov Common Dataset: Library for application data modeling between clients and servers
// Copyright (C) 2024 Asmov LLC
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # Asmov Common Dataset :: Datasets
//! A [Dataset] represents a collection of data models, with basic read/write/delete operations.
//!
//! There are currently four types of datasets available:
//! - [MemoryDataset] Stores and accesses data in memory, using a HashMap.
//! - [SqliteDataset] Accesses data from a SQLite database.
//! - [PostgresDataset] Accesses data from a PostgreSQL database.
//! - [StrategicDataset] Maintains a collection of other datasets and accesses from the correct one
//! for the situation.

pub mod memory;
pub mod strategy;

use crate::*;
use std::{future::Future, sync::Arc};

// Rust issue: https://github.com/rust-lang/rust/issues/96865
// Workaround: https://docs.rs/send-future/latest/send_future/trait.SendFuture.html
#[allow(unused_imports)]
use send_future::SendFuture as _;

/// Generically represents a collection of data models, with basic read/write/delete operations.
/// Refer to the [dataset module documentation](crate::dataset) for more information.
pub trait Dataset {}

/// For datasets that store data directly in memory.
pub trait DatasetDirect: Dataset {
    /// Retrieves a model from the dataset.
    fn get<M>(&self, id: ID) -> impl Future<Output = Result<Option<Arc<M>>>> + Send
    where
        Self: Sized,
        M: DatasetDirectModel<Self>
    {
        M::dataset_get(self, id)
    }

    /// Inserts or updates a model in the dataset.
    fn put_mut<M>(&mut self, model: M) -> impl Future<Output = Result<Arc<M>>> + Send
    where
        Self: Sized,
        M: DatasetDirectModel<Self>
    {
        M::dataset_put_mut(self, model)
    }

    /// Removes a model from the dataset.
    fn delete_mut<M>(&mut self, id: ID) -> impl Future<Output = Result<()>> + Send
    where
        Self: Sized,
        M: DatasetDirectModel<Self>
    {
        M::dataset_delete_mut(self, id)
    }

    /// After mutation, [MutDataset::update] must be called to process changes properly.
    fn take<M>(
        &mut self,
        id: ID,
    ) -> impl Future<Output = Result<Option<Arc<M>>>> + Send
    where
        Self: Sized,
        M: DatasetDirectModel<Self>
    {
        M::dataset_take(self, id)
    }
}

pub trait DatasetIndirect: Dataset {
    fn is_connected(&self) -> bool;
    fn connect(&mut self) -> impl Future<Output = Result<()>> + Send;
    fn disconnect(&mut self) -> impl Future<Output = Result<()>> + Send;

    fn get_owned<M>(
        &self,
        id: ID,
    ) -> impl Future<Output = Result<Option<M>>> + Send
    where
        Self: Sized,
        M: DatasetIndirectModel<Self>
    {
        M::dataset_get_owned(self, id)
    }

    /// Inserts or updates a model in the dataset.
    /// The remote database may mutate the model, so we own it rather than pass it by reference.
    fn put<M>(&self, model: M) -> impl Future<Output = Result<ID>> + Send
    where
        Self: Sized,
        M: DatasetIndirectModel<Self>
    {
        M::dataset_put(self, model)
    }

    /// Removes a model from the dataset.
    fn delete<M>(&self, id: ID) -> impl Future<Output = Result<()>> + Send
    where
        Self: Sized,
        M: DatasetIndirectModel<Self>
    {
        M::dataset_delete(self, id)
    }
}

/// Each data model must implement this trait to be stored in a given dataset. Allows for basic read/write/delete operations.
pub trait DatasetModel<DB: Dataset>: MetaModel + Send {}

/// Each data model must implement this trait to be stored in a given [direct access dataset](DatasetDirect).
pub trait DatasetDirectModel<DB: DatasetDirect>: DatasetModel<DB> {
    fn dataset_get(
        dataset: &DB,
        id: ID,
    ) -> impl Future<Output = Result<Option<Arc<Self>>>> + Send {
        dataset.get(id)
    }

    fn dataset_put_mut(
        dataset: &mut DB,
        model: Self,
    ) -> impl Future<Output = Result<Arc<Self>>> + Send {
        dataset.put_mut(model)
    }

    fn dataset_delete_mut(
        dataset: &mut DB,
        id: ID,
    ) -> impl Future<Output = Result<()>> + Send;

    fn dataset_take(
        dataset: &mut DB,
        id: ID,
    ) -> impl Future<Output = Result<Option<Arc<Self>>>> + Send;
}

pub trait DatasetIndirectModel<DB: DatasetIndirect>: DatasetModel<DB> {
    fn dataset_get_owned(
        dataset: &DB,
        id: ID,
    ) -> impl Future<Output = Result<Option<Self>>> + Send;

    fn dataset_put(
        dataset: &DB,
        model: Self, // database may mutate remotely
    ) -> impl Future<Output = Result<ID>> + Send;

    fn dataset_delete(
        dataset: &DB,
        id: ID,
    ) -> impl Future<Output = Result<()>> + Send;
}
