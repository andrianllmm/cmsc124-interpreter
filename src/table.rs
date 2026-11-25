//! The table value: named columns of scalar cells.

use crate::value::Value;
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;

/// Why a table couldn't be built.
#[derive(Debug, Clone, PartialEq)]
pub enum TableError {
    /// Two columns share a name.
    DuplicateColumn(String),
    /// Columns differ in length, or the names and columns don't pair up.
    UnequalColumns,
    /// A cell that isn't a scalar. Holds the offending value's type, e.g. `"a list"`.
    NonScalarCell(&'static str),
}

impl Display for TableError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TableError::DuplicateColumn(name) => write!(f, "Duplicate column '{}'", name),
            TableError::UnequalColumns => write!(f, "Columns must have equal lengths"),
            TableError::NonScalarCell(found) => {
                write!(f, "Cell must be a scalar, got {}", found)
            }
        }
    }
}

/// Immutable named columns. Built only through `Table::new`, so every table
/// has unique names, equal-length columns, and scalar cells.
#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    names: Vec<String>,
    /// Behind an `Rc` so verbs can share a column between tables.
    columns: Vec<Rc<Vec<Value>>>,
}

impl Table {
    /// Checks the invariants, then builds the table.
    pub fn new(names: Vec<String>, columns: Vec<Rc<Vec<Value>>>) -> Result<Table, TableError> {
        if names.len() != columns.len() {
            return Err(TableError::UnequalColumns);
        }
        // Each name must not appear earlier in the header.
        for (i, name) in names.iter().enumerate() {
            if names[..i].contains(name) {
                return Err(TableError::DuplicateColumn(name.clone()));
            }
        }
        // Every column as long as the first.
        if let Some(first) = columns.first()
            && columns.iter().any(|c| c.len() != first.len())
        {
            return Err(TableError::UnequalColumns);
        }
        // Every cell a scalar.
        for cell in columns.iter().flat_map(|c| c.iter()) {
            if let Some(found) = non_scalar(cell) {
                return Err(TableError::NonScalarCell(found));
            }
        }
        Ok(Table { names, columns })
    }

    pub fn names(&self) -> &[String] {
        &self.names
    }

    pub fn columns(&self) -> &[Rc<Vec<Value>>] {
        &self.columns
    }

    /// Number of rows.
    pub fn len(&self) -> usize {
        self.columns.first().map_or(0, |c| c.len())
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// What a non-scalar value is called in an error, or `None` for a scalar.
fn non_scalar(value: &Value) -> Option<&'static str> {
    match value {
        Value::Int(_) | Value::Float(_) | Value::Str(_) | Value::Bool(_) | Value::Null => None,
        Value::List(_) => Some("a list"),
        Value::Table(_) => Some("a table"),
        Value::Native(_) | Value::Function(_) => Some("a function"),
    }
}

/// An aligned grid: header, a rule line, then one line per row.
/// Cells are left-aligned and padded to the widest in their column.
/// The last column isn't padded, so lines have no trailing spaces.
impl Display for Table {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Every cell as printed text, column by column.
        let cells: Vec<Vec<String>> = self
            .columns
            .iter()
            .map(|c| c.iter().map(|v| v.to_string()).collect())
            .collect();
        // Each column's width: its widest cell or its name, in characters.
        let widths: Vec<usize> = self
            .names
            .iter()
            .zip(&cells)
            .map(|(name, col)| {
                col.iter()
                    .map(|c| c.chars().count())
                    .chain([name.chars().count()])
                    .max()
                    .unwrap_or(0)
            })
            .collect();

        // One line of cells, padded to the column widths and joined by ` | `.
        let line = |texts: Vec<&str>| -> String {
            let last = texts.len().saturating_sub(1);
            texts
                .iter()
                .zip(&widths)
                .enumerate()
                .map(|(i, (t, w))| {
                    if i == last {
                        t.to_string()
                    } else {
                        format!("{:<w$}", t, w = w)
                    }
                })
                .collect::<Vec<_>>()
                .join(" | ")
        };

        // The header, the rule line, then the rows.
        write!(
            f,
            "{}",
            line(self.names.iter().map(String::as_str).collect())
        )?;
        let rule: Vec<String> = widths.iter().map(|w| "-".repeat(*w)).collect();
        write!(f, "\n{}", rule.join("-+-"))?;
        for row in 0..self.len() {
            write!(
                f,
                "\n{}",
                line(cells.iter().map(|c| c[row].as_str()).collect())
            )?;
        }
        Ok(())
    }
}
