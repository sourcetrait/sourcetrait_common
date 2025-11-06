use crate::*;

pub(crate) fn guess_schema(
    guessed: &Guessed,
    line_num: usize,
    is_first: bool,
    cfg: &SchemaConfig,
    mut guesses: JsonlGuessTree,
    obj: Data<'_>,
) -> JsonSqlResult<JsonlGuessTree> {
    let source_keys = guesses.keys(guessed.source_id);
    let DBG_KEYS = source_keys.join("::");
    dbg!(&guessed);
    dbg!(&DBG_KEYS);
    let table_or_cfg = cfg.find_table_or_default(&source_keys);
    let table_cfg = cfg.find_table(&source_keys);
    // Some if needed, Noneif not
    let primary_key_needed = table_cfg.and_then(|cfg| cfg.key.as_deref());
    let mut primary_key_found = false; 
    
    for (source_column, o) in obj.into_iter() {
        let is_primary_key = primary_key_needed.is_some_and(|key_name| key_name == &source_column);
        if is_primary_key {
            primary_key_found = true;
        }
        
        let mut optional = false;
        let data_guess = match &o {
            Value::Bool(_) => Some(JsonDataKind::Bool),
            Value::String(_) => Some(JsonDataKind::String),
            Value::Number(n) if n.is_u64() => Some(JsonDataKind::Number(JsonNumberKind::Integer(JsonIntegerKind::Unsigned))),
            Value::Number(n) if n.is_i64() => Some(JsonDataKind::Number(JsonNumberKind::Integer(JsonIntegerKind::Signed))),
            Value::Number(_) => Some(JsonDataKind::Number(JsonNumberKind::Float)),
            Value::Object(_) => Some(JsonDataKind::Object),
            Value::Array(v) => {
                let mut items_kind = None;
                
                for v in v {
                    let it_kind = match v {
                        Value::Bool(_) => Some(JsonDataKind::PrimitiveList(JsonPrimitiveKind::Bool)),
                        Value::String(_) => Some(JsonDataKind::PrimitiveList(JsonPrimitiveKind::String)),
                        Value::Number(n) if n.is_u64() => Some(JsonDataKind::PrimitiveList(JsonPrimitiveKind::Number(
                            JsonNumberKind::Integer(JsonIntegerKind::Unsigned)
                        ))),
                        Value::Number(n) if n.is_i64() => Some(JsonDataKind::PrimitiveList(JsonPrimitiveKind::Number(
                            JsonNumberKind::Integer(JsonIntegerKind::Signed)
                        ))),
                        Value::Number(_) => Some(JsonDataKind::PrimitiveList(JsonPrimitiveKind::Number(JsonNumberKind::Float))),
                        Value::Object(_) => Some(JsonDataKind::ObjectList),
                        Value::Array(_) => todo!("nested arrays"),
                        Value::Null => {
                            optional = true;
                            None
                        },
                    };
                    
                    items_kind = data_guess_matrix(items_kind, it_kind)?;
                }
                
                items_kind
            },
            Value::Null => {
                optional = true;
                None
            },
        };
        
        let (column_source_id, _is_column_new) = guesses.reserve_child_key(guessed.source_id, &source_column, JsonlGuessNode::default)
            .into_tuple();
        
        let is_column_new = guesses.get(column_source_id).data().is_none_or(|d| d.column.is_none());
        
        if is_column_new && !is_first {
            optional = true;
        }
        
        let column_guess = JsonlColumnGuess {
            source_id: column_source_id,
            data_guess,
            is_primary_key,
            optional,
        };
        
        match column_guess.data_guess {
            Some(JsonDataKind::Object) => {
                let child_obj = match o {
                    Value::Object(v) => v,
                    _ => unreachable!("object expected"),
                };
                
                guesses = guess_schema(
                    &Guessed {
                        kind: JsonSchemaKind::Object,
                        source_id: column_source_id,
                        parent_source_id: Some(guessed.source_id),
                    },
                    line_num,
                    is_column_new,
                    cfg,
                    guesses,
                    Data::Map(child_obj),
                )?;
                
                /*let primary_key = guess_primary_key(cfg, &guesses, column_source_id)
                    .map_err(|_e| todo!("errors"))?;
                    .map_err(|num_found| JsonSqlError::UndeterminedChildSchema {
                        file: source_file.into(),
                        name: child_schema_name.clone(),
                        why: UndeterminedSchemaErr::PrimaryKey { num_found },
                    })?;
                
                let table_or_cfg = cfg.table_or_default(Path::new(&child_schema_name));
                
    pub(crate) kind: JsonSchemaKind,
    pub(crate) source_id: StreeId,
    pub(crate) primary_key: PrimaryKey,
    pub(crate) parent_id_column: Option<String>,
    pub(crate) list_id_column: Option<String>,
                
                let child_schema_guess = JsonlSchemaGuess {
                    kind: JsonSchemaKind::Object,
                    name: child_schema_name,
                    parent_name: Some(schema_name.to_string()),
                    primary_key,
                    parent_id_column: Some(table_or_cfg.db_parent_id.clone()),
                    list_id_column: None,
                    columns: child_schema_column_guesses,
                    child_schemas: sub_schema_guesses,
                };
                
                child_schemas.insert(guess.source_name.clone(), child_schema_guess);*/ 
            },
            Some(JsonDataKind::ObjectList) => {
                let child_array = match o {
                    Value::Array(v) => v,
                    _ => unreachable!("array expected"),
                };
                
                //let child_schema_name = format!("{schema_name}_{column_name}");
                //let child_table_cfg = cfg.table_or_default(&child_schema_name.as_ref());
                //let mut child_schema_column_guesses = HashMap::new();
                //let mut sub_schema_guesses = HashMap::new();
                for child_item in child_array {
                    //let mut child_new: Option<serde_json::Value> = None;
                    let child_item = match child_item {
                        Value::Object(v) => v, 
                        _ => unreachable!("object expected"),
                            //child_new = Some(serde_json::json!({ "_value": child_item }));
                            //child_new.as_ref().expect("exists").as_object().expect("object")
                    };
                    
                    guesses = guess_schema(
                        &Guessed {
                            kind: JsonSchemaKind::ObjectList,
                            source_id: column_source_id,
                            parent_source_id: Some(guessed.source_id),
                        },
                        line_num,
                        is_column_new,
                        cfg,
                        guesses,
                        Data::Map(child_item),
                    )?;
                    
                    /*(child_schema_column_guesses, sub_schema_guesses) = guess_object(
                        source_file,
                        &child_schema_name,
                        line_num,
                        is_new,
                        cfg,
                        child_table_cfg,
                        child_schema_column_guesses,
                        sub_schema_guesses,
                        child_item
                    )?;*/
                }
                
                /*let primary_key = guess_primary_key(cfg, cfg.table(Path::new(&child_schema_name)), &child_schema_column_guesses)
                    .map_err(|num_found| JsonSqlError::UndeterminedChildSchema {
                        file: source_file.into(),
                        name: child_schema_name.clone(),
                        why: UndeterminedSchemaErr::PrimaryKey { num_found },
                    })?;
                    
                let table_or_cfg = cfg.table_or_default(Path::new(&child_schema_name));
                
                let child_schema_guess = JsonlSchemaGuess {
                    kind: JsonSchemaKind::ObjectList,
                    name: child_schema_name,
                    parent_name: Some(schema_name.to_string()),
                    primary_key,
                    parent_id_column: Some(table_or_cfg.db_parent_id.clone()),
                    list_id_column: Some(table_or_cfg.db_list_id.clone()),
                    columns: child_schema_column_guesses,
                    child_schemas: sub_schema_guesses,
                };
                
                child_schemas.insert(guess.source_name.clone(), child_schema_guess);*/
            },
            Some(JsonDataKind::PrimitiveList(_item_kind)) => {
                let child_array = match o {
                    Value::Array(v) => v,
                    _ => unreachable!("array expected"),
                };
                
                for child_item in child_array {
                    guesses = guess_schema(
                        &Guessed {
                            kind: JsonSchemaKind::PrimitiveList,
                            source_id: column_source_id,
                            parent_source_id: Some(guessed.source_id),
                        },
                        line_num,
                        is_column_new,
                        cfg,
                        guesses,
                        Data::Value(Cow::Borrowed("_value"), child_item), // empty object, as we only care about the item type
                    )?;
                }
            },
            _ => {},
        }
        
        if is_column_new {
            guesses.get_mut(column_source_id)
                .data_mut().get_or_insert(&mut JsonlGuessNode::default())
                .column = Some(column_guess);
        } else {
            let keeeys = guesses.keys(column_source_id).join("::");
            let existing = guesses.get_mut(column_source_id).
                data_mut().expect("exists")
                .column.as_mut().expect("exists");
            
            if existing != &column_guess {
                if existing.optional == false && column_guess.optional == true {
                    existing.optional = false;
                }
                if existing.data_guess != data_guess {
                    let next_guess = data_guess_matrix(existing.data_guess, data_guess)
                        .map_err(|_| {
        dbg!(column_source_id);
        dbg!(&keeeys);
                            dbg!(&DBG_KEYS);
                            dbg!(&existing, data_guess);
                            todo!("errors");
                        }/*JsonSqlError::IncompatibleJsonLinesDataGuess {
                            file: source_file.into(),
                            line_num,
                            existing: existing.data_guess,
                            next: data_guess,
                        }*/)?;
                    
                    if existing.data_guess != next_guess {
                        existing.data_guess = next_guess;
                    }
                }
            }
        }
    }
    
    if primary_key_needed.is_some() && !primary_key_found {
        dbg!(&guessed); dbg!(&guesses.keys(guessed.source_id));
        todo!("error primarykey todo");
    }
    
    guesses.get_mut(guessed.source_id)
        .data_mut().expect("exists")
        .schema.get_or_insert_with(|| JsonlSchemaGuess {
            kind: guessed.kind,
            source_id: guessed.source_id,
            primary_key: primary_key_for(table_or_cfg, primary_key_needed),
            parent_id_column: guessed.parent_source_id.map(|_| table_or_cfg.db_parent_id.clone()),
            list_id_column: guessed.parent_source_id.map(|_| table_or_cfg.db_list_id.clone()),
        });
    
    Ok(guesses)
}

