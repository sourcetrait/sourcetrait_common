// Asmov Common Dataset: Library for application data modeling between clients and servers
// Copyright (C) 2024 Asmov LLC
// SPDX-License-Identifier: AGPL-3.0-or-later

//! This module is used to design boilerplate macros for implementation.
//! Do NOT alter without also updating the macros.
//! All impl code should be written in a way that allows for it to be copied and pasted into the macros:
//!  - use root-level namespacing
//!  - use the same macro variable names without the '$'. eg:
//!    - $ImplModel, $ImplMemoryDataset, etc.
//!

#[cfg(test)]
mod tests {
    use asmov_common_dataset::{self as dataset, model::meta::MetaField};
    use asmov_common_dataset_enum::{DatasetFieldEnum, EnumTrait, Treatment};
    use asmov_common_dataset_enum_derive as fieldenum_derive;
    use tokio;

    #[derive(fieldenum_derive::DatasetFieldEnum)]
    pub enum ImplModelField {
        Txt,
        Num,
        Toggle,
        Timestamp,
    }

    #[derive(Debug, PartialEq, Eq, Clone, Hash, derive_builder::Builder)]
    struct ImplModel {
        meta: dataset::Meta,
        txt: String,
        num: u64,
        toggle: bool,
        timestamp: dataset::Timestamp,
    }

    // for testing assertions
    /*impl PartialEq<<'_, ImplModel>> for ImplModel {
        fn eq(&self, other: &Cow<'_, ImplModel>) -> bool {
            match other {
                Cow::Borrowed(b) => &self == b,
                Cow::Owned(o) => self == o,
            }
        }
    }*/

    // for testing assertions
    /*impl PartialEq<ImplModel> for Cow<'_, ImplModel> {
        fn eq(&self, other: &ImplModel) -> bool {
            match self {
                Cow::Borrowed(b) => b == &other,
                Cow::Owned(o) => o == other,
            }
        }
    }*/

    //MACRO: imprint_meta_for_model!($ImplModel, $schema_name, $schema_name_plural)
    impl ::asmov_common_dataset::MetaModel for ImplModel {
        type FieldEnum = ImplModelField;

        const SCHEMA_NAME: &'static str = "impl_model"; //MACRO: replace ".." with $schema_name
        const SCHEMA_NAME_PLURAL: &'static str = "impl_models"; //MACRO: replace ".." with "$schema_name_plural"

        fn meta(&self) -> &::asmov_common_dataset::Meta {
            &self.meta
        }
    }

    impl ::asmov_common_dataset::MetaModelMut for ImplModel {
        fn meta_mut(&mut self) -> &mut ::asmov_common_dataset::Meta {
            &mut self.meta
        }
    }

    #[derive(Debug, Default)]
    struct ImplMemoryDataset {
        impl_model_variable: ::rustc_hash::FxHashMap<::asmov_common_dataset::ID, ::std::sync::Arc<ImplModel>>,
    }

    impl ImplMemoryDataset {
        fn get_model_map_impl_model_variable(&self) -> &::rustc_hash::FxHashMap<::asmov_common_dataset::ID, ::std::sync::Arc<ImplModel>> {
            &self.impl_model_variable
        }

        fn get_model_map_mut_impl_model_variable(&mut self) -> &mut ::rustc_hash::FxHashMap<::asmov_common_dataset::ID, ::std::sync::Arc<ImplModel>> {
            &mut self.impl_model_variable
        }
    }

    // MACRORULE: imprint_memory_dataset_for_model!($ImplModel, $ImplMemoryDataset, $impl_model_variable)
    impl ::asmov_common_dataset::Dataset for ImplMemoryDataset {}
    impl ::asmov_common_dataset::DatasetDirect for ImplMemoryDataset {}
    impl ::asmov_common_dataset::MemoryDataset for ImplMemoryDataset {}

    impl ::asmov_common_dataset::DatasetModel<ImplMemoryDataset> for ImplModel {}

    impl ::asmov_common_dataset::DatasetDirectModel<ImplMemoryDataset> for ImplModel {
        async fn dataset_get(
            dataset: &ImplMemoryDataset,
            id: ::asmov_common_dataset::ID,
        ) -> ::asmov_common_dataset::Result<::std::option::Option<::std::sync::Arc<Self>>>
        {
            Ok(dataset
                .get_model_map_impl_model_variable()
                .get(&id)
                .and_then(|m| Some(m.clone())))
        }

        async fn dataset_put_mut(
            dataset: &mut ImplMemoryDataset,
            model: Self,
        ) -> ::asmov_common_dataset::Result<::std::sync::Arc<Self>>
        {
            let id = <Self as ::asmov_common_dataset::MetaModel>::meta(&model).id();
            dataset.get_model_map_mut_impl_model_variable()
                .insert(id, ::std::sync::Arc::new(model));
            let m = dataset.get_model_map_impl_model_variable()
                .get(&id)
                .expect("Failed to get() model `impl_model_variable` from map in memory dataset after insert()")
                .clone();

            Ok(m)
        }

