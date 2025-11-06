use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaConfig {
    tables: Stree<SchemaTableConfig>,
}

impl SchemaConfig {
    pub fn new(tables: Stree<SchemaTableConfig>) -> JsonSqlResult<Self> {
        Ok(Self {
            tables,
        })
    }
    
    pub fn find_table_or_default<'a>(&self, source: &'a StreeKeys<'a>) -> &SchemaTableConfig {
        self.tables.find(source)
            .map(|node| node.data().expect("data"))
            .unwrap_or_else(|| self.table_default())
    }
    
    pub fn find_table<'a>(&self, source: &'a StreeKeys<'a>) -> Option<&SchemaTableConfig> {
        self.tables.find(&source).and_then(StreeNode::data)
    }
    
    pub fn table_default(&self) -> &SchemaTableConfig {
        self.tables.get(1).data().expect("data")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemaTableConfig {
    pub key: Option<String>,
    pub db_id: String,
    pub db_parent_id: String,
    pub db_list_id: String,
}
