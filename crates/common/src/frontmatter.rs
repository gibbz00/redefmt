use bitflags::bitflags;

use crate::*;

bitflags! {
    pub struct FrontMatter: u8 {
        const PLUS_16_WIDTH = 0b00000001;
        const PLUS_32_WIDTH = 0b00000010;
        // ... remaining reserved
        const STAMP = 0b10000000;
    }
}

impl FrontMatter {
    pub fn new(has_stamp: bool) -> Self {
        let mut front_matter = Self::empty();

        #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
        {
            front_matter |= Self::PLUS_16_WIDTH;
        }

        #[cfg(target_pointer_width = "64")]
        {
            front_matter |= Self::PLUS_32_WIDTH;
        }

        if has_stamp {
            front_matter |= Self::STAMP;
        }

        front_matter
    }

    pub fn pointer_width(&self) -> PointerWidth {
        // 48-bit pointer width is technically representable,
        // but we ignore this as a reality for now
        if self.contains(FrontMatter::PLUS_32_WIDTH) {
            PointerWidth::U64
        } else if self.contains(FrontMatter::PLUS_16_WIDTH) {
            PointerWidth::U32
        } else {
            return PointerWidth::U16;
        }
    }
}
