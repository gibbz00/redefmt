pub fn is_false(bool: &bool) -> bool {
    !(*bool)
}

/// Use only then T::default() is cheap
pub fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}
