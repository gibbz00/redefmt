//! redefmt-end-to-end
//!
//! End to end testing with all permutations of deferred and non-deferred
//! formatting.
//!
//! Usually executed with `cargo hack --package redefmt-end-to-end --feature-powerset test`
#![allow(missing_docs)]

/// Test that it compiles without any features enabled
#[cfg(all(test, not(feature = "deferred"), not(feature = "print"), not(feature = "log")))]
mod tests {
    #[allow(unused_imports)]
    use redefmt::*;
}

// TODO: automate output verification
#[cfg(all(test, not(feature = "deferred"), feature = "print", not(feature = "log")))]
mod tests {
    #[test]
    fn print_non_deferred() {
        let x = 1;
        redefmt::print!("{x}");
        redefmt::println!("{x}");
    }
}

// TODO: automate output verification
#[cfg(all(test, feature = "deferred", feature = "print", not(feature = "log")))]
mod tests {
    #[test]
    fn print_combined() {
        let x = 1;
        redefmt::print!("{x}");
        redefmt::println!("{x}");
    }
}

#[cfg(all(test, not(feature = "deferred"), not(feature = "print"), feature = "log"))]
mod tests {
    use crate::*;

    #[test]
    fn log_non_deferred() {
        let logger = TestLogger::init();

        redefmt::log!(redefmt::Level::Trace, "1");
        redefmt::trace!("2");
        redefmt::log!(redefmt::Level::Debug, "3");
        redefmt::debug!("4");
        redefmt::log!(redefmt::Level::Info, "5");
        redefmt::info!("6");
        redefmt::log!(redefmt::Level::Warn, "7");
        redefmt::warn!("8");
        redefmt::log!(redefmt::Level::Error, "9");
        redefmt::error!("10");

        logger.assert_logs(&[
            "TRACE - 1",
            "TRACE - 2",
            "DEBUG - 3",
            "DEBUG - 4",
            "INFO - 5",
            "INFO - 6",
            "WARN - 7",
            "WARN - 8",
            "ERROR - 9",
            "ERROR - 10",
        ]);
    }
}

#[cfg(all(test, feature = "deferred", feature = "print", feature = "log"))]
mod tests {
    use redefmt::logger::GlobalLogger;
    use redefmt_core::SharedTestDispatcher;
    use redefmt_decoder::{RedefmtDecoder, RedefmtDecoderCache};
    use redefmt_pretty_printer::{
        PrettyPrinter,
        config::{PrettyPrinterConfig, PrintStampConfig},
    };

    use crate::*;

    #[test]
    fn log_combined() {
        let logger = TestLogger::init();

        let mut dispatcher = SharedTestDispatcher::new();

        GlobalLogger::init_alloc_logger(dispatcher.clone(), None).unwrap();

        let decoder_cache = RedefmtDecoderCache::default();
        let mut decoder = RedefmtDecoder::new(&decoder_cache).unwrap();

        let printer_config =
            PrettyPrinterConfig::new_with_format(PrintStampConfig::Counter, "{level} - {statement}").unwrap();
        let mut printer = PrettyPrinter::new(printer_config);

        assert_print!(dispatcher, decoder, printer, redefmt::info!("10"), "INFO - 10\n");

        logger.assert_logs(&["INFO - 10"]);
    }
}

#[cfg(all(test, feature = "deferred", not(feature = "print"), not(feature = "log")))]
mod tests {
    use redefmt::{Level, logger::GlobalLogger};
    use redefmt_core::{SharedTestDispatcher, logger::TestStamper};
    use redefmt_decoder::{RedefmtDecoder, RedefmtDecoderCache};
    use redefmt_pretty_printer::{
        PrettyPrinter,
        config::{PrettyPrinterConfig, PrintStampConfig},
    };

    use crate::*;

