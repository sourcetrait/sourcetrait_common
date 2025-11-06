use crate::*;

/// Converts JsonL into SQLite schema and data
/// 
/// Nested objects and lists of objects are represented as separate tables
/// with foreign key constraints.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(clap::Parser)]
pub struct Cli {
    /// Field from the source JSON that represents a primary key, if available 
    #[clap(long, short = 'k')]
    pub key: Option<String>,
    /// Column name for local data IDs
    #[clap(long, alias = "id", default_value_t = String::from("_db_id"))]
    pub db_id: String,
    /// Column name for local data IDs referencing a parent table.
    /// Used for nested objects and object lists
    #[clap(long, alias = "parent_id", default_value_t = String::from("_db_parent_id"))]
    pub db_parent_id: String,
    /// Column name for local indexes used.
    /// Used for enumerating nested object lists.
    #[clap(long, alias = "list_id", default_value_t = String::from("_db_list_id"))]
    pub db_list_id: String,
    #[clap(subcommand)]
    pub cmd: CliCmd,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum InputKind {
    JsonLines,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum OutputKind {
    Sqlite,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(clap::Subcommand)]
pub enum CliCmd {
    Schema(SchemaCli),
    Data(DataCli),
    Database(DatabaseCli),
}

/// Converts a JSON into a single SQL schema file
/// 
/// Defaults to a JSON file, unless --dir is passed.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(clap::Parser)]
pub struct SchemaCli {
    #[clap(long, short = 'd', default_value_t = false)]
    pub dir: bool,
    pub input_path: PathBuf,
    pub output_file: PathBuf,
}

/// Converts a single JSON file into a single SQL data file
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(clap::Parser)]
pub struct DataCli {
    pub input_file: PathBuf,
    pub output_file: PathBuf,
}

/// Converts an entire directory structure into a schema and data file
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(clap::Parser)]
pub struct DatabaseCli {
    pub input_dir: PathBuf,
    pub output_schema_file: PathBuf,
    pub output_data_file: PathBuf,
}
