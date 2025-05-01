mod print {
    use uuid::Uuid;

    use crate::*;

    pub struct PrintDocumentId(Uuid);

    // TODO: denormalize PrintDocument
    pub struct PrintDocument {
        info: PrintInfo,
        segments: Vec<Segment>,
    }
}
pub(crate) use print::{PrintDocument, PrintDocumentId};
