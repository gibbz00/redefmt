use std::borrow::Cow;

use redefmt_common::identifiers::PrintStatementId;

use crate::*;

impl StatementTable for PrintStatement<'_> {
    type Id = PrintStatementId;

    const NAME: &'static str = "print_register";
}

#[derive(Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, derive_getters::Getters)]
pub struct PrintStatement<'a> {
    info: PrintInfo<'a>,
    #[serde(borrow)]
    segments: Vec<Segment<'a>>,
}

impl<'a> PrintStatement<'a> {
    pub fn new(info: PrintInfo<'a>, segments: Vec<Segment<'a>>) -> Self {
        Self { info, segments }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PrintInfo<'a> {
    /// None for regular `println!()` statements
    level: Option<LogLevel>,
    location: Location<'a>,
}

impl<'a> PrintInfo<'a> {
    pub fn new(level: Option<LogLevel>, location: Location<'a>) -> Self {
        Self { level, location }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Print statement call site location
///
/// Constructed with the [`location!`] macro.
///
/// Crate inferred could be inferred the crate database itself, or by "parsing" the module path.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Location<'a> {
    /// Usually retrieved by `file!()`
    file: Cow<'a, str>,
    /// Usually retrieved by `module_path!()`
    module: Cow<'a, str>,
    /// Usually retrieved by `line!()`
    line: u32,
}

impl<'a> Location<'a> {
    #[doc(hidden)]
    pub fn new(file: Cow<'a, str>, module: Cow<'a, str>, line: u32) -> Self {
        Self { file, module, line }
    }
}

#[macro_export]
macro_rules! location {
    () => {
        $crate::statement_table::print::Location::new(file!().into(), module_path!().into(), line!())
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    statement_table_tests!(PrintStatement);

    impl StatementTableTest for PrintStatement<'_> {
        fn mock_id() -> Self::Id {
            PrintStatementId::new(123)
        }

        fn mock() -> Self {
            PrintStatement::new(mock_print_info(), vec![Segment::Str("x".into())])
        }

        fn mock_other() -> Self {
            PrintStatement::new(mock_print_info(), vec![Segment::Str("y".into())])
        }
    }

    fn mock_print_info() -> PrintInfo<'static> {
        PrintInfo { level: Some(LogLevel::Debug), location: location!() }
    }
}
