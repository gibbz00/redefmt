use bitflags::bitflags;

pub enum PointerWidth {
    U16,
    U32,
    U64,
}

bitflags! {
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

        #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
        {
            header |= Self::PLUS_16_WIDTH;
        }

        #[cfg(target_pointer_width = "64")]
        {
            header |= Self::PLUS_32_WIDTH;
        }

        if has_stamp {
            header |= Self::STAMP;
        }

        header
    }

    pub fn pointer_width(&self) -> PointerWidth {
        // 48-bit pointer width is technically representable,
        // but we ignore this as a reality for now
        if self.contains(Header::PLUS_32_WIDTH) {
            PointerWidth::U64
        } else if self.contains(Header::PLUS_16_WIDTH) {
            PointerWidth::U32
        } else {
            return PointerWidth::U16;
        }
    }
}
