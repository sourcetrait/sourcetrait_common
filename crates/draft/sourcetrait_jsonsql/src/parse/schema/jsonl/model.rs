use crate::*;

#[derive(Default, Debug)]
pub(crate) struct JsonlGuessNode {
    pub(crate) schema: Option<JsonlSchemaGuess>,
    pub(crate) column: Option<JsonlColumnGuess>,
}

pub(crate) type JsonlGuessTree = Stree<JsonlGuessNode>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct JsonlSchemaGuess {
    pub(crate) kind: JsonSchemaKind,
    pub(crate) source_id: StreeId,
    pub(crate) primary_key: PrimaryKey,
    pub(crate) parent_id_column: Option<String>,
    pub(crate) list_id_column: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JsonlColumnGuess {
    pub(crate) source_id: StreeId,
    pub(crate) data_guess: Option<JsonDataKind>,
    pub(crate) is_primary_key: bool,
    pub(crate) optional: bool,
}

#[derive(Debug)]
pub(crate) struct Guessed {
    pub kind: JsonSchemaKind,
    pub source_id: StreeId,
    pub parent_source_id: Option<StreeId>,
}

pub(crate) enum Data<'s> {
    Map(serde_json::Map<String, serde_json::Value>),
    Value(Cow<'s, str>, serde_json::Value),
}

impl<'s> Data<'s> {
    pub fn into_iter(self) -> Box<dyn Iterator<Item = (Cow<'s, str>, serde_json::Value)> + 's> {
        match self {
            Data::Map(m) => Box::new(m.into_iter().map(|(k, v)| (Cow::Owned(k), v))),
            Data::Value(k, v) => Box::new(std::iter::once((k, v))),
        } 
    }
}
