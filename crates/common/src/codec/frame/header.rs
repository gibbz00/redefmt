use bitflags::bitflags;

use crate::*;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Header: u8 {
        const PLUS_16_WIDTH = 0b00000001;
        const PLUS_32_WIDTH = 0b00000010;
        // ... remaining reserved
        const STAMP = 0b10000000;
    }
}

impl Header {
    pub fn new(has_stamp: bool) -> Self {
        let mut header = Self::empty();

        match PointerWidth::of_target() {
            PointerWidth::U16 => {}
            PointerWidth::U32 => header |= Self::PLUS_16_WIDTH,
            PointerWidth::U64 => {
                header |= Self::PLUS_16_WIDTH;
                header |= Self::PLUS_32_WIDTH;
            }
        }

        if has_stamp {
            header |= Self::STAMP;
        }

        header
    }

    pub fn pointer_width(&self) -> PointerWidth {
        // NOTE: 48-bit pointer width is technically representable, but we
        // ignore this as a reality for now since word sizes are usually in
        // powers of 2
        if self.contains(Header::PLUS_32_WIDTH) {
            PointerWidth::U64
        } else if self.contains(Header::PLUS_16_WIDTH) {
            PointerWidth::U32
        } else {
            return PointerWidth::U16;
        }
    }
}
