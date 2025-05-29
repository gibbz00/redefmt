use redefmt_core::codec::frame::{PointerWidth, TypeHint};
use tokio_util::bytes::{Buf, BytesMut};

use crate::*;

pub struct DecoderUtils;

impl DecoderUtils {
    pub fn get_target_usize(src: &mut BytesMut, pointer_width: PointerWidth) -> Option<u64> {
        if src.len() < pointer_width.size() {
            return None;
        }

        let num = match pointer_width {
            PointerWidth::U16 => src.get_u16() as u64,
            PointerWidth::U32 => src.get_u32() as u64,
            PointerWidth::U64 => src.get_u64(),
        };

        Some(num)
    }

    pub fn get_target_isize(src: &mut BytesMut, pointer_width: PointerWidth) -> Option<i64> {
        if src.len() < pointer_width.size() {
            return None;
        }

        let num = match pointer_width {
            PointerWidth::U16 => src.get_i16() as i64,
            PointerWidth::U32 => src.get_i32() as i64,
            PointerWidth::U64 => src.get_i64(),
        };

        Some(num)
    }

    pub fn get_type_hint(src: &mut BytesMut) -> Result<Option<TypeHint>, RedefmtDecoderError> {
        let Ok(type_hint_repr) = src.try_get_u8() else {
            return Ok(None);
        };

        TypeHint::from_repr(type_hint_repr)
            .ok_or(RedefmtDecoderError::UnknownTypeHint(type_hint_repr))
            .map(Some)
    }
}
