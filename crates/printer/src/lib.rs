//! # `redefmt-printer`

// TEMP:
#![allow(missing_docs)]

pub mod configuration;
pub(crate) use configuration::*;

mod pretty_printer;
pub(crate) use pretty_printer::PrettyPrinter;

// TEMP: move to examples
#[cfg(feature = "async-example")]
pub mod async_example {
    use std::io::Write;

    use anyhow::Context;
    use futures_util::TryStreamExt;
    use redefmt_decoder::{RedefmtDecoder, RedefmtDecoderCache};
    use tokio::io::AsyncRead;
    use tokio_util::codec::FramedRead;

    use crate::*;

    pub async fn run(input: impl AsyncRead, mut output: impl Write, config: PrettyPrinterConfig) -> anyhow::Result<()> {
        let decoder_cache = RedefmtDecoderCache::default();
        let decoder = RedefmtDecoder::new(&decoder_cache).context("failed to init decoder")?;
        let framed_read = FramedRead::new(input, decoder);
        pin_utils::pin_mut!(framed_read);

        let mut pretty_printer = PrettyPrinter::new(config);

        while let Some(redefmt_frame) = framed_read.try_next().await.context("failed to decode redefmt frame")? {
            let log_statement = pretty_printer
                .log_statement(redefmt_frame)
                .context("failed create log statement")?;

            output
                .write_all(log_statement.as_bytes())
                .context("failed to write log statement to output")?;

            output.flush()?;
        }

        Ok(())
    }
}
