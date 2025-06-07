#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StatementWriterHint {
    Continue = 0,
    End = 1,
}

impl StatementWriterHint {
    pub const fn from_repr(repr: u8) -> Option<Self> {
        match repr {
            0 => Some(Self::Continue),
            1 => Some(Self::End),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hint_repr_bijectivity() {
        assert_bijectivity(StatementWriterHint::Continue);
        assert_bijectivity(StatementWriterHint::End);

        fn assert_bijectivity(hint: StatementWriterHint) {
            let repr = hint as u8;
            let from_repr = StatementWriterHint::from_repr(repr).unwrap();
            assert_eq!(hint, from_repr);
        }
    }
}
