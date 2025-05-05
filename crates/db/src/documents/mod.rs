mod print {
    use uuid::Uuid;

    use crate::*;

    pub struct PrintDocumentId(Uuid);

    // TODO: denormalize PrintDocument
    pub struct PrintDocument<'a> {
        info: PrintInfo,
        segments: Vec<Segment<'a>>,
    }
}
pub(crate) use print::{PrintDocument, PrintDocumentId};
