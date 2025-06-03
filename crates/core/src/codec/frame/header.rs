use bitflags::bitflags;

use crate::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Header(u8);

bitflags! {
    impl Header: u8 {
        const PLUS_16_WIDTH = 0b00000001;
        const PLUS_32_WIDTH = 0b00000010;
        const LEVEL_TRACE = 0b01000000;
        const LEVEL_DEBUG = 0b00100000;
        const LEVEL_INFO = 0b01100000;
        const LEVEL_WARN = 0b00010000;
        const LEVEL_ERROR = 0b01010000;
        // Value inverse to verbosity level to make room for higher verbosity levels, ex.
        // const LEVEL_CRITICAL = 0b01110000;
        // ... remaining reserved
        const STAMP = 0b10000000;
    }
}

impl Header {
    pub fn new(has_stamp: bool, level: Option<Level>) -> Self {
        let mut header = Self::empty();

        match PointerWidth::of_target() {
            PointerWidth::U16 => {}
            PointerWidth::U32 => header |= Self::PLUS_16_WIDTH,
            PointerWidth::U64 => {
                header |= Self::PLUS_16_WIDTH;
                header |= Self::PLUS_32_WIDTH;
            }
        }

        if let Some(level) = level {
            let level_bits = match level {
                Level::Error => Self::LEVEL_ERROR,
                Level::Warn => Self::LEVEL_WARN,
                Level::Info => Self::LEVEL_INFO,
                Level::Debug => Self::LEVEL_DEBUG,
                Level::Trace => Self::LEVEL_TRACE,
            };

            header |= level_bits;
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
            PointerWidth::U16
        }
    }

    pub fn level(&self) -> Option<Level> {
        let level_bits = self.bits() & 0b01110000;

        let level = match level_bits {
            bits if bits == Header::LEVEL_TRACE.bits() => Level::Trace,
            bits if bits == Header::LEVEL_DEBUG.bits() => Level::Debug,
            bits if bits == Header::LEVEL_INFO.bits() => Level::Info,
            bits if bits == Header::LEVEL_WARN.bits() => Level::Warn,
            bits if bits == Header::LEVEL_ERROR.bits() => Level::Error,
            _ => return None,
        };

        Some(level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level() {
        let mut header = Header::empty();

        assert!(header.level().is_none());

        header |= Header::STAMP;
        header |= Header::PLUS_16_WIDTH;
        header |= Header::PLUS_32_WIDTH;

        assert!(header.level().is_none());

        header |= Header::LEVEL_INFO;

        assert!(header.level().is_none_or(|header| header == Level::Info));
    }
}
