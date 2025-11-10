use std::ops::RangeInclusive;

use crate::prelude::*;

/// Template for adding tables decision
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "AddTables")]
pub struct AddTablesTemplate {
    /// Range of number of tables to add
    pub num_range: RangeInclusive<usize>,
}

/// Model for adding tables decision
#[derive(Debug, Clone)]
pub struct AddTablesModel {
    /// Number of tables to add
    pub num_tables: usize,
}

/// Template for removing tables decision
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "RemoveTables")]
pub struct RemoveTablesTemplate {
    /// Range of number of tables to add
    pub num_range: RangeInclusive<usize>,
}

/// Model for removing tables decision
#[derive(Debug, Clone)]
pub struct RemoveTablesModel {
    /// Number of tables to add
    pub num_tables: usize,
}

/// Template for disarranging tables decision
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "DisarrangeTables")]
pub struct DisarrangeTablesTemplate {
    /// Range of number of tables to add
    pub num_range: RangeInclusive<usize>,
}

/// Model for disarranging tables decision
#[derive(Debug, Clone)]
pub struct DisarrangeTablesModel {
    /// Number of tables to add
    pub num_tables: usize,
}

/// Template for opening a window decision
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "OpenWindow")]
pub struct OpenWindowTemplate {}

/// Model for opening a window decision
#[derive(Debug, Clone)]
pub struct OpenWindowModel {}

/// Template for closing a window decision
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "CloseWindow")]
pub struct CloseWindowTemplate {}

/// Model for closing a window decision
#[derive(Debug, Clone)]
pub struct CloseWindowModel {}

/// Template for changing window service decision
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "ChangeWindowService")]
pub struct ChangeWindowServiceTemplate {}

/// Model for changing window service decision
#[derive(Debug, Clone)]
pub struct ChangeWindowServiceModel {}
