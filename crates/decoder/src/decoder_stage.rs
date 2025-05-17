use redefmt::codec::frame::{Header, Stamp};
use redefmt_db::statement_table::print::{PrintInfo, PrintStatement};

use crate::*;

// Can't use generic state parameter on tokio_util::codec::Decoder
#[derive(Default)]
pub enum DecoderWants<'cache> {
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
    pub fn next<'cache>(self, print_crate: CrateContext<'cache>) -> DecoderWants<'cache> {
        let Self { header, stamp } = self;
        DecoderWants::PrintStatementId(WantsPrintStatementIdStage { header, stamp, print_crate })
    }
}

pub struct WantsPrintStatementIdStage<'cache> {
    pub header: Header,
    pub stamp: Option<Stamp>,
    pub print_crate: CrateContext<'cache>,
}

impl<'cache> WantsPrintStatementIdStage<'cache> {
    pub fn next(self, print_statement: &'cache PrintStatement<'static>) -> DecoderWants<'cache> {
        let Self { header, stamp, .. } = self;

        let print_info = print_statement.info();
        let segment_decoder = SegmentsDecoder::new(header.pointer_width(), print_statement.segments());

        DecoderWants::PrintStatement(WantsPrintStatementStage { stamp, print_info, segment_decoder })
    }
}

pub struct WantsPrintStatementStage<'cache> {
    pub stamp: Option<Stamp>,
    pub print_info: &'cache PrintInfo<'static>,
    pub segment_decoder: SegmentsDecoder<'cache>,
}
