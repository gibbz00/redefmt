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
        segments: StringDocumentId,
    }

    pub enum WriteSegment<'a> {
        // TODO: just store string directly?
        Str(StringDocumentId),
        Arg(Value, FormatOptions<'a>),
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
