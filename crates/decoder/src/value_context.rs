use redefmt_common::{
    codec::frame::{PointerWidth, TypeHint},
    identifiers::{CrateId, WriteStatementId},
};
use tokio_util::bytes::{Buf, BytesMut};

use crate::*;

#[derive(Default)]
pub struct ValueContext {
    pub length: Option<usize>,
    pub list_context: Option<ListValueContext>,
    pub write_context: Option<WriteStatementContext>,
}

impl ValueContext {
    pub fn get_or_store_usize_length(
        &mut self,
        src: &mut BytesMut,
        pointer_width: PointerWidth,
    ) -> Result<Option<usize>, RedefmtDecoderError> {
        if let Some(length) = self.length {
            return Ok(Some(length));
        };

        let Some(length) = DecoderUtils::get_target_usize(src, pointer_width) else {
            return Ok(None);
        };

        let length = usize::try_from(length).map_err(|_| RedefmtDecoderError::LengthOverflow(length))?;

        self.length = Some(length);

        Ok(Some(length))
    }

    pub fn get_or_store_u8_length(&mut self, src: &mut BytesMut) -> Option<usize> {
        if let Some(length) = self.length {
            return Some(length);
        };

        let Ok(length) = src.try_get_u8().map(Into::into) else {
            return None;
        };

        self.length = Some(length);

        Some(length)
    }
}

pub struct ListValueContext {
    pub buffer: Vec<Value>,
    pub element_context: Option<Box<ValueContext>>,
    pub element_type_hint: Option<TypeHint>,
}

impl ListValueContext {
    pub fn new(expected_length: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(expected_length),
            element_context: Default::default(),
            element_type_hint: None,
        }
    }

    pub fn get_or_insert_element_type_hint(
        &mut self,
        src: &mut BytesMut,
    ) -> Result<Option<TypeHint>, RedefmtDecoderError> {
        if let Some(type_hint) = self.element_type_hint {
            return Ok(Some(type_hint));
        }

        let Ok(type_hint_repr) = src.try_get_u8() else {
            return Ok(None);
        };

        let type_hint =
            TypeHint::from_repr(type_hint_repr).ok_or(RedefmtDecoderError::UnknownTypeHint(type_hint_repr))?;

        self.element_type_hint = Some(type_hint);

        Ok(Some(type_hint))
    }
}

#[derive(Default)]
pub struct WriteStatementContext {
    pub crate_id: Option<CrateId>,
    pub statement_id: Option<WriteStatementId>,
}

impl WriteStatementContext {
    pub fn get_or_store_crate_id(&mut self, src: &mut BytesMut) -> Option<CrateId> {
        if let Some(crate_id) = self.crate_id {
            return Some(crate_id);
        }

        let crate_id = src.try_get_u16().ok().map(CrateId::new)?;
        self.crate_id = Some(crate_id);
        Some(crate_id)
    }

    pub fn get_or_store_statement_id(&mut self, src: &mut BytesMut) -> Option<WriteStatementId> {
        if let Some(statement_id) = self.statement_id {
            return Some(statement_id);
        }

        let statement_id = src.try_get_u16().ok().map(WriteStatementId::new)?;
        self.statement_id = Some(statement_id);
        Some(statement_id)
    }
}
