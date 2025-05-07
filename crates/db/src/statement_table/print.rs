use std::borrow::Cow;

use crate::*;

statement_table!(PrintStatementId, PrintStatement<'_>, "print_register");

#[derive(Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PrintStatement<'a> {
    info: PrintInfo<'a>,
    #[serde(borrow)]
    segments: Vec<Segment<'a>>,
}

#[derive(Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PrintInfo<'a> {
    /// None for regular `println!()` statements
    level: Option<LogLevel>,
    location: Location<'a>,
}

/// Print statement call site location
///
/// Crate inferred could be inferred the crate database itself, or by "parsing" the module path.
#[derive(Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Location<'a> {
    /// Usually retrieved by `file!()`
    file: Cow<'a, str>,
    /// Usually retrieved by `module_path!()`
    module: Cow<'a, str>,
    /// Usually retrieved by `line!()`
    line: u32,
}

#[derive(Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    statement_table_tests!(PrintStatement);

    impl StatementTableTest for PrintStatement<'_> {
        fn mock_id() -> Self::Id {
            PrintStatementId(ShortId(123))
        }

        fn mock() -> Self {
            PrintStatement { segments: vec![Segment::Str("x".into())], info: mock_print_info() }
        }

        fn mock_other() -> Self {
            PrintStatement { segments: vec![Segment::Str("y".into())], info: mock_print_info() }
        }
    }

    fn mock_print_info() -> PrintInfo<'static> {
        PrintInfo {
            level: Some(LogLevel::Debug),
            location: Location { file: file!().into(), module: module_path!().into(), line: line!() },
        }
    }
}
