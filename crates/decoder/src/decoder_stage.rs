use redefmt_common::codec::frame::{Header, Stamp};
use redefmt_db::statement_table::print::{PrintInfo, PrintStatement};

use crate::*;

// Can't use generic state parameter on tokio_util::codec::Decoder
#[derive(Default)]
pub enum DecoderWants<'caches> {
    #[default]
    Header,
    Stamp(WantsStampStage),
    PrintCrateId(WantsPrintCrateIdStage),
    PrintStatementId(WantsPrintStatementIdStage<'caches>),
    PrintStatement(WantsPrintStatementStage<'caches>),
}

pub struct WantsStampStage {
    pub header: Header,
}

pub struct WantsPrintCrateIdStage {
    pub header: Header,
    pub stamp: Option<Stamp>,
}

impl WantsPrintCrateIdStage {
    pub fn next<'caches>(self, print_crate_value: &'caches CrateValue) -> DecoderWants<'caches> {
        let Self { header, stamp } = self;
        DecoderWants::PrintStatementId(WantsPrintStatementIdStage { header, stamp, print_crate_value })
    }
}

pub struct WantsPrintStatementIdStage<'caches> {
    pub header: Header,
    pub stamp: Option<Stamp>,
    pub print_crate_value: &'caches CrateValue,
}

impl<'caches> WantsPrintStatementIdStage<'caches> {
    pub fn next(self, print_statement: &'caches PrintStatement<'static>) -> DecoderWants<'caches> {
        let Self { header, stamp, .. } = self;

        let print_info = print_statement.info();
        let segment_decoder = SegmentsDecoder::new(header.pointer_width(), print_statement.segments());

        DecoderWants::PrintStatement(WantsPrintStatementStage { stamp, print_info, segment_decoder })
    }
}

pub struct WantsPrintStatementStage<'caches> {
    pub stamp: Option<Stamp>,
    pub print_info: &'caches PrintInfo<'static>,
    pub segment_decoder: SegmentsDecoder<'caches>,
}
