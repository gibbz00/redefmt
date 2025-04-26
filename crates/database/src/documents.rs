mod format {
    use core::any::TypeId;

    use crate::*;

    pub struct FormatDocumentId(TypeId);

    // Created when implementing Format
    pub enum FormatDocument {
        // Often from derived implementations
        Tree(NormalizedType),
        // Often from manual implementations
        WriteStatements(Vec<NormalizedSegment>),
    }
}
pub(crate) use format::{FormatDocument, FormatDocumentId};

mod print {
    use uuid::Uuid;

    use crate::*;

    pub struct PrintDocumentId(Uuid);

    // TODO: denormalize PrintDocument
    pub struct PrintDocument {
        info: PrintInfo,
        segments: Vec<NormalizedSegment>,
    }
}
pub(crate) use print::{PrintDocument, PrintDocumentId};
