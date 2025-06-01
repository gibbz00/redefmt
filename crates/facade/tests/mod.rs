#![allow(missing_docs)]

use redefmt::{Level, logger::GlobalLogger};
use redefmt_args::deferred_format_string;
use redefmt_core::codec::encoding::SharedTestDispatcher;
use redefmt_decoder::{RedefmtDecoder, RedefmtDecoderCache, RedefmtFrame};
use tokio_util::codec::Decoder;

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fail/**/*.rs");
    t.pass("tests/ui/pass/**/*.rs");
}

macro_rules! assert_end_to_end {
    ($dispatcher:expr, $decoder:expr, $level:expr, $append_newline:literal, $method:path, $($arg:tt)*) => {
        $method!($($arg)*);

        let expected_frame = RedefmtFrame {
            level: $level,
            stamp: None,
            file_name: file!(),
            file_line: 18,
            format_string: &deferred_format_string!($($arg)*),
            append_newline: $append_newline,
            decoded_values: Default::default(),
        };

        assert_end_to_end_impl(&mut $dispatcher, &mut $decoder, expected_frame);
    };
}

#[test]
fn basic_macro_and_codec() {
    let mut dispatcher = SharedTestDispatcher::new();
    GlobalLogger::init_alloc_logger(dispatcher.clone(), None).unwrap();

    let decoder_cache = RedefmtDecoderCache::default();
    let mut decoder = RedefmtDecoder::new(&decoder_cache).expect("failed to init decoder");

    assert_end_to_end!(dispatcher, decoder, None, false, redefmt::print, "x");
    assert_end_to_end!(dispatcher, decoder, None, true, redefmt::println, "x");
    assert_end_to_end!(dispatcher, decoder, Some(Level::Trace), true, redefmt::trace, "x");
    assert_end_to_end!(dispatcher, decoder, Some(Level::Debug), true, redefmt::debug, "x");
    assert_end_to_end!(dispatcher, decoder, Some(Level::Info), true, redefmt::info, "x");
    assert_end_to_end!(dispatcher, decoder, Some(Level::Warn), true, redefmt::warn, "x");
    assert_end_to_end!(dispatcher, decoder, Some(Level::Error), true, redefmt::error, "x");
}

fn assert_end_to_end_impl(
    dispatcher: &mut SharedTestDispatcher,
    decoder: &mut RedefmtDecoder,
    expected_frame: RedefmtFrame,
) {
    let decoded_frame = decoder
        .decode(&mut dispatcher.take_bytes())
        .expect("failed to decode frame")
        .expect("decoder expected more bytes for frame");

    assert_eq!(expected_frame, decoded_frame);
}
