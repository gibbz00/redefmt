use redefmt_core::codec::frame::{Header, Level, Stamp};
use redefmt_db::statement_table::print::PrintStatement;

use crate::*;

// Can't use generic state parameter on tokio_util::codec::Decoder
#[derive(Default)]
pub enum FrameDecoderWants<'cache> {
    #[default]
    Header,
    Stamp(WantsStampStage),
    PrintCrateId(WantsPrintCrateIdStage),
    PrintStatementId(WantsPrintStatementIdStage<'cache>),
    PrintStatement(WantsPrintStatementStage<'cache>),
}

pub struct WantsStampStage {
    pub header: Header,
}

pub struct WantsPrintCrateIdStage {
    pub header: Header,
    pub stamp: Option<Stamp>,
}

impl WantsPrintCrateIdStage {
    pub fn next<'cache>(self, print_crate: CrateContext<'cache>) -> FrameDecoderWants<'cache> {
        let Self { header, stamp } = self;
        FrameDecoderWants::PrintStatementId(WantsPrintStatementIdStage { header, stamp, print_crate })
    }
}

pub struct WantsPrintStatementIdStage<'cache> {
    pub header: Header,
    pub stamp: Option<Stamp>,
    pub print_crate: CrateContext<'cache>,
}

impl<'cache> WantsPrintStatementIdStage<'cache> {
    pub fn next(self, print_statement: &'cache PrintStatement<'static>) -> FrameDecoderWants<'cache> {
        let Self { header, stamp, .. } = self;
        let level = header.level();
        let segment_decoder = SegmentsDecoder::new(header.pointer_width(), print_statement.format_expression());

        FrameDecoderWants::PrintStatement(WantsPrintStatementStage { level, stamp, print_statement, segment_decoder })
    }
}

pub struct WantsPrintStatementStage<'cache> {
    pub level: Option<Level>,
    pub stamp: Option<Stamp>,
    pub print_statement: &'cache PrintStatement<'static>,
    pub segment_decoder: SegmentsDecoder<'cache>,
}
