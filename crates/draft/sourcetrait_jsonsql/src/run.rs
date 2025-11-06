use crate::*;

pub fn run() -> ExitCode {
    let cli = Cli::parse();
    
    match self::run_with(cli) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => on_error(e),
    }
}

pub fn run_with(cli: Cli) -> JsonSqlResult<()> {
    let run = RunState::new(cli);
    
    match &run.cli.cmd {
        CliCmd::Data(cmd) => run_data(&run, cmd),
        CliCmd::Database(cmd) => run_database(&run, cmd),
        CliCmd::Schema(cmd) => run_schema(&run, cmd),
    }
}

fn on_error(e: JsonSqlError) -> ExitCode {
    print!("error: {e:#?}");
    ExitCode::FAILURE
}

fn run_data(run: &RunState, cmd: &DataCli) -> JsonSqlResult<()> {
    todo!()
}

fn run_database(run: &RunState, cmd: &DatabaseCli) -> JsonSqlResult<()> {
    todo!()
}

fn run_schema(run: &RunState, cmd: &SchemaCli) -> JsonSqlResult<()> {
    let is_dir = cmd.dir;
    let dir = match is_dir {
        true => &cmd.input_path,
        false => {
            cmd.input_path.parent()
                .ok_or_else(|| JsonSqlError::FileIo { 
                    file: cmd.input_path.clone(), 
                    source: io::Error::new(io::ErrorKind::IsADirectory, ""),
                })?
        }
    };
    
    let mut cfg_tree = Stree::new();
    cfg_tree.set(
        &StreeKeys::new(vec!["*"]),
        Some(SchemaTableConfig {
            key: run.cli.key.clone(),
            db_id: run.cli.db_id.clone(),
            db_parent_id: run.cli.db_parent_id.clone(),
            db_list_id: run.cli.db_list_id.clone(),
        })
    );
    
    let cfg = SchemaConfig::new(cfg_tree)?;
    
    match is_dir {
        true => run_schema_dir(cmd, dir, &cfg),
        false => run_schema_file(cmd, dir, &cfg),
    }
}

fn run_schema_file(cmd: &SchemaCli, dir: &Path, cfg: &SchemaConfig) -> JsonSqlResult<()> {
    let file = cmd.input_path.file_name()
        .map(|f| PathBuf::from(f))
        .ok_or_else(|| JsonSqlError::UnsupportedFile { file: cmd.input_path.to_path_buf() })?;
    
    
    let json_schema = JsonSchema::from_file(&dir, &file, &cfg)?;
    let sqlite_schema = SqliteSchema::new(vec![json_schema]);
    sqlite_schema.write_file(&cmd.output_file)
}

fn run_schema_dir(cmd: &SchemaCli, dir: &Path, cfg: &SchemaConfig) -> JsonSqlResult<()> {
    let walk = walkdir::WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| match e.path().extension().and_then(|s| s.to_str()) {
            Some(EXT_JSONL) => true,
            _ => false,
        });
    
    let mut json_schemas = vec![];
    for entry in walk {
        let entry = entry.into_path();
        let file = entry.strip_prefix(dir)
            .map_err(|_| JsonSqlError::FileIo { file: entry.clone(), source: io::Error::new(io::ErrorKind::InvalidFilename, "") })?;
        
        println!("generating schema: {}", file.to_string_lossy());
        let json_schema = JsonSchema::from_file(&dir, file, &cfg)?;
        json_schemas.push(json_schema);
    }
    
    let sqlite_schema = SqliteSchema::new(json_schemas);
    println!("writing sql: {}", cmd.output_file.to_string_lossy());
    sqlite_schema.write_file(&cmd.output_file)?;
    
    Ok(())
}

