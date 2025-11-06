use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct JsonSchema {
    pub kind: JsonSchemaKind,
    pub source: String,
    pub parent_name: Option<String>,
    pub name: String,
    pub primary_key: PrimaryKey,
    pub parent_id_column: Option<String>,
    pub list_id_column: Option<String>,
    pub columns: HashMap<String, JsonColumn>,
    pub child_schemas: HashMap<String, Self>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub enum JsonSchemaKind {
    Object,
    ObjectList,
    PrimitiveList,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub enum PrimaryKey {
    Db(String),
    Source {
        db: String,
        source: String,
    }
}

impl PrimaryKey {
    pub fn db(&self) -> &str {
        match self {
            PrimaryKey::Db(db) => db,
            PrimaryKey::Source { db, .. } => db,
        }
    }
    
    pub fn source(&self) -> Option<&str> {
        match self {
            PrimaryKey::Db(_) => None,
            PrimaryKey::Source { source, .. } => Some(source),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub struct JsonColumn {
    pub source: String,
    pub name: String,
    pub data_kind: JsonDataKind,
    pub optional: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub enum JsonDataKind {
    Bool,
    String,
    Number(JsonNumberKind),
    Object,
    ObjectList,
    PrimitiveList(JsonPrimitiveKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub enum JsonPrimitiveKind {
    Bool,
    String,
    Number(JsonNumberKind),
}

impl From<JsonPrimitiveKind> for JsonDataKind {
    fn from(primitive: JsonPrimitiveKind) -> Self {
        match primitive {
            JsonPrimitiveKind::Bool => JsonDataKind::Bool,
            JsonPrimitiveKind::String => JsonDataKind::String,
            JsonPrimitiveKind::Number(n) => JsonDataKind::Number(n),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub enum JsonNumberKind {
    Integer(JsonIntegerKind),
    Float,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub enum JsonIntegerKind {
    Signed,
    Unsigned,
}

pub(crate) const EXT_JSONL: &'static str = "jsonl";
//pub(crate) const EXT_JSON: &'static str = "json";
pub(crate) const EXT_SCHEMA_JSON: &'static str = "schema.json";

impl JsonSchema {
    pub fn from_file(dir: &Path, file: &Path, cfg: &SchemaConfig) -> JsonSqlResult<Self> {
        let file_ext = dir.join(file).extension()
            .map(|ext| ext.to_string_lossy().to_lowercase())
            .ok_or_else(|| JsonSqlError::UnsupportedFile { file: file.into() })?;
        
        match file_ext.as_str() {
            EXT_JSONL => Self::from_jsonl_file(dir, file, cfg),
            _ => Err(JsonSqlError::UnsupportedFile { file: file.into() }),
        } 
    }
}