fn data_guess_matrix(existing: Option<JsonDataKind>, current: Option<JsonDataKind>) -> JsonSqlResult<Option<JsonDataKind>> {
    fn num_matrix(existing: JsonNumberKind, current: JsonNumberKind) -> JsonNumberKind {
        match (existing, current) {
            (JsonNumberKind::Integer(n1), JsonNumberKind::Integer(n2)) => JsonNumberKind::Integer(match (n1, n2) {
                (JsonIntegerKind::Signed, JsonIntegerKind::Signed) => JsonIntegerKind::Signed,
                (JsonIntegerKind::Signed, JsonIntegerKind::Unsigned) => JsonIntegerKind::Signed,
                (JsonIntegerKind::Unsigned, JsonIntegerKind::Signed) => JsonIntegerKind::Signed,
                (JsonIntegerKind::Unsigned, JsonIntegerKind::Unsigned) => JsonIntegerKind::Unsigned,
            }),
            (JsonNumberKind::Integer(_), JsonNumberKind::Float) => JsonNumberKind::Float,
            (JsonNumberKind::Float, JsonNumberKind::Integer(_)) => JsonNumberKind::Float,
            (JsonNumberKind::Float, JsonNumberKind::Float) => JsonNumberKind::Float,
        }
    }
    
    match (existing, current) {
        (None, None) => Ok(None),
        (None, Some(JsonDataKind::Bool)) => Ok(Some(JsonDataKind::Bool)),
        (None, Some(JsonDataKind::String)) => Ok(Some(JsonDataKind::String)),
        (None, Some(JsonDataKind::Number(n))) => Ok(Some(JsonDataKind::Number(n))),
        (None, Some(JsonDataKind::Object)) => Ok(Some(JsonDataKind::Object)),
        (None, Some(JsonDataKind::ObjectList)) => Ok(Some(JsonDataKind::ObjectList)),
        (None, Some(JsonDataKind::PrimitiveList(l))) => Ok(Some(JsonDataKind::PrimitiveList(l))),
        (Some(JsonDataKind::Bool), None) => Ok(Some(JsonDataKind::Bool)),
        (Some(JsonDataKind::Bool), Some(JsonDataKind::Bool)) => Ok(Some(JsonDataKind::Bool)),
        (Some(JsonDataKind::Bool), Some(JsonDataKind::String)) => Ok(Some(JsonDataKind::String)),
        (Some(JsonDataKind::Bool), Some(JsonDataKind::Number(_))) => Ok(Some(JsonDataKind::String)),
        (Some(JsonDataKind::Bool), Some(_)) => Err(JsonSqlError::Guess),
        (Some(JsonDataKind::String), None) => Ok(Some(JsonDataKind::String)),
        (Some(JsonDataKind::String), Some(JsonDataKind::Bool)) => Ok(Some(JsonDataKind::String)),
        (Some(JsonDataKind::String), Some(JsonDataKind::String)) => Ok(Some(JsonDataKind::String)),
        (Some(JsonDataKind::String), Some(JsonDataKind::Number(_))) => Ok(Some(JsonDataKind::String)),
        (Some(JsonDataKind::String), Some(_)) => Err(JsonSqlError::Guess),
        (Some(JsonDataKind::Number(n)), None) => Ok(Some(JsonDataKind::Number(n))),
        (Some(JsonDataKind::Number(_)), Some(JsonDataKind::Bool)) => Ok(Some(JsonDataKind::String)),
        (Some(JsonDataKind::Number(_)), Some(JsonDataKind::String)) => Ok(Some(JsonDataKind::String)),
        (Some(JsonDataKind::Number(n)), Some(JsonDataKind::Number(nn))) => Ok(Some(JsonDataKind::Number(num_matrix(n, nn)))),
        (Some(JsonDataKind::Number(_)), Some(_)) => Err(JsonSqlError::Guess),
        (Some(JsonDataKind::Object), None) => Ok(Some(JsonDataKind::Object)),
        (Some(JsonDataKind::Object), Some(JsonDataKind::Object)) => Ok(Some(JsonDataKind::Object)),
        (Some(JsonDataKind::Object), Some(_)) => Err(JsonSqlError::Guess),
        (Some(JsonDataKind::ObjectList), None) => Ok(Some(JsonDataKind::ObjectList)),
        (Some(JsonDataKind::ObjectList), Some(JsonDataKind::ObjectList)) => Ok(Some(JsonDataKind::ObjectList)),
        (Some(JsonDataKind::ObjectList), Some(_)) => Err(JsonSqlError::Guess),
        (Some(JsonDataKind::PrimitiveList(l)), None) => Ok(Some(JsonDataKind::PrimitiveList(l))),
        (Some(JsonDataKind::PrimitiveList(l)), Some(JsonDataKind::PrimitiveList(ll))) => match (l, ll) {
            (JsonPrimitiveKind::Bool, JsonPrimitiveKind::Bool) => Ok(Some(JsonDataKind::PrimitiveList(JsonPrimitiveKind::Bool))),
            (JsonPrimitiveKind::Bool, JsonPrimitiveKind::String) => Ok(Some(JsonDataKind::PrimitiveList(JsonPrimitiveKind::String))),
            (JsonPrimitiveKind::Bool, JsonPrimitiveKind::Number(_)) => Ok(Some(JsonDataKind::PrimitiveList(JsonPrimitiveKind::String))),
            (JsonPrimitiveKind::String, JsonPrimitiveKind::Bool) => Ok(Some(JsonDataKind::PrimitiveList(JsonPrimitiveKind::String))),
            (JsonPrimitiveKind::String, JsonPrimitiveKind::String) => Ok(Some(JsonDataKind::PrimitiveList(JsonPrimitiveKind::String))),
            (JsonPrimitiveKind::String, JsonPrimitiveKind::Number(_)) => Ok(Some(JsonDataKind::PrimitiveList(JsonPrimitiveKind::String))),
            (JsonPrimitiveKind::Number(_), JsonPrimitiveKind::Bool) => Ok(Some(JsonDataKind::PrimitiveList(JsonPrimitiveKind::String))),
            (JsonPrimitiveKind::Number(_), JsonPrimitiveKind::String) => Ok(Some(JsonDataKind::PrimitiveList(JsonPrimitiveKind::String))),
            (JsonPrimitiveKind::Number(n), JsonPrimitiveKind::Number(nn)) => Ok(Some(JsonDataKind::PrimitiveList(JsonPrimitiveKind::Number(num_matrix(n, nn))))),
        }, 
        (Some(JsonDataKind::PrimitiveList(_)), Some(_)) => Err(JsonSqlError::Guess),
        
    }
}
