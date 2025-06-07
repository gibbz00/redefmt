use crate::*;

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
