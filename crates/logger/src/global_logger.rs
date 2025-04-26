use crate::*;

pub struct GlobalLogger;

impl GlobalLogger {
    fn write_start(&self, print_document_id: u128) -> LoggerHandle {
        let dispatcher = GlobalRegistry::dispatcher();

        dispatcher.acquire();

        GlobalRegistry::stamper()
            .map(Stamper::stamp)
            .map(u64::from)
            .unwrap_or_default()
            .write_primitive(dispatcher);

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
