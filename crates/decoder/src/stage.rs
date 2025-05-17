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
        let segment_context = SegmentContext::new(print_statement.segments());
        // Hard to know capacity advance, (print statement args + any write
        // statement args). One could theoretically keep track of the number
        // of additional write args for a given print statement, store it in a
        // map before returning a decoded frame, and then reuse that count when
        // encountering the same print statement. Doubtful that there's anything
        // to gain from that.
        let decoded_segments = Vec::new();

        DecoderWants::PrintStatement(WantsPrintStatementStage {
            header,
            stamp,
            print_info,
            segment_context,
            decoded_segments,
        })
    }
}

pub struct WantsPrintStatementStage<'caches> {
    pub header: Header,
    pub stamp: Option<Stamp>,
    pub print_info: &'caches PrintInfo<'static>,
    pub segment_context: SegmentContext<'caches>,
    pub decoded_segments: Vec<DecodedSegment<'caches>>,
}
