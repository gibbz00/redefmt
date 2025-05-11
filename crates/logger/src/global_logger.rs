use redefmt_common::{
    codec::Header,
    identifiers::{CrateId, PrintStatementId},
};

use crate::*;

pub struct GlobalLogger;

impl GlobalLogger {
    fn write_start(&self, print_id: (CrateId, PrintStatementId)) -> GlobalDispatcherHandle {
        let mut dispatcher = GlobalDispatcher::dispatcher();

        let stamper = GlobalStamper::stamper();

        let header = Header::new(stamper.is_some());
        dispatcher.write(&[header.bits()]);

        if let Some(stamp) = stamper.map(Stamper::stamp) {
            dispatcher.write(&stamp.as_ref().to_be_bytes());
        }

        let (crate_id, print_statement_id) = print_id;

        dispatcher.write(&crate_id.as_ref().to_be_bytes());
        dispatcher.write(&print_statement_id.as_ref().to_be_bytes());

        dispatcher
    }
}