        async fn dataset_delete_mut(
            dataset: &mut ImplMemoryDataset,
            id: ::asmov_common_dataset::ID,
        ) -> ::asmov_common_dataset::Result<()>
        {
            let _ = dataset.get_model_map_mut_impl_model_variable()
                .remove(&id);
            Ok(())
        }

        async fn dataset_take(
            dataset: &mut ImplMemoryDataset,
            id: ::asmov_common_dataset::ID,
        ) -> ::asmov_common_dataset::Result<::std::option::Option<::std::sync::Arc<Self>>> {
            dataset
                .get_model_map_mut_impl_model_variable()
                .remove(&id)
                .map_or_else(|| Ok(None), |m| Ok(Some(m)))
        }
}

    fn fixture_model(id: dataset::ID) -> ImplModel {
        use asmov_common_dataset::{self as dataset, prelude::*};
        ImplModelBuilder::default()
            .meta(
                MetaBuilder::default()
                    .id(id)
                    .shard_id(dataset::ID::LOCAL_USER)
                    .time_created(dataset::Timestamp::now())
                    .time_modified(dataset::Timestamp::now())
                    .build()
                    .unwrap(),
            )
            .txt("Hello, world!".to_string())
            .num(42)
            .toggle(true)
            .timestamp(dataset::Timestamp::new(2024, 10, 26, 10, 32, 0, 0).unwrap())
            .build()
            .unwrap()
            .rehashed()
    }

    #[tokio::test]
    async fn test_memory() {
        use asmov_common_dataset::{self as dataset, prelude::*};

        let mut memory_dataset = ImplMemoryDataset::default();

        assert!(
            matches!(
                memory_dataset
                    .get::<ImplModel>(dataset::ID::Local(101))
                    .await
                    .unwrap(),
                None
            ),
            "Memory dataset should return Ok(None) for a missing model"
        );

        let model = fixture_model(dataset::ID::Local(101));

        let field_names = ImplModelField::iter()
            .map(|f| f.name())
            .chain(MetaField::iter().map(|f| f.name()).filter(|n| *n != "id"))
            .collect::<Vec<_>>();

        println!("Field names: {:?}", field_names);

        // put() the model into memory and make sure that we can get() it back
        assert!(
            matches!(
                memory_dataset.put_mut(model.clone()).await.unwrap(),
                model),
            "Memory dataset should return with the expected ID after writing model"
        );
        assert!(
            matches!(memory_dataset.get::<ImplModel>(dataset::ID::Local(101)).await.unwrap(), model),
            "Memory dataset should return an exact copy of the model inserted"
        );

        // clone e model from memory, alter it, put() it back, then get() to compare
        let mut model_clone = (*memory_dataset
            .take::<ImplModel>(dataset::ID::Local(101))
            .await
            .unwrap()
            .unwrap()).clone();
        model_clone.txt = "Hello, universe!".to_string();
        assert!(
            matches!(memory_dataset.put_mut(model_clone.clone()).await.unwrap(), model_clone),
            "Memory dataset should return the same ID after modifying it"
        );
        let model_updated = memory_dataset
            .get::<ImplModel>(dataset::ID::Local(101))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(model_updated.txt, "Hello, universe!".to_string(),
            "Memory dataset should return the updated model after taking, altering, and putting it back");

        // delete() the model from memory and make sure that we can't get() it afterwards
        assert!(
            matches!(
                memory_dataset
                    .delete_mut::<ImplModel>(dataset::ID::Local(101))
                    .await,
                Ok(())
            ),
            "Memory dataset should deleting the model and return Ok"
        );
        assert!(
            matches!(
                memory_dataset
                    .get::<ImplModel>(dataset::ID::Local(101))
                    .await,
                Ok(None)
            ),
            "Memory dataset should return Ok(None) for a missing model after deletion"
        );
    }

    #[cfg(feature = "sqlite")]
    mod sqlite {
        use std::sync::Arc;

        use super::*;
        use asmov_common_dataset::dataset::strategy::DatasetStrategy;
        use sqlx::{self, sqlite, Executor, Row};
        use tokio;

        impl sqlx::FromRow<'_, sqlite::SqliteRow> for ImplModel {
            fn from_row(row: &sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
                Ok(Self {
                    meta: dataset::Meta::from_row(row)?,
                    txt: row.try_get("txt")?,
                    num: row.try_get("num")?,
                    toggle: row.try_get("num")?,
                    timestamp: row.try_get("timestamp")?,
                })
            }
        }

