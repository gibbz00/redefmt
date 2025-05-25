use std::borrow::Cow;

use redefmt_internal::identifiers::PrintStatementId;

use crate::*;

impl StatementTable for PrintStatement<'_> {
    type Id = PrintStatementId;

    const NAME: &'static str = "print_register";
}

#[derive(Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, derive_getters::Getters)]
pub struct PrintStatement<'a> {
    info: PrintInfo<'a>,
    #[serde(borrow)]
    format_expression: StoredFormatExpression<'a>,
}

impl<'a> PrintStatement<'a> {
    pub fn new(info: PrintInfo<'a>, format_expression: StoredFormatExpression<'a>) -> Self {
        Self { info, format_expression }
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
    use redefmt_args::mapped_format_expression;

    use super::*;

    statement_table_tests!(PrintStatement);

    impl StatementTableTest for PrintStatement<'_> {
        fn mock_id() -> Self::Id {
            PrintStatementId::new(123)
        }

        fn mock() -> Self {
            PrintStatement::new(
                mock_print_info(),
                StoredFormatExpression::new(mapped_format_expression!("x"), false),
            )
        }

        fn mock_other() -> Self {
            PrintStatement::new(
                mock_print_info(),
                StoredFormatExpression::new(mapped_format_expression!("y"), false),
            )
        }
    }

    fn mock_print_info() -> PrintInfo<'static> {
        PrintInfo { level: Some(LogLevel::Debug), location: location!() }
    }
}
