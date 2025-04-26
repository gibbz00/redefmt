// TODO: premature optimization?
mod string {
    use alloc::string::String;

    use uuid::Uuid;

    // ID: UUID
    pub struct StringDocumentId(Uuid);

    /// Creates effectively an interning collection
    /// TODO: really necessary?
    pub struct StringDocument {
        str: String,
    }
}
pub(crate) use string::{StringDocument, StringDocumentId};

mod write {
    use redefmt_args::FormatOptions;
    use uuid::Uuid;

    use crate::*;

    pub struct WriteDocumentId(Uuid);

    // ID: write_id
    pub struct WriteDocument {
        segments: DisplaySegment,
    }
}
pub(crate) use write::{WriteDocument, WriteDocumentId};

mod r#type {
    use core::any::TypeId;

    use crate::*;

    pub struct TypeDocumentId(TypeId);

    // ID: TypeID
    pub struct TypeDocument {
        r#type: Type,
    }
}
pub(crate) use r#type::{TypeDocument, TypeDocumentId};

mod print {
    use uuid::Uuid;

    use crate::*;

    pub struct PrintDocumentId(Uuid);

    pub struct PrintDocument {
        segments: DisplaySegment,
        /// None for regular `println!()` statements
        log_level: Option<LogLevel>,
        // File / module inferred by DB file path?
        // Remaining would be line
        location: (),
    }

    pub enum LogLevel {
        Trace,
        Debug,
        Info,
        Warn,
        Error,
    }
}
pub(crate) use print::{LogLevel, PrintDocument, PrintDocumentId};
