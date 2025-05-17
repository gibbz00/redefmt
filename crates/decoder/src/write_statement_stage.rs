use redefmt_common::identifiers::CrateId;
use redefmt_db::{CrateDb, DbClient, crate_table::Crate};
use tokio_util::bytes::{Buf, BytesMut};

use crate::*;

#[derive(Default)]
pub enum WriteStatementWants<'caches> {
    #[default]
    CrateContext,
    Statement(CrateContext<'caches>),
}

#[derive(Clone, Copy)]
pub struct CrateContext<'caches> {
    id: CrateId,
    record: &'caches Crate<'static>,
    db: &'caches DbClient<CrateDb>,
}

impl<'caches> CrateContext<'caches> {
    pub fn new(stores: &DecoderStores<'caches>, src: &mut BytesMut) -> Result<Option<Self>, RedefmtDecoderError> {
        let Some(id) = src.try_get_u16().ok().map(CrateId::new) else {
            return Ok(None);
        };

        let (db, record) = stores.get_or_insert_crate(id)?;

        Ok(Some(CrateContext { id, record, db }))
    }
}
