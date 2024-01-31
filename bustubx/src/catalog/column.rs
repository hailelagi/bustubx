use sqlparser::ast::ColumnDef;
use std::sync::Arc;

use crate::catalog::DataType;

pub type ColumnRef = Arc<Column>;

// 列定义
#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub data_type: DataType,
}

impl PartialEq for Column {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.data_type == other.data_type
    }
}

impl Column {
    pub fn new(name: String, data_type: DataType) -> Self {
        Self { name, data_type }
    }

    pub fn from_sqlparser_column(column_def: &ColumnDef) -> Self {
        let column_name = column_def.name.to_string();
        let column_type: DataType = (&column_def.data_type).try_into().unwrap();
        Self::new(column_name, column_type)
    }
}
