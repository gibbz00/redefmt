use crate::*;

#[derive(Debug)]
pub enum GlobalLoggerError {
    StamperAlreadyInitialized,
    LoggerAlreadyInitialized,
}

pub struct GlobalLogger {
    handle: GlobalDispatcherHandle,
}

impl GlobalLogger {
    #[cfg(feature = "alloc")]
    pub fn init_alloc_logger(
        dispatcher: impl Dispatcher + Send + Sync + 'static,
        stamper: Option<&'static dyn Stamper>,
    ) -> Result<(), GlobalLoggerError> {
        if let Some(stamper) = stamper {
            GlobalStamper::init(stamper)?;
        }

        GlobalDispatcher::init_boxed(dispatcher)
    }

    pub fn init_static_logger(
        dispatcher: &'static mut dyn Dispatcher,
        stamper: Option<&'static dyn Stamper>,
    ) -> Result<(), GlobalLoggerError> {
        if let Some(stamper) = stamper {
            GlobalStamper::init(stamper)?;
        }

        GlobalDispatcher::init_static(dispatcher)
    }

    // Acquires global dispatcher
    //
    // Hidden because it should only be used by print proc-macros
    #[doc(hidden)]
    pub fn write_start(print_id: (CrateId, PrintStatementId), level: Option<Level>) -> Self {
        let mut handle = GlobalDispatcher::global_dispatcher();

        let stamper = GlobalStamper::stamper();

        let header = Header::new(stamper.is_some(), level);
        handle.get(|dispatcher| dispatcher.write(&[header.bits()]));

        if let Some(stamp) = stamper.map(Stamper::stamp) {
            handle.get(|dispatcher| dispatcher.write(&stamp.as_ref().to_be_bytes()));
        }

        let (crate_id, print_statement_id) = print_id;

        handle.get(|dispatcher| dispatcher.write(&crate_id.as_ref().to_be_bytes()));
        handle.get(|dispatcher| dispatcher.write(&print_statement_id.as_ref().to_be_bytes()));

        Self { handle }
    }

    // Dynamic dispatch on `format` to reduce code monoporphization
    //
    // Hidden because it should only be used by print proc-macros
    #[doc(hidden)]
    pub fn write_format(&mut self, format: &dyn Format) {
        self.handle.get(|dispatcher| {
            let mut formatter = Formatter::new(dispatcher);
            // TODO: do anything with fmt error?
            let _ = format.fmt(&mut formatter);
        });
    }

    // Releases global dispatcher
    //
    // Hidden because it should only be used by print proc-macros
    #[doc(hidden)]
    pub fn write_end(self) {
        drop(self)
    }
}
