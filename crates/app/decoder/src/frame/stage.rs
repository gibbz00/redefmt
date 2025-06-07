use redefmt_core::frame::{Header, Level, Stamp};
use redefmt_db::{crate_table::CrateName, statement_table::print::PrintStatement};

use crate::*;

// Can't use generic state parameter on tokio_util::codec::Decoder
#[derive(Default)]
// IMPROVEMENT: benchmark if there's anything gained
// from placing `Self::PrintStatement` behind a `Box`
#[allow(clippy::large_enum_variant)]
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
        let Self { header, stamp, print_crate } = self;

        let level = header.level();
        let crate_name = &print_crate.record.name;
        let segment_decoder = SegmentsDecoder::new(header.pointer_width(), &print_statement.stored_expression);

        FrameDecoderWants::PrintStatement(WantsPrintStatementStage {
            level,
            stamp,
            crate_name,
            print_statement,
            segment_decoder,
        })
    }
}

pub struct WantsPrintStatementStage<'cache> {
    pub level: Option<Level>,
    pub stamp: Option<Stamp>,
    pub crate_name: &'cache CrateName<'static>,
    pub print_statement: &'cache PrintStatement<'static>,
    pub segment_decoder: SegmentsDecoder<'cache>,
}
