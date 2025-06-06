use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StatementWriterHint {
    Continue = 0,
    End = 1,
}

impl StatementWriterHint {
    pub const fn from_repr(repr: u8) -> Option<Self> {
        match repr {
            0 => Some(Self::Continue),
            1 => Some(Self::End),
            _ => None,
        }
    }
}

pub struct StatementWriter<'a, 'b> {
    formatter: &'a mut Formatter<'b>,
}

impl<'a, 'b> StatementWriter<'a, 'b> {
    pub(crate) fn init(formatter: &'a mut Formatter<'b>) -> Self {
        formatter.write_raw(TypeHint::WriteStatements as u8);

        Self { formatter }
    }

    #[doc(hidden)]
    pub fn write_statement_id(&mut self, crate_id: CrateId, write_id: WriteStatementId) {
        self.formatter.write_raw(StatementWriterHint::Continue as u8);
        self.formatter.write_raw(crate_id.as_ref());
        self.formatter.write_raw(write_id.as_ref());
    }

    #[doc(hidden)]
    pub fn formatter(&'a mut self) -> &'a mut Formatter<'b> {
        self.formatter
    }
}

impl Drop for StatementWriter<'_, '_> {
    fn drop(&mut self) {
        self.formatter.write_raw(StatementWriterHint::End as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hint_repr_bijectivity() {
        assert_bijectivity(StatementWriterHint::Continue);
        assert_bijectivity(StatementWriterHint::End);

        fn assert_bijectivity(hint: StatementWriterHint) {
            let repr = hint as u8;
            let from_repr = StatementWriterHint::from_repr(repr).unwrap();
            assert_eq!(hint, from_repr);
        }
    }
}
