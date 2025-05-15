use redefmt_common::codec::frame::PointerWidth;
use tokio_util::bytes::{Buf, BytesMut};

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
}
