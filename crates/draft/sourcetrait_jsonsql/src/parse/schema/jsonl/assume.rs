use crate::*;

pub(crate) fn primary_key_for(table_or_cfg: &SchemaTableConfig, key_needed: Option<&str>) -> PrimaryKey {
    match key_needed {
        None => PrimaryKey::Db(table_or_cfg.db_id.clone()),
        Some(key_name) => PrimaryKey::Source {
            db: table_or_cfg.db_id.clone(),
            source: key_name.to_string(),
        }
    }
}

pub(crate) fn assume_schema(cfg: &SchemaConfig, mut guesses: JsonlGuessTree, guess: JsonlSchemaGuess) -> StreeResult<JsonSchema, JsonlGuessNode, ChildSchemaError> {
    let column_guesses = guesses.get_mut(guess.source_id)
        .iter_children_mut()
        .filter_map(|node| match node.data_mut() {
            Some(data) => data.column.take().map(|c| (node.id(), c)),
            None => None,
        })
        .collect::<Vec<_>>();
    
    let mut columns: HashMap<String, JsonColumn> = HashMap::new();
    for (node_id, column_guess) in column_guesses {
        let column;
        (column, guesses) = match assume_column(guesses, column_guess) {
            Ok(ok) => ok,
            Err((_, guesses)) => return Err((ChildSchemaError::UndeterminedColumn { source_path: guesses.to_path_string(node_id) }, guesses)),
        };
        
        let name = guesses.get(node_id).key().to_snake_case();
        columns.insert(name, column);
    }
    
    let child_schema_guesses = guesses.get_mut(guess.source_id)
        .iter_children_mut()
        .filter_map(|node| match node.data_mut() { 
            Some(data) => data.schema.take().map(|c| (node.id(), c)),
            None => None,
        })
        .collect::<Vec<_>>();
    
    let mut child_schemas: HashMap<String, JsonSchema> = HashMap::new();
    for (child_schema_source, child_schema_guess) in child_schema_guesses {
        let child_schema;
        (child_schema, guesses) = match assume_schema(cfg, guesses, child_schema_guess) {
            Ok(ok) => ok,
            Err((_, guesses)) => return Err((ChildSchemaError::UndeterminedSubSchema { source_path: guesses.to_path_string(child_schema_source) }, guesses)),
        };
        
        let name = guesses.keys(child_schema_source).to_snake_case();
        child_schemas.insert(name, child_schema);
    }
    
    let table_or_cfg = cfg.find_table_or_default(&guesses.keys(guess.source_id));
    let parent_id_column = Some(table_or_cfg.db_parent_id.clone());
    let list_id_column = match guess.kind {
        JsonSchemaKind::Object => None,
        JsonSchemaKind::ObjectList => Some(table_or_cfg.db_list_id.clone()),
        JsonSchemaKind::PrimitiveList => Some(table_or_cfg.db_list_id.clone()),
    };
    
    let name = guesses.keys(guess.source_id).to_snake_case();
    let parent_name = guesses.parent_keys(guess.source_id).to_snake_case();
    let source = guesses.to_path_string(guess.source_id);
    let primary_key = guess.primary_key;
    
    Ok((
        JsonSchema {
            kind: guess.kind,
            source, 
            parent_name: Some(parent_name),
            name: name,
            primary_key,
            parent_id_column,
            list_id_column,
            columns,
            child_schemas,
        },
        guesses,
    ))
}

fn assume_column(guesses: JsonlGuessTree, guess: JsonlColumnGuess) -> StreeResult<JsonColumn, JsonlGuessNode, JsonlColumnGuess> {
    let data_kind = match guess.data_guess {
        Some(k) => k,
        None => return Err((guess, guesses)),
    };
    
    let source = guesses.to_path_string(guess.source_id);
    let name = guesses.keys(guess.source_id).to_snake_case();
    
    Ok((
        JsonColumn {
            source,
            name,
            data_kind,
            optional: guess.optional,
        },
        guesses
    ))
}
