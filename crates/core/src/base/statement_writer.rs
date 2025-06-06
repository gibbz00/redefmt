use crate::*;

pub struct StatementWriter<'a, 'b> {
    formatter: &'a mut Formatter<'b>,
}

impl core::fmt::Write for StatementWriter<'_, '_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.formatter.dispatcher.write(s.as_bytes());
        Ok(())
    }
}

impl<'a, 'b> StatementWriter<'a, 'b> {
    pub(crate) fn init(formatter: &'a mut Formatter<'b>) -> Self {
        #[cfg(feature = "deferred")]
        formatter.write_raw_deferred(TypeHint::WriteStatements as u8);

        Self { formatter }
    }

    #[cfg(feature = "deferred")]
    #[doc(hidden)]
    pub fn write_statement_id(&mut self, crate_id: CrateId, write_id: WriteStatementId) {
        self.formatter.write_raw_deferred(StatementWriterHint::Continue as u8);
        self.formatter.write_raw_deferred(crate_id.as_ref());
        self.formatter.write_raw_deferred(write_id.as_ref());
    }

    #[cfg(feature = "deferred")]
    #[doc(hidden)]
    pub fn formatter(&'a mut self) -> &'a mut Formatter<'b> {
        self.formatter
    }
}

impl Drop for StatementWriter<'_, '_> {
    fn drop(&mut self) {
        #[cfg(feature = "deferred")]
        self.formatter.write_raw_deferred(StatementWriterHint::End as u8);
    }
}
