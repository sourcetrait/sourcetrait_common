use crate::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SqliteSchema {
    json_schemas: Vec<JsonSchema>
}

impl SqliteSchema {
    pub fn new(schemas: Vec<JsonSchema>) -> Self {
        Self {
            json_schemas: schemas
        }
    }
    
    pub fn write<R: io::Write>(&self, mut writer: io::BufWriter<R>) -> JsonSqlResult<()> {
        let top_sql = indoc::formatdoc!{"
            #PRAGMA foreign_keys = ON;
            
        "};
        
        writer.write(top_sql.as_bytes())
            .map_err(|e| JsonSqlError::FileIo { file: PathBuf::from("<output>"), source: e })?;
        
        for json_schema in &self.json_schemas {
            self.write_schema(json_schema, &mut writer)?;
            
            for child_schema in json_schema.child_schemas.values() {
                self.write_schema(child_schema, &mut writer)?;
            }
        }
        
        Ok(())
    }
    
    pub fn write_schema<R: io::Write>(&self, schema: &JsonSchema, writer: &mut io::BufWriter<R>) -> JsonSqlResult<()> {
        let table_name = &schema.name;
        let db_id = schema.primary_key.db();
        
        let mut columns = schema.columns.values()
            .collect::<Vec<&JsonColumn>>();
        columns.sort_by(|a, b| a.name.cmp(&b.name));
        
        let mut columns_sql = columns.into_iter() 
            .map(|col| col.to_sqlite(schema))
            .filter_map(|v| v)
            .collect::<Vec<String>>();
            
        let mut needs_auto_increment = false;
        if let Some(db_parent_id) = schema.parent_id_column.as_ref() {
            let parent_schema = schema.parent_name.as_ref().expect("parent");
            let parent_schema_id = schema.primary_key.db();
            let sql = format!("{db_parent_id} INTEGER REFERENCES {parent_schema}({parent_schema_id}) ON DELETE CASCADE");
            columns_sql.insert(0, sql);
            
            if let Some(db_list_id) = &schema.list_id_column {
                let sql = format!("{db_list_id} INTEGER");
                columns_sql.insert(1, sql);
                needs_auto_increment = true;
            }
        };
        
        let columns_sql = columns_sql.join(",\n    ");
        
        let list_autoincrement_sql = match needs_auto_increment {
            false => "",
            true => {
                let db_list_id = schema.list_id_column.as_ref().expect("list id column");
                &indoc::formatdoc! {"
                    
                    CREATE TRIGGER {table_name}_increment_list_id_trigger 
                        AFTER INSERT ON {table_name} 
                        WHEN new.{db_list_id} IS NULL
                        BEGIN
                            UPDATE {table_name} 
                            SET {db_list_id} = (SELECT IFNULL(MAX({db_list_id}), 0) + 1 FROM {table_name})
                            WHERE {db_id} = new.{db_id};
                        END;
                "}
            },
        };
        
        let sql = indoc::formatdoc! {"
            CREATE TABLE {table_name} (
                {db_id} INTEGER PRIMARY KEY AUTOINCREMENT,
                {columns_sql}
            );
            {list_autoincrement_sql}
        "};
        
        writer.write(sql.as_bytes())
            .map_err(|e| JsonSqlError::FileIo { file: PathBuf::from("<output>"), source: e })?;
        
        Ok(())
    }
    
    pub fn write_file<P: AsRef<Path>>(&self, output_file: P) -> JsonSqlResult<()> {
        let file = fs::File::create(&output_file)
            .map_err(|e| JsonSqlError::FileIo { file: output_file.as_ref().to_path_buf(), source: e })?;
        let writer = io::BufWriter::new(file);
        
        self.write(writer)?;
        
        let json_output_file = output_file.as_ref().with_extension(EXT_SCHEMA_JSON);
        let json_file = fs::File::create(&json_output_file)
            .map_err(|e| JsonSqlError::FileIo { file: json_output_file.to_path_buf(), source: e })?;
        let writer = io::BufWriter::new(json_file);
        serde_json::to_writer(writer, self)
            .map_err(|e| JsonSqlError::FileIo { file: json_output_file.to_path_buf(), source: io::Error::new(io::ErrorKind::Other, e) })?;
        
        Ok(())
    }
}

impl JsonColumn {
    fn to_sqlite(&self, schema: &JsonSchema) -> Option<String> {
        let data_type = match self.data_kind {
            JsonDataKind::Bool => "INTEGER",
            JsonDataKind::String => "TEXT",
            JsonDataKind::Number(JsonNumberKind::Integer(_)) => "INTEGER",
            JsonDataKind::Number(JsonNumberKind::Float) => "REAL",
            JsonDataKind::Object => return None,
            JsonDataKind::ObjectList => return None,
            JsonDataKind::PrimitiveList(_) => return None,
        };
        let column_name = &self.name;
        let nullable = match self.optional {
            true => "",
            false => " NOT NULL",
        };
        let unique = match schema.primary_key.source().is_some_and(|column| column == self.name) {
            true => " UNIQUE",
            false => "",
        };
        
        Some(format!("{column_name} {data_type}{nullable}{unique}"))
    }
}