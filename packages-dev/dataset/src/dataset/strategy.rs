// Asmov Common Dataset: Library for application data modeling between clients and servers
// Copyright (C) 2024 Asmov LLC
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::*;
use std::{borrow::Cow, future::Future, sync::Arc};

// Rust issue: https://github.com/rust-lang/rust/issues/96865
// Workaround: https://docs.rs/send-future/latest/send_future/trait.SendFuture.html
#[allow(unused_imports)]
use send_future::SendFuture as _;

pub trait DatasetStrategy<MEM>: Dataset
where
    MEM: MemoryDataset,
{
    fn strategic_get<'a, M>(
        &'a self,
        id: ID,
    ) -> impl Future<Output = Result<Option<Cow<'a, M>>>> + Send
    where
        Self: Sized + 'a + Send,
        M: MetaModel + DatasetModel<Self> + 'a + Send;
}

/*pub trait StrategicGet: Dataset {
    fn strategic_get<'a, M>(&'a self, id: ID) -> impl Future<Output = Result<Option<Cow<'a, M>>>> + Send
    where
        Self: Sized + 'a + Send,
        M: MetaModel + DatasetModel<Self> + StrategicDatasetModel<MEM> + 'a + Send;
}*/

pub struct StrategicDataset<MEM: MemoryDataset> {
    strategy: Strategy<MEM>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatasetType {
    None,
    Memory,
    Sqlite,
    Backend,
    Postgres,
}

pub enum Strategy<MEM: MemoryDataset> {
    /// Includes:
    /// 1. Authorative remote DB via server API stream
    /// 2. Persistant local DB via driver (sqlite, etc.)
    /// 3. Memory cache
    StandardApp(StandardAppStrategy<MEM>),
    // Includes:
    // 1. Authorative remote DB via driver (postgres, etc.)
    // 2. Multiple client API streams
    StandardServer(StandardServerStrategy),
}

impl<MEM: MemoryDataset> Strategy<MEM> {
    pub fn standard_app(&self) -> &StandardAppStrategy<MEM> {
        match self {
            Strategy::StandardApp(strategy) => strategy,
            _ => panic!("Invalid strategy type"),
        }
    }

    pub fn standard_app_mut(&mut self) -> &mut StandardAppStrategy<MEM> {
        match self {
            Strategy::StandardApp(strategy) => strategy,
            _ => panic!("Invalid strategy type"),
        }
    }
}

pub struct StandardAppStrategy<MEM: MemoryDataset> {
    pub memory_dataset: Option<MEM>,
    pub local_dataset: Option<SqliteDataset>,
    //pub remote_dataset: Option<>,
}

impl<MEM: MemoryDataset> StandardAppStrategy<MEM> {
    pub fn new(local_dataset: Option<SqliteDataset>) -> Self {
        Self {
            memory_dataset: Some(MEM::default()),
            local_dataset,
        }
    }

    pub fn memory_dataset(&self) -> Option<&MEM> {
            self.memory_dataset.as_ref()
    }

    pub fn memory_dataset_mut(&mut self) -> Option<&mut MEM> {
        self.memory_dataset.as_mut()
    }

    pub fn local_dataset(&self) -> Option<&SqliteDataset> {
        self.local_dataset.as_ref()
    }

    pub fn local_dataset_mut(&mut self) -> Option<&mut SqliteDataset> {
        self.local_dataset.as_mut()
    }
}

pub struct StandardServerStrategy {
    pub remote_dataset: Option<Box<dyn Dataset>>,
}

impl<MEM: MemoryDataset> StrategicDataset<MEM> {
    pub fn new(strategy: Strategy<MEM>) -> Self {
        Self {
            strategy
        }
    }

    pub fn strategy(&self) -> &Strategy<MEM> {
        &self.strategy
    }