    #[test]
    fn deferred() {
        static STAMPER: TestStamper = TestStamper::new();
        let dispatcher = SharedTestDispatcher::new();
        GlobalLogger::init_alloc_logger(dispatcher.clone(), Some(&STAMPER)).unwrap();

        let decoder_cache = RedefmtDecoderCache::default();
        let decoder = RedefmtDecoder::new(&decoder_cache).unwrap();

        let printer_config = PrettyPrinterConfig::new(PrintStampConfig::Counter);
        let printer = PrettyPrinter::new(printer_config);

        // Assertions in same test as global logger may only be set once.

        let mut dispatcher = dispatcher;
        let mut decoder = decoder;
        let mut printer = printer;

        let crate_name = env!("CARGO_PKG_NAME");

        let value = 10;
        assert_print!(
            dispatcher,
            decoder,
            printer,
            redefmt::print!("{value}"),
            "0 [NONE] - {crate_name}: {value}"
        );

        assert_print!(
            dispatcher,
            decoder,
            printer,
            redefmt::println!("{value}"),
            "1 [NONE] - {crate_name}: {value}\n"
        );

        assert_print!(
            dispatcher,
            decoder,
            printer,
            redefmt::trace!("{value}"),
            "2 [TRACE] - {crate_name}: {value}\n"
        );

        assert_print!(
            dispatcher,
            decoder,
            printer,
            redefmt::debug!("{value}"),
            "3 [DEBUG] - {crate_name}: {value}\n"
        );

        assert_print!(
            dispatcher,
            decoder,
            printer,
            redefmt::info!("{value}"),
            "4 [INFO] - {crate_name}: {value}\n"
        );

        assert_print!(
            dispatcher,
            decoder,
            printer,
            redefmt::warn!("{value}"),
            "5 [WARN] - {crate_name}: {value}\n"
        );

        assert_print!(
            dispatcher,
            decoder,
            printer,
            redefmt::error!("{value}"),
            "6 [ERROR] - {crate_name}: {value}\n"
        );

        assert_print!(
            dispatcher,
            decoder,
            printer,
            redefmt::log!(Level::Trace, "{value}"),
            "7 [TRACE] - {crate_name}: {value}\n"
        );

        assert_print!(
            dispatcher,
            decoder,
            printer,
            redefmt::log!(Level::Debug, "{value}"),
            "8 [DEBUG] - {crate_name}: {value}\n"
        );

        assert_print!(
            dispatcher,
            decoder,
            printer,
            redefmt::log!(Level::Info, "{value}"),
            "9 [INFO] - {crate_name}: {value}\n"
        );

        assert_print!(
            dispatcher,
            decoder,
            printer,
            redefmt::log!(Level::Warn, "{value}"),
            "10 [WARN] - {crate_name}: {value}\n"
        );

        assert_print!(
            dispatcher,
            decoder,
            printer,
            redefmt::log!(Level::Error, "{value}"),
            "11 [ERROR] - {crate_name}: {value}\n"
        );

        // level can be full-blown expression
        assert_print!(
            dispatcher,
            decoder,
            printer,
            redefmt::log!(
                if true {
                    redefmt::Level::Info
                } else {
                    redefmt::Level::Error
                },
                "{value}"
            ),
            "12 [INFO] - {crate_name}: {value}\n"
        );

        // manual impl
        #[derive(Debug)]
        struct FooWrite;
        impl redefmt::Format for FooWrite {
            fn fmt(&self, f: &mut redefmt::Formatter) -> ::core::fmt::Result {
                let mut s = f.write_statements();
                redefmt::writeln!(s, "x")?;
                redefmt::write!(s, "y")?;
                Ok(())
            }
        }

        // note how FooWrite can be captured and created at the same time
        assert_print!(
            dispatcher,
            decoder,
            printer,
            redefmt::print!("{FooWrite:?}"),
            "13 [NONE] - {crate_name}: \"x\\ny\""
        );

        // derive unit struct
        #[derive(Debug, redefmt::Format)]
        struct FooUnit;
        let value = FooUnit;
        assert_print!(
            dispatcher,
            decoder,
            printer,
            redefmt::print!("{value:?}"),
            "14 [NONE] - {crate_name}: FooUnit"
        );

        // generics
        #[derive(Debug, redefmt::Format)]
        struct FooGeneric<T>(T);
        assert_print!(
            dispatcher,
            decoder,
            printer,
            redefmt::print!("{:?}", FooGeneric(1)),
            "15 [NONE] - {crate_name}: FooGeneric(1)"
        );
        assert_print!(
            dispatcher,
            decoder,
            printer,
            redefmt::print!("{:?}", FooGeneric("x")),
            "16 [NONE] - {crate_name}: FooGeneric(\"x\")"
        );

        // derive named struct without fields
        #[derive(Debug, redefmt::Format)]
        struct FooNamed {}
        let value = FooNamed {};
        assert_print!(
            dispatcher,
            decoder,
            printer,
            redefmt::print!("{value:?}"),
            "17 [NONE] - {crate_name}: FooNamed"
        );

        // derive named struct with fields
        #[derive(Debug, redefmt::Format)]
        struct BarNamed {
            a: FooUnit,
            b: FooUnit,
        }
        let value = BarNamed { a: FooUnit, b: FooUnit };
        assert_print!(
            dispatcher,
            decoder,
            printer,
            redefmt::print!("{value:?}"),
            "18 [NONE] - {crate_name}: BarNamed {{ a: FooUnit, b: FooUnit }}"
        );

        // derive empty tuple struct
        #[derive(Debug, redefmt::Format)]
        struct FooTuple();
        let value = FooTuple();
        assert_print!(
            dispatcher,
            decoder,
            printer,
            redefmt::print!("{value:?}"),
            "19 [NONE] - {crate_name}: FooTuple"
        );

        // derive non-empty tuple struct
        #[derive(Debug, redefmt::Format)]
        struct BarTuple(FooUnit, FooUnit);
        let value = BarTuple(FooUnit, FooUnit);
        assert_print!(
            dispatcher,
            decoder,
            printer,
            redefmt::print!("{value:?}"),
            "20 [NONE] - {crate_name}: BarTuple(FooUnit, FooUnit)"
        );

        // derive on enum
        #[derive(Debug, redefmt::Format)]
        enum FooEnum {
            Unit,
            Tuple(usize, usize),
            Named { a: usize, b: usize },
        }

        let value = FooEnum::Unit;
        assert_print!(
            dispatcher,
            decoder,
            printer,
            redefmt::print!("{value:?}"),
            "21 [NONE] - {crate_name}: Unit"
        );

        let value = FooEnum::Tuple(1, 2);
        assert_print!(
            dispatcher,
            decoder,
            printer,
            redefmt::print!("{value:?}"),
            "22 [NONE] - {crate_name}: Tuple(1, 2)"
        );

        let value = FooEnum::Named { a: 1, b: 2 };
        assert_print!(
            dispatcher,
            decoder,
            printer,
            redefmt::print!("{value:?}"),
            "23 [NONE] - {crate_name}: Named {{ a: 1, b: 2 }}"
        );

        // derive on enum with no variants
        #[allow(unused)]
        #[derive(redefmt::Format)]
        enum FooEnumInfallible {}
    }
}

