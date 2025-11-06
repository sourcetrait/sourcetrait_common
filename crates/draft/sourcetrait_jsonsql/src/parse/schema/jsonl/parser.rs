use crate::*;

impl JsonSchema {
    pub(crate) fn from_jsonl_file(dir: &Path, jsonl_file: &Path, cfg: &SchemaConfig) -> JsonSqlResult<Self> {
        let subdir = jsonl_file.parent().unwrap_or(Path::new(""));
        let filestem = jsonl_file.file_stem()
            .ok_or_else(|| JsonSqlError::UnsupportedFile { file: jsonl_file.to_path_buf() })?;
        let source_path = subdir.join(filestem);
        let source = StreeKeys::from_path(&source_path)
            .map_err(|_| JsonSqlError::UnsupportedFile { file: jsonl_file.to_path_buf() })?;
        
        let file = File::open(dir.join(jsonl_file))
            .map_err(|source| JsonSqlError::FileIo { file: jsonl_file.into(), source })?;
        let reader = BufReader::new(file);
        
        let mut guesses = JsonlGuessTree::new();
        let source_id = guesses.set(&source, Some(JsonlGuessNode::default()));
        
        let guessed = Guessed {
            kind: JsonSchemaKind::Object,
            source_id,
            parent_source_id: None,
        };
        
        let mut line_num: usize = 0;
        for line in reader.lines() {
            line_num += 1;
            let line = line
                .map_err(|source| JsonSqlError::FileIo { file: jsonl_file.into(), source })?;
            let value = serde_json::from_str::<serde_json::Value>(&(line))
                .map_err(|source| JsonSqlError::SerdeJsonFile { file: jsonl_file.into(), source })?;
            
            let obj = match value {
                Value::Object(v) => v,
                _ => return Err(JsonSqlError::JsonLinesFileNotObject { file: jsonl_file.into(), line_num }),
            };
            
            
            guesses = guess_schema(
                &guessed,
                line_num,
                line_num == 1,
                cfg,
                guesses,
                Data::Map(obj),
            )?;
        } 
        
        /*
        let primary_key = guess_primary_key(cfg, &guesses, source_id)
            .map_err(|num_found| { JsonSqlError::UndeterminedSchema {
                file: jsonl_file.into(),
                why: UndeterminedSchemaErr::PrimaryKey { num_found }
            }})?;
        
        let schema_guesses = guesses.root().iter_mut()
            .filter_map(|node| match node.data_mut() {
                Some(data) => data.schema.take(),
                None => None,
            })
            .collect::<Vec<_>>();
        
        for schema_guess in schema_guesses {
            let (schema, guesses) = match assume_schema(cfg, guesses, schema_guess) {
                Ok(ok) => ok,
                Err(err) => todo!("errors"),
            };
        }
        
        for (child_schema_name, child_schema_guess) in guess_child_schemas {
            let child_schema: JsonSchema = match Self::try_from_guess(cfg, cfg.table(Path::new(&child_schema_name)), jsonl_file, &schema_name, child_schema_guess) {
                Ok(v) => v,
                Err(e) => return Err(JsonSqlError::UndeterminedJsonLinesChildSchema {
                    file: jsonl_file.into(),
                    column_name: child_schema_name,
                    source: e,
                }),
            };
            
            child_schemas.insert(child_schema_name, child_schema);
        }*/
        
        let schema_guess = guesses.get_mut(source_id)
            .data_mut().expect("exists")
            .schema.take().expect("exists");
        
        let (schema, _) = assume_schema(cfg, guesses, schema_guess)
            .map_err(|_| todo!())?; //todo: errors
        
        Ok(schema)
    }
}
