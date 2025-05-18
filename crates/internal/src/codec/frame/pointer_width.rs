#[derive(Debug, Clone, Copy)]
pub enum PointerWidth {
    U16,
    U32,
    U64,
}

impl PointerWidth {
    pub fn of_target() -> PointerWidth {
        #[cfg(target_pointer_width = "16")]
        {
            PointerWidth::U16
        }

        #[cfg(target_pointer_width = "32")]
        {
            PointerWidth::U32
        }

        #[cfg(target_pointer_width = "64")]
        {
            PointerWidth::U64
        }
    }

    pub fn size(&self) -> usize {
        match self {
            PointerWidth::U16 => 2,
            PointerWidth::U32 => 4,
            PointerWidth::U64 => 8,
        }
    }
}