#[allow(unused_imports)]
#[cfg(feature = "log")]
pub(crate) use test_logger::TestLogger;
#[allow(unused)]
#[cfg(feature = "log")]
mod test_logger {
    #[derive(Clone)]
    pub struct TestLogger(std::sync::Arc<std::sync::Mutex<Vec<String>>>);

    impl TestLogger {
        pub fn init() -> Self {
            log::set_max_level(log::LevelFilter::Trace);

            let this = Self(Default::default());

            log::set_boxed_logger(Box::new(this.clone())).unwrap();

            this
        }

        pub fn assert_logs(&self, expected: &[&'static str]) {
            let handle = self.0.lock().unwrap();

            let actual_logs = handle.iter().map(|row| row.as_str()).collect::<Vec<_>>();

            assert_eq!(expected, &actual_logs)
        }
    }

    impl log::Log for TestLogger {
        fn enabled(&self, _metadata: &log::Metadata) -> bool {
            true
        }

        fn log(&self, record: &log::Record) {
            if record
                .module_path()
                .is_some_and(|path| path == "redefmt_end_to_end::tests")
            {
                let mut handle = self.0.lock().unwrap();
                handle.push(format!("{} - {}", record.level(), record.args()));
            }
        }

        fn flush(&self) {}
    }
}

#[allow(unused)]
#[cfg(feature = "deferred")]
mod test_deferred {
    use redefmt_core::SharedTestDispatcher;
    use redefmt_decoder::RedefmtDecoder;
    use redefmt_pretty_printer::PrettyPrinter;

    #[macro_export]
    macro_rules! assert_print {
        ($dispatcher:expr, $decoder:expr, $printer:expr, $log_expr:expr, $expected:literal) => {{
            $log_expr;
            let expected = format!($expected);
            $crate::test_deferred::assert_print_impl(&mut $dispatcher, &mut $decoder, &mut $printer, &expected);
        }};
    }

    pub fn assert_print_impl(
        dispatcher: &mut SharedTestDispatcher,
        decoder: &mut RedefmtDecoder,
        printer: &mut PrettyPrinter,
        expected: &str,
    ) {
        let frame = decoder.decode(&mut dispatcher.take_bytes()).unwrap().unwrap();
        let actual = printer.format(frame).unwrap();
        assert_eq!(expected, actual);
    }
}
