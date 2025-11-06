use crate::*;

pub type JsonSqlResult<T> = Result<T, JsonSqlError>;

#[derive(Debug, snafu::Snafu)]
pub enum JsonSqlError {
    FileIo { file: PathBuf, source: io::Error },
    SerdeJsonFile { file: PathBuf, source: serde_json::Error },
    JsonLinesFileNotObject { file: PathBuf, line_num: usize },
    UnsupportedFile { file: PathBuf },
    IncompatibleJsonLinesDataGuess { file: PathBuf, line_num: usize, existing: Option<JsonDataKind>, next: Option<JsonDataKind> },
    UndeterminedJsonLinesColumn { file: PathBuf, column_name: String },
    UndeterminedSchema { file: PathBuf, why: UndeterminedSchemaErr },
    UndeterminedChildSchema { file: PathBuf, name: String, why: UndeterminedSchemaErr },
    UndeterminedJsonLinesChildSchema { file: PathBuf, column_name: String, source: ChildSchemaError },
    SchemaConfig { why: SchemaConfigErr },
    Guess,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UndeterminedSchemaErr {
    PrimaryKey { num_found: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaConfigErr {
    DefaultTable,
}

#[derive(Debug, snafu::Snafu)]
pub enum ChildSchemaError {
    UndeterminedColumn { source_path: String },
    UndeterminedSubSchema { source_path: String },
    PrimaryKey { num_found: usize },
}

impl ChildSchemaError {
    pub(crate) fn source(&self) -> &str {
        match self {
            ChildSchemaError::UndeterminedColumn { source_path, .. } => source_path,
            ChildSchemaError::UndeterminedSubSchema { source_path, .. } => source_path,
            ChildSchemaError::PrimaryKey { .. } => "<primary_key>",
        }
    }
}
