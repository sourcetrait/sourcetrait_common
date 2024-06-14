// Asmov Common Dataset: Library for application data modeling between clients and servers
// Copyright (C) 2024 Asmov LLC
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::*;
use dataset::Dataset;
use sqlx::{self, sqlite, Executor, Row};
use std::borrow::Cow;

#[allow(unused_imports)]
use send_future::SendFuture as _;

pub struct SqliteDatasetConfig {
    pub url: String
}

pub struct SqliteDataset {
    config: SqliteDatasetConfig,
    modeling_context: ModelingContext, // harcoded to local
    pool: Option<sqlite::SqlitePool>, // connection
}

impl SqliteDataset {
    pub fn new(config: SqliteDatasetConfig) -> Self {
        Self {
            config,
            modeling_context: ModelingContext::Local,
            pool: None,
        }
    }

    pub fn connection(&self) -> Result<&sqlite::SqlitePool> {
        self.pool.as_ref().ok_or(Error::Disconnected)
    }

    /// Performs a SELECT query for a single row.
    /// Expects a valid ID.
    /// Expects an authorative ID if this dataset is authorative (currently impossible for SQLite).
    pub async fn standard_get<'d: 'm, 'm, M>(&'d self, id: ID) -> Result<Option<M>>
    where
        M: DatasetModel<Self> + 'm,
        for<'r> M: sqlx::FromRow<'r, sqlite::SqliteRow> + Unpin + 'r,
    {
        let table = M::SCHEMA_NAME;
        let querystr;

        match id.into_option() {
            OptionID::None => {
                panic!("Cannot SQLite SELECT for an invalid ID");
            }
            OptionID::Some(ID::Authorative(_)) | OptionID::Some(ID::Mutual(_, _)) => {
                querystr = format!("SELECT * FROM {table} WHERE id = ? LIMIT 1");
            }
            OptionID::Some(ID::Local(_)) if !self.modeling_context.is_local() => {
                panic!("Cannot SQLite SELECT for a local ID from a non-local dataset")
            }
            OptionID::Some(ID::Local(_)) => {
                querystr = format!("SELECT * FROM {table} WHERE local_id = ? LIMIT 1");
            }
            OptionID::Reserved => panic!("Cannot SQLite INSERT or UPDATE for a reserved ID"),
        }

        #[cfg(debug_assertions)]
        println!("SQLite: {querystr}");

        let result: sqlx::Result<M> = sqlx::query_as::<_, M>(&querystr)
            .bind(id.contextual_sql(self.modeling_context))
            .fetch_one(self.connection()?)
            .await;

        match result {
            Ok(m) => Ok(Some(m)),
            Err(e) => match e {
                sqlx::Error::RowNotFound => Ok(None),
                _ => Err(Error::Database(e.to_string())),
            },
        }
    }

    /// Performs an UPDATE or INSERT operation depending on whether the model has a valid ID or not, respectively.
    /// Expects the SQLite table to have a column named "local_id" if this dataset is local (currently always true for SQLite).
    /// Returns an ID if INSERTed.
    pub async fn standard_insert_or_update<'d: 'm, 'm: 'q, 'q, M>(
        &'d self,
        model: &'m M,
    ) -> Result<ID>
    where
        M: DatasetModel<Self> + ToArguments<sqlx::sqlite::Sqlite>,
    {
        let table = M::SCHEMA_NAME;
        let id_field = MetaField::ID.contextual_name(self.modeling_context);
        let querystr;
        let mut format;
        match model.id().into_option() {
            OptionID::None => {
                format = model
                    .format_arguments(ArgumentFormatType::Insert, self.modeling_context)
                    .map_err(|e| Error::from(e))?;
                let insert_places = format
                    .placement_values
                    .as_ref()
                    .expect("Expected placement_values for INSERT");
                let insert_columns = format
                    .placement_columns
                    .as_ref()
                    .expect("Expected placement_columns for InsertOrUpdate");
                querystr = format!("INSERT INTO {table} ({insert_columns}) VALUES ({insert_places}) RETURNING {id_field}");
            }
            OptionID::Some(ID::Authorative(_)) | OptionID::Some(ID::Mutual(_, _)) => {
                format = model
                    .format_arguments(ArgumentFormatType::InsertOrUpdate, self.modeling_context)
                    .map_err(|e| Error::from(e))?;
                let insert_places = format
                    .placement_values
                    .as_ref()
                    .expect("Expected placement_values for InsertOrUpdate");
                let set_places = format
                    .placement_set
                    .as_ref()
                    .expect("Expected placement_set for InsertOrUpdate");
                querystr = format!("INSERT INTO {table} VALUES ({insert_places}) ON CONFLICT(id) DO UPDATE SET {set_places} WHERE {id_field} = ? RETURNING {id_field}");
                format
                    .arguments
                    .add_arg(model.id().contextual_sql(self.modeling_context))?;
            }
            OptionID::Some(ID::Local(_)) if !self.modeling_context.is_local() => {
                panic!("Cannot SQLite UPDATE for a local ID from a non-local dataset")
            }
            OptionID::Some(ID::Local(_)) => {
                format = model
                    .format_arguments(ArgumentFormatType::InsertOrUpdate, self.modeling_context)
                    .map_err(|e| Error::from(e))?;
                let insert_columns = format
                    .placement_columns
                    .as_ref()
                    .expect("Expected placement_columns for InsertOrUpdate");
                let insert_places = format
                    .placement_values
                    .as_ref()
                    .expect("Expected placement_values for InsertOrUpdate");
                let set_places = format
                    .placement_set
                    .as_ref()
                    .expect("Expected placement_set for InsertOrUpdate");
                querystr = format!("INSERT INTO {table} ({insert_columns}) VALUES ({insert_places}) ON CONFLICT(local_id) DO UPDATE SET {set_places} WHERE {id_field} = ? RETURNING {id_field}");
            }
            OptionID::Reserved => panic!("Cannot SQLite INSERT or UPDATE for a reserved ID"),
        }

        #[cfg(debug_assertions)]
        println!("SQLite: {querystr}");

        let query = sqlx::query_with(&querystr, format.arguments);
        match query.fetch_one(self.connection()?).await {
            Ok(row) => {
                let id: i64 = row.try_get::<i64, _>("local_id")?;
                Ok(ID::Local(id as u64))
            }
            Err(e) => match e {
                sqlx::Error::RowNotFound => Err(Error::MissingRow(model.id())),
                _ => Err(Error::Database(e.to_string())),
            },
        }
    }

    /// Performs a DELETE query for a single row.
    /// Expects a valid ID.
    /// Expects an authorative ID if this dataset is authorative (currently impossible for SQLite).
    pub async fn standard_delete<M>(&self, id: ID) -> Result<()>
    where
        M: DatasetModel<Self>,
    {
        let table = M::SCHEMA_NAME;
        let id_field = MetaField::ID.contextual_name(self.modeling_context);
        let querystr;

        match id.into_option() {
            OptionID::None => { panic!("Cannot SQLite SELECT for an invalid ID"); },
            OptionID::Some(ID::Authorative(_)) if self.modeling_context.is_local() => panic!("Cannot SQLite DELETE for an authorative ID from a local dataset"),
            OptionID::Some(ID::Local(_)) if self.modeling_context.is_authorative() => panic!("Cannot SQLite SELECT for a local ID from an authorative dataset"),
            _ => querystr = format!("DELETE FROM {table} WHERE {id_field} IN ( SELECT {id_field} FROM {table} WHERE {id_field} = ? LIMIT 1 )"),
        }

        #[cfg(debug_assertions)]
        println!("SQLite: {querystr}");

        let query = sqlx::query(&querystr).bind(id.contextual_sql(self.modeling_context));

        let result = self.connection()?.execute(query).await;
        match result {
            Ok(_) => Ok(()),
            Err(e) => match e {
                sqlx::Error::RowNotFound => Ok(()),
                _ => Err(Error::Database(e.to_string())),
            },
        }
    }
}

impl Dataset for SqliteDataset {}

impl DatasetIndirect for SqliteDataset {
    fn is_connected(&self) -> bool {
        self.pool.is_some()
    }

    async fn connect(&mut self) -> Result<()> {
        if self.is_connected() {
            return Err(Error::Connected);
        }

        let pool = sqlite::SqlitePool::connect(&self.config.url).await?;
        self.pool = Some(pool);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        match self.pool.take() {
            None => Ok(()),
            Some(pool) => {
                pool.close().await;
                Ok(())
            },
        }
    }
}
