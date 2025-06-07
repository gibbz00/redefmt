//! redefmt-end-to-end
//!
//! End to end testing with all permutations of deferred and non-deferred formatting.

#![allow(missing_docs)]

// Not placed in a tests directory as dev-dependencies doesn't support optional dependencies.
#[cfg(all(test, feature = "deferred"))]
mod tests {
    use redefmt::{Level, logger::GlobalLogger};
    use redefmt_core::{SharedTestDispatcher, logger::TestStamper};
    use redefmt_decoder::{RedefmtDecoder, RedefmtDecoderCache};
    use redefmt_pretty_printer::{
        PrettyPrinter,
        config::{PrettyPrinterConfig, PrintStampConfig},
    };

    macro_rules! assert_print {
        ($dispatcher:expr, $decoder:expr, $printer:expr, $log_expr:expr, $expected:literal) => {{
            $log_expr;
            let expected = format!($expected);
            assert_print_impl(&mut $dispatcher, &mut $decoder, &mut $printer, &expected);
        }};
    }

    fn assert_print_impl(
        dispatcher: &mut SharedTestDispatcher,
        decoder: &mut RedefmtDecoder,
        printer: &mut PrettyPrinter,
        expected: &str,
    ) {
        let frame = decoder.decode(&mut dispatcher.take_bytes()).unwrap().unwrap();
        let actual = printer.format(frame).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn end_to_end() {
        static STAMPER: TestStamper = TestStamper::new();
        let dispatcher = SharedTestDispatcher::new();
        GlobalLogger::init_alloc_logger(dispatcher.clone(), Some(&STAMPER)).unwrap();

        let decoder_cache = RedefmtDecoderCache::default();
        let decoder = RedefmtDecoder::new(&decoder_cache).unwrap();

        let printer_config = PrettyPrinterConfig::new(PrintStampConfig::Counter);
        let printer = PrettyPrinter::new(printer_config);

        let value = 10;
        let crate_name = env!("CARGO_PKG_NAME");

        // Assertions in same test as global logger may only be set once.

        let mut l = dispatcher;
        let mut d = decoder;
        let mut p = printer;

        assert_print!(l, d, p, redefmt::print!("{value}"), "0 [NONE] - {crate_name}: {value}");
        assert_print!(
            l,
            d,
            p,
            redefmt::println!("{value}"),
            "1 [NONE] - {crate_name}: {value}\n"
        );
        assert_print!(
            l,
            d,
            p,
            redefmt::trace!("{value}"),
            "2 [TRACE] - {crate_name}: {value}\n"
        );
        assert_print!(
            l,
            d,
            p,
            redefmt::debug!("{value}"),
            "3 [DEBUG] - {crate_name}: {value}\n"
        );
        assert_print!(l, d, p, redefmt::info!("{value}"), "4 [INFO] - {crate_name}: {value}\n");
        assert_print!(l, d, p, redefmt::warn!("{value}"), "5 [WARN] - {crate_name}: {value}\n");
        assert_print!(
            l,
            d,
            p,
            redefmt::error!("{value}"),
            "6 [ERROR] - {crate_name}: {value}\n"
        );
        assert_print!(
            l,
            d,
            p,
            redefmt::log!(Level::Trace, "{value}"),
            "7 [TRACE] - {crate_name}: {value}\n"
        );
        assert_print!(
            l,
            d,
            p,
            redefmt::log!(Level::Debug, "{value}"),
            "8 [DEBUG] - {crate_name}: {value}\n"
        );
        assert_print!(
            l,
            d,
            p,
            redefmt::log!(Level::Info, "{value}"),
            "9 [INFO] - {crate_name}: {value}\n"
        );
        assert_print!(
            l,
            d,
            p,
            redefmt::log!(Level::Warn, "{value}"),
            "10 [WARN] - {crate_name}: {value}\n"
        );
        assert_print!(
            l,
            d,
            p,
            redefmt::log!(Level::Error, "{value}"),
            "11 [ERROR] - {crate_name}: {value}\n"
        );

        // level can be full-blown expression
        assert_print!(
            l,
            d,
            p,
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
            l,
            d,
            p,
            redefmt::print!("{FooWrite:?}"),
            "13 [NONE] - {crate_name}: \"x\\ny\""
        );

        // derive unit struct
        #[derive(Debug, redefmt::Format)]
        struct FooUnit;
        let value = FooUnit;
        assert_print!(
            l,
            d,
            p,
            redefmt::print!("{value:?}"),
            "14 [NONE] - {crate_name}: FooUnit"
        );

        // generics
        #[derive(Debug, redefmt::Format)]
        struct FooGeneric<T>(T);
        assert_print!(
            l,
            d,
            p,
            redefmt::print!("{:?}", FooGeneric(1)),
            "15 [NONE] - {crate_name}: FooGeneric(1)"
        );
        assert_print!(
            l,
            d,
            p,
            redefmt::print!("{:?}", FooGeneric("x")),
            "16 [NONE] - {crate_name}: FooGeneric(\"x\")"
        );

        // derive named struct without fields
        #[derive(Debug, redefmt::Format)]
        struct FooNamed {}
        let value = FooNamed {};
        assert_print!(
            l,
            d,
            p,
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
            l,
            d,
            p,
            redefmt::print!("{value:?}"),
            "18 [NONE] - {crate_name}: BarNamed {{ a: FooUnit, b: FooUnit }}"
        );

        // derive empty tuple struct
        #[derive(Debug, redefmt::Format)]
        struct FooTuple();
        let value = FooTuple();
        assert_print!(
            l,
            d,
            p,
            redefmt::print!("{value:?}"),
            "19 [NONE] - {crate_name}: FooTuple"
        );

        // derive non-empty tuple struct
        #[derive(Debug, redefmt::Format)]
        struct BarTuple(FooUnit, FooUnit);
        let value = BarTuple(FooUnit, FooUnit);
        assert_print!(
            l,
            d,
            p,
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
        assert_print!(l, d, p, redefmt::print!("{value:?}"), "21 [NONE] - {crate_name}: Unit");
        let value = FooEnum::Tuple(1, 2);
        assert_print!(
            l,
            d,
            p,
            redefmt::print!("{value:?}"),
            "22 [NONE] - {crate_name}: Tuple(1, 2)"
        );
        let value = FooEnum::Named { a: 1, b: 2 };
        assert_print!(
            l,
            d,
            p,
            redefmt::print!("{value:?}"),
            "23 [NONE] - {crate_name}: Named {{ a: 1, b: 2 }}"
        );

        // derive on enum with no variants
        #[allow(unused)]
        #[derive(redefmt::Format)]
        enum FooEnumInfallible {}
    }
}