    pub fn strategy_mut(&mut self) -> &mut Strategy<MEM> {
        &mut self.strategy
    }
}

impl<MEM: MemoryDataset> Dataset for StrategicDataset<MEM> {}

#[cfg(not(feature = "sql"))]
pub trait StrategicDatasetModel<MEM: MemoryDataset>:
    MetaModel + DatasetModel<StrategicDataset<MEM>> + DatasetModel<MEM>
{}

#[cfg(all(not(feature = "sqlite"), feature = "postgres"))]
pub trait StrategicDatasetModel<MEM: MemoryDataset>:
    MetaModel
    + DatasetModel<StrategicDataset<MEM>>
    + DatasetModel<MEM>
    + DatasetModel<postgres::PostgresDataset>
{
}

pub trait StrategicDatasetModel<MEM: MemoryDataset>:
    MetaModel
    + DatasetModel<StrategicDataset<MEM>>
    + DatasetDirectModel<MEM>
    + DatasetIndirectModel<sqlite::SqliteDataset>
{
}

#[cfg(all(feature = "postgres", feature = "sqlite"))]
pub trait StrategicDatasetModel<MEM: MemoryDataset>:
    MetaModel
    + DatasetModel<StrategicDataset<MEM>>
    + DatasetModel<MEM>
    + DatasetModel<sqlite::SqliteDataset>
    + DatasetModel<postgres::PostgresDataset>
{
}

impl<MEM: MemoryDataset> StandardAppStrategy<MEM> {
    pub async fn get<M>(
        &mut self,
        id: ID,
    ) -> Result<Option<Arc<M>>>
    where
        MEM: MemoryDataset,
        M: DatasetDirectModel<MEM>
            + DatasetIndirectModel<sqlite::SqliteDataset>
            + Send
    {
        let mut model = None;

        if let Some(memory_dataset) = self.memory_dataset() {
            let result = M::dataset_get(memory_dataset, id).await?;
            if let Some(m) = result {
                return Ok(Some(m));
            }
        }

        if let Some(local_dataset) = self.local_dataset() {
            let result = M::dataset_get_owned(local_dataset, id).await?;
            if result.is_some() {
                model = result;
            } else {
                return Ok(None);
            }
        }

        if let Some(model) = model {
            if let Some(memory_dataset) = self.memory_dataset_mut() {
                let m = M::dataset_put_mut(memory_dataset, model).await?;
                return Ok(Some(m));
            } else {
               return Ok(Some(Arc::new(model)))
            }
        }

        Ok(None)
    }

    pub async fn put<M>(
        &mut self,
        model: M,
    ) -> Result<Arc<M>>
    where
        MEM: MemoryDataset,
        M: MetaModelMut
            + DatasetDirectModel<MEM>
            + DatasetIndirectModel<sqlite::SqliteDataset>
            + Send
    {
        let mut m: Option<M> = Some(model);

        if let Some(sqlite_dataset) = self.local_dataset() {
            let id = M::dataset_put(sqlite_dataset, m.take().unwrap()).await?;
            let model_back = M::dataset_get_owned(sqlite_dataset, id).await?
                .ok_or_else(|| Error::MissingRow(id))?;
            m = Some(model_back);
        }

        if let Some(memory_dataset) = self.memory_dataset.as_mut() {
            M::dataset_put_mut(memory_dataset, m.unwrap()).await
        } else {
            Ok(Arc::new(m.expect("Expected model to be present")))
        }
    }

    pub async fn delete_mut<M>(
        &mut self,
        id: ID,
    ) -> Result<()>
    where
        MEM: MemoryDataset,
        M: MetaModelMut
            + DatasetDirectModel<MEM>
            + DatasetIndirectModel<sqlite::SqliteDataset>
            + Send
    {
        if let Some(memory_dataset) = self.memory_dataset.as_mut() {
            M::dataset_delete_mut(memory_dataset, id).await?;
        }

        if let Some(sqlite_dataset) = self.local_dataset() {
            M::dataset_delete(sqlite_dataset, id).await?;
        }

        Ok(())
    }
}
