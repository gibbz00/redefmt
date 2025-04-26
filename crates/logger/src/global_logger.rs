use redefmt_common::FrontMatter;

use crate::*;

pub struct GlobalLogger;

impl GlobalLogger {
    fn write_start(&self, print_document_id: u128) -> LoggerHandle {
        let dispatcher = GlobalRegistry::dispatcher();

        dispatcher.acquire();

        let stamper = GlobalRegistry::stamper();

        let front_matter = FrontMatter::new(stamper.is_some());
        dispatcher.write(&[front_matter.bits()]);

        if let Some(stamp) = stamper.map(Stamper::stamp) {
            dispatcher.write(&stamp.to_le_bytes());
        }

        print_document_id.write_primitive(dispatcher);

        LoggerHandle { dispatcher }
    }
}

pub struct LoggerHandle {
    dispatcher: &'static dyn Dispatcher,
}

impl LoggerHandle {
    fn write_primitive(&self, primitive: impl WritePrimitive) {
        primitive.write_primitive(self.dispatcher);
    }
}

impl Drop for LoggerHandle {
    fn drop(&mut self) {
        self.dispatcher.release();
    }
}