        use dataset::SqlxArgumentsExtended;

        impl ::asmov_common_dataset::ToArguments<::sqlx::sqlite::Sqlite> for ImplModel {
            fn to_arguments<'m: 'q, 'q>(
                &'m self,
                args: <::sqlx::sqlite::Sqlite as ::sqlx::Database>::Arguments<'q>,
                context: ::asmov_common_dataset::ModelingContext,
            ) -> ::sqlx::Result<<::sqlx::sqlite::Sqlite as ::sqlx::Database>::Arguments<'q>>
            {
                Ok(args
                    .meta_args(self, context)?
                    .arg(&self.txt)?
                    .arg(self.num as i64)?
                    .arg(self.toggle)?
                    .arg(self.timestamp)?)
            }
        }

        impl ::asmov_common_dataset::DatasetModel<::asmov_common_dataset::SqliteDataset> for ImplModel {}

        impl ::asmov_common_dataset::DatasetIndirectModel<::asmov_common_dataset::SqliteDataset> for ImplModel {
            async fn dataset_get_owned(
                dataset: &::asmov_common_dataset::SqliteDataset,
                id: asmov_common_dataset::ID,
            ) -> ::asmov_common_dataset::Result<std::option::Option<Self>>
            {
                ::asmov_common_dataset::SqliteDataset::standard_get(dataset, id).await
            }

            async fn dataset_put(
                dataset: &::asmov_common_dataset::SqliteDataset,
                model: Self,
            ) -> ::asmov_common_dataset::Result<::asmov_common_dataset::ID>
            {
                ::asmov_common_dataset::SqliteDataset::standard_insert_or_update(dataset, &model).await
            }

            async fn dataset_delete(
                dataset: &::asmov_common_dataset::SqliteDataset,
                id: ::asmov_common_dataset::ID,
            ) -> ::asmov_common_dataset::Result<()>
            {
                ::asmov_common_dataset::SqliteDataset::standard_delete::<Self>(dataset, id).await
            }
        }

        const SQLITE_SCHEMA: &'static str = r#"
            CREATE TABLE impl_model (
                local_id INTEGER PRIMARY KEY AUTOINCREMENT,
                id INTEGER,
                shard_local_id INTEGER,
                shard_id INTEGER,
                time_created TEXT,
                time_modified TEXT,
                hashcode INTEGER,
                txt TEXT,
                num INTEGER,
                toggle INTEGER,
                timestamp TEXT
            );

            CREATE INDEX idx_id ON impl_model (id);
            CREATE INDEX idx_shard_id ON impl_model (shard_id);
        "#;

        #[tokio::test]
        async fn test_sqlite() {
            use asmov_common_dataset::{self as dataset, prelude::*};

            let mut sqlite_dataset = dataset::SqliteDataset::new(dataset::SqliteDatasetConfig{
                url: String::from(":memory:")
            });

            sqlite_dataset.connect().await.unwrap();
            sqlite_dataset.connection().unwrap()
                .execute(SQLITE_SCHEMA).await.unwrap();

            assert!(
                matches!(
                    sqlite_dataset
                        .get_owned::<ImplModel>(dataset::ID::Local(101))
                        .await
                        .unwrap(),
                    None
                ),
                "SQLite dataset should return Ok(None) for a missing model"
            );

            let mut model = fixture_model(dataset::ID::LOCAL_NONE);

            // put() the model into memory and make sure that we can get() it back
            assert!(
                matches!(
                    sqlite_dataset.put(model.clone()).await,
                    Ok(dataset::ID::Local(1))
                ),
                "SQLite dataset should return with the expected ID after writing model"
            );

            model.set_local_id(1);

            let model = fixture_model(dataset::ID::Local(1));

            let model_read = sqlite_dataset
                .get_owned::<ImplModel>(dataset::ID::Local(1))
                .await
                .unwrap()
                .unwrap();
            assert_eq!(
                model_read, model,
                "SQLite dataset should return an exact copy of the model inserted"
            );

            // delete() the model from memory and make sure that we can't get() it afterwards
            assert!(
                matches!(
                    sqlite_dataset
                        .delete::<ImplModel>(dataset::ID::Local(1))
                        .await
                        .unwrap(),
                    ()
                ),
                "SQLite dataset should deleting the model and return Ok"
            );
            assert!(
                matches!(
                    sqlite_dataset
                        .get_owned::<ImplModel>(dataset::ID::Local(1))
                        .await
                        .unwrap(),
                    None
                ),
                "SQLite dataset should return Ok(None) for a missing model after deletion"
            );
        }

        struct Client {
            pub strategic_dataset: dataset::StrategicDataset<ImplMemoryDataset>
        }

        impl Client {
            pub async fn new() -> Self {
                use asmov_common_dataset::{self as dataset, prelude::*};

                let mut sqlite_dataset = dataset::SqliteDataset::new(dataset::SqliteDatasetConfig{
                    url: String::from(":memory:")
                });

                sqlite_dataset.connect().await.unwrap();
                sqlite_dataset.connection().unwrap()
                    .execute(SQLITE_SCHEMA).await.unwrap();

                let strategic_dataset = StrategicDataset::new(
                    Strategy::StandardApp(StandardAppStrategy::new(Some(sqlite_dataset)))
                );

                Client {
                    strategic_dataset,
                }
            }
        }

        #[tokio::test]
        async fn test_client_strategy_sqlite_memory() {
            use asmov_common_dataset::{self as dataset, prelude::*};
            let mut client = Client::new().await;

            // use this as the prototype model
            let model = fixture_model(dataset::ID::LOCAL_NONE);

            // put the model strategically, get it back strategically, make sure we can
            let strat = client.strategic_dataset.strategy_mut().standard_app_mut();
            let model_put = strat.put::<ImplModel>(model.clone()).await
                .unwrap();

            let model_get = strat.get::<ImplModel>(dataset::ID::Local(1)).await
                .unwrap().unwrap();

            assert_eq!(model_put.id(), dataset::ID::Local(1), "ID should autoincrement to 1");

            assert_eq!(model, *model_put,
                "Model should be the same between get() and put()");

            assert_eq!(model, *model_get,
                "Model should be the same between get() and put()");

            // make sure we can get it back from SQLite directly
            let model_sql = strat.local_dataset().unwrap()
                .get_owned::<ImplModel>(dataset::ID::Local(1)).await
                .unwrap().unwrap();

            assert_eq!(model, model_sql,
                "Model should be the same for SQLite get()");

            // make sure we can get it back from memory directly
            let model_mem = strat.memory_dataset().unwrap()
                .get::<ImplModel>(dataset::ID::Local(1)).await.unwrap().unwrap();

            assert_eq!(model, *model_mem,
                "Model should be the same for memory get()");

            // delete from memory only, then ensure that it doesn't exist there, but still exists in SQLite
            strat.memory_dataset_mut().unwrap()
                .delete_mut::<ImplModel>(dataset::ID::Local(1)).await.unwrap();

            let model_mem = strat.memory_dataset().unwrap()
                .get::<ImplModel>(dataset::ID::Local(1)).await.unwrap();

            let model_sql = strat.local_dataset().unwrap()
                .get_owned::<ImplModel>(dataset::ID::Local(1)).await.unwrap().unwrap();

            assert!(model_mem.is_none(),
                "Model should be deleted from memory");

            assert_eq!(model, model_sql,
                "Model should still exist in SQLite");

            // now delete directly from SQLite, then check to see that it doesn't exist at all
            strat.local_dataset().unwrap()
                .delete::<ImplModel>(dataset::ID::Local(1)).await.unwrap();

            let model_sql = strat.local_dataset().unwrap()
                .get_owned::<ImplModel>(dataset::ID::Local(1)).await.unwrap();

            let model_get = strat.get::<ImplModel>(dataset::ID::Local(1)).await.unwrap();

            assert!(model_sql.is_none() && model_get.is_none(),
                "Model should be deleted from SQLite and from memory");

            // put directly into sql, check on both datasets, then get() to see that it gets cached
            let local_id = strat.local_dataset().unwrap()
                .put::<ImplModel>(model.clone()).await.unwrap();

            let model_sql = strat.local_dataset().unwrap()
                .get_owned::<ImplModel>(local_id).await.unwrap().unwrap();

            let model_mem = strat.memory_dataset().unwrap()
                .get::<ImplModel>(local_id).await.unwrap();

            assert_eq!(local_id, dataset::ID::Local(2), "ID should autoincrement to 2");

            assert!(model_sql == model && model_mem.is_none(),
                "Model should be in SQLite, but not in memory");

            let model_get = strat.get::<ImplModel>(local_id).await.unwrap().unwrap();

            let model_mem = strat.memory_dataset().unwrap()
                .get::<ImplModel>(local_id).await.unwrap().unwrap();

            assert!(*model_get == model && *model_mem == model,
                "Model should be cached in memory after get() it");

            // delete strategically and ensure it's gone from both
            strat.delete_mut::<ImplModel>(local_id).await.unwrap();

            let model_sql = strat.local_dataset().unwrap()
                .get_owned::<ImplModel>(local_id).await.unwrap();

            let model_mem = strat.memory_dataset().unwrap()
                .get::<ImplModel>(local_id).await.unwrap();

            assert!(model_sql.is_none() && model_mem.is_none(),
                "Model should be deleted from SQLite and from memory");
        }
    }
}
