use tokio_util::bytes::BytesMut;

use crate::*;

pub struct WriteStatementDecoder<'caches> {
    pub write_crate: CrateContext<'caches>,
}

impl<'caches> WriteStatementDecoder<'caches> {
    pub fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self>, RedefmtDecoderError> {
        // TODO: find write statement
        // self.crate_context.db.find_by_id(id)
        todo!()
    }
}
