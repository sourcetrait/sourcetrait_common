// Asmov Common Dataset: Library for application data modeling between clients and servers
// Copyright (C) 2024 Asmov LLC
// SPDX-License-Identifier: AGPL-3.0-or-later

use sqlx::{self, Arguments};
use crate::*;

pub enum ArgumentFormatType {
    Insert,
    InsertOrUpdate,
    //Select,
    //Delete,
}

pub struct ArgumentFormat<'q, DB: sqlx::Database> {
    pub arguments: <DB as sqlx::Database>::Arguments<'q>,
    pub placement_columns: Option<String>,
    pub placement_values: Option<String>,
    pub placement_set: Option<String>,
}

pub trait ToArguments<DB: sqlx::Database>: crate::MetaModel + Send {
    fn to_arguments<'m:'q,'q>(&'m self, args: <DB as sqlx::Database>::Arguments<'q>, context: ModelingContext) -> sqlx::Result<<DB as sqlx::Database>::Arguments<'q>>;
    fn num_fields() -> usize { Self::FieldEnum::iter().count() + <Meta as MetaModel>::FieldEnum::iter().count() - 1 }
    fn format_arguments<'m:'q,'q>(&'m self, format_type: ArgumentFormatType, context: ModelingContext) -> sqlx::Result<ArgumentFormat<'q, DB>> where <DB as sqlx::Database>::Arguments<'q>: SqlxArgumentsExtended<'q, DB> {
        let mut args;
        let placement_values;
        let placement_set: Option<String>;
        match format_type {
            ArgumentFormatType::Insert => {
                args = <DB as sqlx::Database>::Arguments::init_args(Self::num_fields())?;
                args = self.to_arguments(args, context)?;
                placement_values = Some("?, ".repeat(Self::num_fields()-1) + "?");
                placement_set = None;

            },
            ArgumentFormatType::InsertOrUpdate => {
                args = <DB as sqlx::Database>::Arguments::init_args(Self::num_fields())?;
                args = self.to_arguments(args, context)?;
                args = self.to_arguments(args, context)?;
                placement_values = Some("?, ".repeat(Self::num_fields()-1) + "?");
                placement_set = Some(
                    <Meta as MetaModel>::FieldEnum::iter()
                    .filter_map(|field| match field.treatment() {
                        Treatment::PrimaryMutualID => None,
                        _ => Some(field.contextual_name(context)) })
                    .chain(Self::FieldEnum::iter()
                        .filter_map(|field| match field.treatment() {
                            Treatment::PrimaryMutualID => None,
                            _ => Some(field.contextual_name(context)) }))
                    .collect::<Vec<&str>>()
                    .join(" = ?, ") + " = ?");
            },
        }

        let placement_columns = Some(<Meta as MetaModel>::FieldEnum::iter()
                    .filter_map(|field| match field.treatment() {
                        Treatment::PrimaryMutualID => None,
                        _ => Some(field.contextual_name(context)) })
                    .chain(Self::FieldEnum::iter()
                        .filter_map(|field| match field.treatment() {
                            Treatment::PrimaryMutualID => None,
                            _ => Some(field.contextual_name(context)) }))
                    .collect::<Vec<&str>>()
                    .join(", "));

        Ok(ArgumentFormat {
            arguments: args,
            placement_columns,
            placement_values,
            placement_set })
    }
}

pub trait SqlxArgumentsExtended<'q, DB: sqlx::Database>: sqlx::Arguments<'q> {
    fn init_args(capacity: usize) -> sqlx::Result<Self, sqlx::Error>;
    fn arg<T>(self, value: T) -> sqlx::Result<Self, sqlx::Error> where T: sqlx::Encode<'q, DB> + sqlx::Type<sqlx::Sqlite> + 'q;
    fn add_arg<T>(&mut self, value: T) -> sqlx::Result<(), sqlx::Error> where T: sqlx::Encode<'q, DB> + sqlx::Type<sqlx::Sqlite> + 'q;
    fn meta_args<M: crate::MetaModel>(self, model: &M, context: ModelingContext) -> sqlx::Result<Self, sqlx::Error>;
}

impl<'q> SqlxArgumentsExtended<'q, sqlx::sqlite::Sqlite> for sqlx::sqlite::SqliteArguments<'q> {
    fn init_args(capacity: usize) -> sqlx::Result<Self, sqlx::Error> {
        let mut args = sqlx::sqlite::SqliteArguments::default();
        args.reserve(capacity, capacity*64);
        Ok(args)
    }

    fn arg<T>(mut self, value: T) -> sqlx::Result<Self, sqlx::Error> where T: sqlx::Encode<'q, sqlx::sqlite::Sqlite> + sqlx::Type<sqlx::Sqlite> + 'q {
        self.add(value).map_err(|e| sqlx::Error::Encode(e))?;
        Ok(self)
    }

    fn add_arg<T>(&mut self, value: T) -> sqlx::Result<(), sqlx::Error> where T: sqlx::Encode<'q, sqlx::sqlite::Sqlite> + sqlx::Type<sqlx::Sqlite> + 'q {
        self.add(value).map_err(|e| sqlx::Error::Encode(e))?;
        Ok(())
    }

    fn meta_args<M: crate::MetaModel>(self, model: &M, context: ModelingContext) -> sqlx::Result<Self, sqlx::Error> {
        Ok(self
            .arg(model.shard_id().contextual_sql(context))?
            .arg(model.time_created())?
            .arg(model.time_modified())?
            .arg(model.hashcode() as i64)?)
    }
}
